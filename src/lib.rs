use std::{
    borrow::Cow,
    cmp::min,
    convert::TryFrom,
    io::{self, prelude::*, BufReader, BufWriter},
    net::TcpListener,
    os::unix::net::UnixListener,
};

use gdb_remote_protocol::{
    Error, FileSystem, Handler, Id, LibcFS, MemoryRegion, ProcessType,
    Signal, StopReason, ThreadId, VCont, VContFeature,
};
use num_traits::FromPrimitive;
use log::debug;
use structopt::StructOpt;

mod os;

use os::{Os, Registers, Target};

#[allow(unused)]
const ERROR_PARSE_STRING: u8 = std::u8::MAX;
#[allow(unused)]
const ERROR_GET_PATH: u8 = std::u8::MAX - 1;

#[derive(Debug, StructOpt)]
pub struct Opt {
    /// The address which to bind the server to
    #[structopt(short = "a", long = "addr", default_value = "127.0.0.1:64126")]
    pub addr: String,
    /// The type of address specified
    #[structopt(short = "t", long = "type", default_value = "tcp", possible_values = &["tcp", "unix", "stdio"])]
    pub kind: String,
    /// The program that should be debugged
    pub program: String,
    /// The arguments of the program
    pub args: Vec<String>,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub struct App {
    tracee: Os,
    fs: LibcFS,
}
impl Handler for App {
    fn attached(&self, _pid: Option<u64>) -> Result<ProcessType> {
        Ok(ProcessType::Created)
    }
    fn halt_reason(&self) -> Result<StopReason> {
        Ok(self.tracee.status())
    }
    fn read_general_registers(&self) -> Result<Vec<u8>> {
        let regs = self.tracee.getregs()?;

        let mut bytes = Vec::new();
        regs.encode(&mut bytes);

        Ok(bytes)
    }
    fn write_general_registers(&self, content: &[u8]) -> Result<()> {
        let regs = Registers::decode(content);
        self.tracee.setregs(&regs)?;
        Ok(())
    }
    fn read_memory(&self, region: MemoryRegion) -> Result<Vec<u8>> {
        let mut buf = vec![0; region.length as usize];
        self.tracee.getmem(region.address as usize, &mut buf)?;
        Ok(buf)
    }
    fn write_memory(&self, address: u64, bytes: &[u8]) -> Result<()> {
        self.tracee.setmem(address as usize, bytes)?;
        Ok(())
    }
    fn query_supported_features(&self) -> Vec<String> {
        vec![
            String::from("qXfer:features:read+"),
            String::from("qXfer:exec-file:read+"),
        ]
    }
    fn query_supported_vcont(&self) -> Result<Cow<'static, [VContFeature]>> {
        Ok(Cow::Borrowed(&[
            VContFeature::Continue,
            VContFeature::ContinueWithSignal,
            VContFeature::Step,
            VContFeature::StepWithSignal,
            VContFeature::RangeStep,
        ]))
    }
    fn thread_list(&self, reset: bool) -> Result<Vec<ThreadId>> {
        if reset {
            let id = Id::Id(self.tracee.pid());
            Ok(vec![
                ThreadId { pid: id, tid: id },
            ])
        } else {
            Ok(Vec::new())
        }
    }
    fn vcont(&self, actions: Vec<(VCont, Option<ThreadId>)>) -> Result<StopReason> {
        for (cmd, id) in &actions {
            let id = id.unwrap_or(ThreadId { pid: Id::All, tid: Id::All });
            debug!("Continuing thread: {:?}", id);
            debug!("Continuing PID: {:?}", self.tracee.pid());
            match (id.pid, id.tid) {
                (Id::Id(pid), _) if pid != self.tracee.pid() => continue,
                (_, Id::Id(tid)) if tid != self.tracee.pid() => continue,
                (_, _) => (),
            }
            match *cmd {
                VCont::Continue => {
                    self.tracee.cont(None)?;
                }
                VCont::ContinueWithSignal(signal) => {
                    self.tracee.cont(Signal::from_u8(signal).and_then(Signal::to_libc).map(|s| s as u8))?;
                }
                VCont::Step => {
                    self.tracee.step(None)?;
                }
                VCont::StepWithSignal(signal) => {
                    self.tracee.step(Signal::from_u8(signal).and_then(Signal::to_libc).map(|s| s as u8))?;
                }
                VCont::RangeStep(ref range) => {
                    // std::ops::Range<T: Copy> should probably also be Copy, but it isn't.
                    self.tracee.resume(range.clone())?;
                }
                _ => return Err(Error::Unimplemented),
            }
            break;
        }

        let status = self.tracee.status();
        debug!("vCont sending status {:?}", status);
        Ok(status)
    }
    fn read_bytes(&self, object: String, annex: String, offset: u64, length: u64) -> Result<(Vec<u8>, bool)> {
        let transfer_bytes = |source: &[u8]| -> Result<(Vec<u8>, bool)> {
            let start = usize::try_from(offset).expect("usize < u64");
            let end = start.saturating_add(usize::try_from(length).expect("usize < u64"));
            if start >= source.len() {
                return Ok((Vec::new(), true));
            }
            let slice = &source[start..min(end, source.len())];
            Ok((Vec::from(slice), false))
        };
        match (&*object, &*annex) {
            ("features", "target.xml") => {
                let target_xml = include_bytes!("../target-desc.xml");
                transfer_bytes(&target_xml[..])
            },
            ("exec-file", pid) => {
                let pid = usize::from_str_radix(pid, 16).map_err(|_| Error::Error(ERROR_PARSE_STRING))?;
                let path = self.tracee.path(pid)?;
                transfer_bytes(&path[..])
            },
            _ => Err(Error::Unimplemented),
        }
    }
    fn fs(&self) -> Result<&dyn FileSystem, ()> {
        Ok(&self.fs)
    }
}

pub fn main(mut opt: Opt) -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    opt.args.insert(0, opt.program.clone());

    let stdin = io::stdin();
    let stdout = io::stdout();

    let (mut reader, mut writer): (Box<dyn Read>, Box<dyn Write>) = if opt.kind == "unix" {
        let listener = UnixListener::bind(opt.addr)?;
        let (writer, _addr) = listener.accept()?;
        (
            Box::new(BufReader::new(writer.try_clone()?)),
            Box::new(BufWriter::new(writer)),
        )
    } else if opt.kind == "stdio" {
        (
            Box::new(stdin.lock()),
            Box::new(BufWriter::new(stdout.lock())),
        )
    } else {
        assert_eq!(opt.kind, "tcp");
        let listener = TcpListener::bind(opt.addr)?;
        let (writer, _addr) = listener.accept()?;
        (
            Box::new(BufReader::new(writer.try_clone()?)),
            Box::new(BufWriter::new(writer)),
        )
    };

    let tracee = Os::new(opt.program, opt.args)?;

    gdb_remote_protocol::process_packets_from(&mut reader, &mut writer, App {
        tracee,
        fs: LibcFS::default(),
    });

    Ok(())
}
