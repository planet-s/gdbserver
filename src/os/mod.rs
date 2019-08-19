use crate::Result;

use std::ops::RangeBounds;

mod regs;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod sys;

pub use regs::Registers;
pub use sys::Os;

pub enum Status {
    Exited(i32),
    Signaled(i32),
    Stopped(i32),
}

pub trait Target: Sized {
    /// Spawn a new tracee and return a tracer for it
    fn new(program: String, args: Vec<String>) -> Result<Os>;

    /// Get the last status of the tracee
    fn status(&mut self) -> Status;

    /// Read all the process register
    fn getregs(&mut self) -> Result<Registers, i32>;

    /// Read all the process register
    fn setregs(&mut self, regs: &Registers) -> Result<(), i32>;

    /// Read a region of memory from tracee
    fn getmem(&mut self, src: usize, dest: &mut [u8]) -> Result<usize, i32>;

    /// Read a region of memory from tracee
    fn setmem(&mut self, src: &[u8], dest: usize) -> Result<(), i32>;

    /// Single-step one instruction, return instruction pointer
    fn step(&mut self, signal: Option<u8>) -> Result<u64>;

    /// Resume execution while instruction pointer is inside the range
    fn resume<R>(&mut self, range: R) -> Result<u64>
        where R: RangeBounds<u64>
    {
        let mut last = None;
        loop {
            let rip = self.step(None)?;
            println!("{:X}", rip);
            // Break if outside the range or if in what appears to be
            // an infinite loop? Somehow this seems to sometimes occur
            // and I don't know what it means
            if !range.contains(&rip) || last == Some(rip) {
                break Ok(rip);
            }
            last = Some(rip);
        }
    }

    /// Continue execution until signal or other breakpoint
    fn cont(&mut self, signal: Option<u8>) -> Result<()>;
}
