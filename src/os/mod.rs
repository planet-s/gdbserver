use crate::Result;

use std::ops::RangeBounds;

use gdb_remote_protocol::StopReason;

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
    fn status(&self) -> StopReason;

    /// Get the process/thread id
    fn pid(&self) -> u32;

    /// Read all the process register
    fn getregs(&self) -> Result<Registers>;

    /// Read all the process register
    fn setregs(&self, regs: &Registers) -> Result<()>;

    /// Read a region of memory from tracee
    fn getmem(&self, src: usize, dest: &mut [u8]) -> Result<usize>;

    /// Read a region of memory from tracee
    fn setmem(&self, src: &[u8], dest: usize) -> Result<()>;

    /// Single-step one instruction, return instruction pointer
    fn step(&self, signal: Option<u8>) -> Result<Option<u64>>;

    /// Resume execution while instruction pointer is inside the range
    fn resume<R>(&self, range: R) -> Result<()>
    where
        R: RangeBounds<u64>,
    {
        loop {
            let rip = self.step(None)?;
            println!("{:X?}", rip);
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
