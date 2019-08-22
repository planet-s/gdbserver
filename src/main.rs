use gdb_remote_protocol::{
    Error, Handler, MemoryRegion, ProcessType, StopReason, ThreadId, VCont, VContFeature,
};
use structopt::StructOpt;

use std::{borrow::Cow, net::TcpListener};

mod os;

use os::{Os, Registers, Target};

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
    fn query_supported_vcont(&self) -> Result<Cow<'static, [VContFeature]>> {
        Ok(Cow::Borrowed(&[
            VContFeature::Continue,
            VContFeature::ContinueWithSignal,
            VContFeature::Step,
            VContFeature::StepWithSignal,
            VContFeature::RangeStep,
        ]))
    }
    fn vcont(&self, actions: Vec<(VCont, Option<ThreadId>)>) -> Result<StopReason> {
        if let Some((cmd, _id)) = actions.first() {
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
        }

        Ok(self.tracee.status())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut opt = Opt::from_args();
    opt.args.insert(0, opt.program.clone());

    let mut writer = {
        let listener = TcpListener::bind(opt.addr)?;
        let (stream, _addr) = listener.accept()?;
        stream
    };
    let mut reader = writer.try_clone()?;

    let tracee = Os::new(opt.program, opt.args)?;

    gdb_remote_protocol::process_packets_from(&mut reader, &mut writer, App { tracee });

    Ok(())
}
