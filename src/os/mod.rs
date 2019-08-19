use crate::Result;

use std::ops::RangeBounds;

mod regs;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod sys;

pub use regs::Registers;
pub use sys::Os;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Status {
    Exited(i32),
    Signaled(i32),
    Stopped(i32),
}
impl Status {
    pub fn is_exited(&self) -> bool {
        match self {
            Status::Exited(_) => true,
            _ => false
        }
    }
    pub fn is_signaled(&self) -> bool {
        match self {
            Status::Signaled(_) => true,
            _ => false
        }
    }
    pub fn is_stopped(&self) -> bool {
        match self {
            Status::Stopped(_) => true,
            _ => false
        }
    }
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
    fn step(&mut self, signal: Option<u8>) -> Result<Option<u64>>;

    /// Resume execution while instruction pointer is inside the range
    fn resume<R>(&mut self, range: R) -> Result<()>
        where R: RangeBounds<u64>
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
    fn cont(&mut self, signal: Option<u8>) -> Result<()>;
}
