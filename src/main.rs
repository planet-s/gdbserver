use std::{cmp::min, borrow::Cow, net::TcpListener, convert::TryFrom};

use gdb_remote_protocol::{
    Error, FileSystem, Handler, Id, LibcFS, MemoryRegion, ProcessType,
    StopReason, ThreadId, VCont, VContFeature,
};
use log::debug;
use structopt::StructOpt;

mod os;

use os::{Os, Registers, Target};

const ERROR_PARSE_STRING: u8 = 0;
const ERROR_GET_PATH: u8 = 1;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The address which to bind the server to
    #[structopt(short = "a", long = "addr", default_value = "0.0.0.0:64126")]
    addr: String,
    /// The program that should be debugged
    program: String,
    /// The arguments of the program
    args: Vec<String>,
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
        self.tracee.setmem(bytes, address as usize)?;
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
                    self.tracee.cont(Some(signal))?;
                }
                VCont::Step => {
                    self.tracee.step(None)?;
                }
                VCont::StepWithSignal(signal) => {
                    self.tracee.step(Some(signal))?;
                }
                VCont::RangeStep(ref range) => {
                    // std::ops::Range<T: Copy> should probably also be Copy, but it isn't.
                    self.tracee.resume(range.clone())?;
                }
                _ => return Err(Error::Unimplemented),
            }
            break;
        }

        Ok(self.tracee.status())
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let mut opt = Opt::from_args();
    opt.args.insert(0, opt.program.clone());

    let mut writer = {
        let listener = TcpListener::bind(opt.addr)?;
        let (stream, _addr) = listener.accept()?;
        stream
    };
    let mut reader = writer.try_clone()?;

    let tracee = Os::new(opt.program, opt.args)?;

    gdb_remote_protocol::process_packets_from(&mut reader, &mut writer, App {
        tracee,
        fs: LibcFS::default(),
    });

    Ok(())
}
