use crate::Result;

use std::ops::RangeBounds;

use gdb_remote_protocol::{StopReason, Signal};

mod regs;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod sys;
#[cfg(target_os = "redox")]
#[path = "redox.rs"]
mod sys;

pub use regs::Registers;
pub use sys::Os;

pub trait Target: Sized {
    /// Spawn a new tracee and return a tracer for it
    fn new(program: String, args: Vec<String>) -> Result<Os, Box<dyn std::error::Error>>;

    /// Get the last status of the tracee
    fn status_native(&self) -> StopReason;

    /// Get the last status of the tracee, but convert it from libc signals to GDB signals
    fn status(&self) -> StopReason {
        match self.status_native() {
            StopReason::Signal(sig) => StopReason::Signal(
                Signal::from_libc(libc::c_int::from(sig)).unwrap_or(Signal::SIGTRAP) as u8,
            ),
            StopReason::ExitedWithSignal(pid, sig) => StopReason::ExitedWithSignal(
                pid,
                Signal::from_libc(libc::c_int::from(sig)).unwrap_or(Signal::SIGTERM) as u8,
            ),
            status => status,
        }
    }

    /// Get the process/thread id
    fn pid(&self) -> u32;

    /// Read all the process register
    fn getregs(&self) -> Result<Registers>;

    /// Read all the process register
    fn setregs(&self, regs: &Registers) -> Result<()>;

    /// Read a region of memory from tracee
    fn getmem(&self, address: usize, memory: &mut [u8]) -> Result<usize>;

    /// Read a region of memory from tracee
    fn setmem(&self, address: usize, memory: &[u8]) -> Result<()>;

    /// Single-step one instruction, return instruction pointer
    fn step(&self, signal: Option<u8>) -> Result<Option<u64>>;

    /// Resume execution while instruction pointer is inside the range
    fn resume<R>(&self, range: R) -> Result<()>
    where
        R: RangeBounds<u64>,
    {
        loop {
            let rip = self.step(None)?;
            //println!("{:X?}", rip);
            if rip.map(|rip| !range.contains(&rip)).unwrap_or(true) {
                break;
            }
        }
        Ok(())
    }

    /// Continue execution until signal or other breakpoint
    fn cont(&self, signal: Option<u8>) -> Result<()>;

    /// Return the executable that's being run for specified PID
    fn path(&self, pid: usize) -> Result<Vec<u8>>;
}
