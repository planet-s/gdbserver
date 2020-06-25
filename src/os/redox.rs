use super::Registers;
use crate::Result;

use std::{
    cell::{Cell, RefCell},
    fs,
    io,
    mem,
    os::unix::ffi::OsStrExt,
};

use gdb_remote_protocol::{Error, StopReason};
use log::error;
use strace::{Flags, Tracer};
use syscall::flag::*;

pub struct Os {
    pid: usize,
    last_status: Cell<usize>,
    tracer: RefCell<Tracer>,
}

trait FromOsError<T>: Sized {
    fn from_os_error(error: T) -> Self;
}

impl FromOsError<syscall::Error> for Error {
    fn from_os_error(error: syscall::Error) -> Self {
        Error::Error(error.errno as u8)
    }
}
impl FromOsError<io::Error> for Error {
    fn from_os_error(error: io::Error) -> Self {
        Error::Error(error.raw_os_error().unwrap_or(0) as u8)
    }
}
impl FromOsError<syscall::Error> for io::Error {
    fn from_os_error(error: syscall::Error) -> Self {
        io::Error::from_raw_os_error(error.errno)
    }
}
impl FromOsError<io::Error> for Box<dyn std::error::Error> {
    fn from_os_error(error: io::Error) -> Self {
        Box::new(error)
    }
}
impl FromOsError<syscall::Error> for Box<dyn std::error::Error> {
    fn from_os_error(error: syscall::Error) -> Self {
        Box::new(io::Error::from_os_error(error))
    }
}

macro_rules! e {
    ($result:expr) => {{
        match $result {
            Ok(inner) => inner,
            Err(err) => return Err(FromOsError::from_os_error(err)),
        }
    }};
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProcessState {
    Running,
    Exited,
}

impl Os {
    /// Continues the tracer with the specified flags, and the breakpoint
    /// flag. If the process exits, we waitpid the child and set the status. We
    /// also return `Exited` to signal that the process is no longer
    /// alive. Returns `Running` and sets status to SIGSTOP if the process does
    /// not exit.
    fn next(&self, flags: Flags) -> Result<ProcessState> {
        let mut tracer = self.tracer.borrow_mut();

        match tracer.next(flags | Flags::STOP_BREAKPOINT) {
            Ok(_event) => {
                // Just pretend ptrace SIGSTOP:ped this
                let status = (SIGSTOP << 8) | 0x7f;
                assert!(syscall::wifstopped(status));
                assert_eq!(syscall::wstopsig(status), SIGSTOP);

                Ok(ProcessState::Running)
            },
            Err(err) if err.raw_os_error() == Some(syscall::ESRCH) => {
                let mut status = 0;
                e!(syscall::waitpid(0, &mut status, WNOHANG));
                self.last_status.set(status);

                Ok(ProcessState::Exited)
            },
            Err(err) => e!(Err(err)),
        }
    }
}

impl super::Target for Os {
    fn new(program: String, args: Vec<String>) -> Result<Self, Box<dyn std::error::Error>> {
        unsafe {
            let pid = e!(syscall::clone(CloneFlags::empty()));

            if pid == 0 {
                // Must not drop any memory, and not unwind (panic).
                let result = (|| -> io::Result<()> {
                    let args = args.iter()
                        .map(|s| [s.as_ptr() as usize, s.len()])
                        .collect::<Vec<[usize; 2]>>();
                    let vars = std::env::vars_os()
                        .map(|(mut key, value)| {
                            key.push("=");
                            key.push(&value);

                            let slice = key.as_bytes();
                            let res = [slice.as_ptr() as usize, slice.len()];
                            mem::forget(key);
                            res
                        })
                        .collect::<Vec<[usize; 2]>>();

                    let program = e!(syscall::open(program.as_bytes(), O_RDONLY | O_CLOEXEC));

                    let pid = e!(syscall::getpid());
                    e!(syscall::kill(pid, SIGSTOP));

                    e!(syscall::fexec(program, &args, &vars));
                    Ok(())
                })();

                match result {
                    Ok(()) => {
                        error!("fexec(...) should not be able to succeed");
                        let _ = syscall::exit(1);
                        unreachable!();
                    },
                    Err(err) => {
                        error!("failure: {}", err);
                        let _ = syscall::exit(1);
                        unreachable!();
                    },
                }
            } else {
                // Wait for tracee to stop
                let mut status = 0;
                e!(syscall::waitpid(pid, &mut status, WaitFlags::WUNTRACED));

                // Attach tracer
                let mut tracer = e!(Tracer::attach(pid));

                // Step past fexec
                e!(syscall::kill(pid, SIGCONT));
                e!(tracer.next(Flags::STOP_PRE_SYSCALL));
                assert_eq!(e!(tracer.regs.get_int()).return_value(), syscall::SYS_FEXEC);

                // TODO: Don't stop only on syscall, stop on first instruction.
                // Single-stepping doesn't work across fexec yet for some reason.
                // e!(tracer.next(Flags::STOP_SINGLESTEP));
                e!(tracer.next(Flags::STOP_EXEC));

                Ok(Os {
                    pid,
                    last_status: Cell::new(status),
                    tracer: RefCell::new(tracer),
                })
            }
        }
    }

    fn status(&self) -> StopReason {
        if syscall::wifexited(self.last_status.get()) {
            StopReason::Exited(
                self.pid as _,
                syscall::wexitstatus(self.last_status.get()) as _,
            )
        } else if syscall::wifsignaled(self.last_status.get()) {
            StopReason::ExitedWithSignal(
                self.pid as _,
                syscall::wtermsig(self.last_status.get()) as _,
            )
        } else if syscall::wifstopped(self.last_status.get()) {
            StopReason::Signal(syscall::wstopsig(self.last_status.get()) as _)
        } else {
            unimplemented!("TODO: Implement status {}", self.last_status.get());
        }
    }

    fn pid(&self) -> u32 {
        return self.pid as _;
    }

    fn getregs(&self) -> Result<Registers> {
        let mut tracer = self.tracer.borrow_mut();
        let int = e!(tracer.regs.get_int()).0;
        let float = e!(tracer.regs.get_float()).0;

        let mut registers = Registers::default();
        registers.r15 = Some(int.r15 as _);
        registers.r14 = Some(int.r14 as _);
        registers.r13 = Some(int.r13 as _);
        registers.r12 = Some(int.r12 as _);
        registers.rbp = Some(int.rbp as _);
        registers.rbx = Some(int.rbx as _);
        registers.r11 = Some(int.r11 as _);
        registers.r10 = Some(int.r10 as _);
        registers.r9 = Some(int.r9 as _);
        registers.r8 = Some(int.r8 as _);
        registers.rax = Some(int.rax as _);
        registers.rcx = Some(int.rcx as _);
        registers.rdx = Some(int.rdx as _);
        registers.rsi = Some(int.rsi as _);
        registers.rdi = Some(int.rdi as _);
        registers.rip = Some(int.rip as _);
        registers.cs = Some(int.cs as _);
        registers.eflags = Some(int.rflags as _);
        registers.rsp = Some(int.rsp as _);
        registers.ss = Some(int.ss as _);
        // registers.ds = Some(int.ds as _);
        // registers.es = Some(int.es as _);
        registers.fs = Some(int.fs as _);
        // registers.gs = Some(int.gs as _);

        registers.fctrl = Some(float.fcw as _);
        registers.fop = Some(float.fop as _);

        registers.st0 = Some(float.st_space[0] as _);
        registers.st1 = Some(float.st_space[1] as _);
        registers.st2 = Some(float.st_space[2] as _);
        registers.st3 = Some(float.st_space[3] as _);
        registers.st4 = Some(float.st_space[4] as _);
        registers.st5 = Some(float.st_space[5] as _);
        registers.st6 = Some(float.st_space[6] as _);
        registers.st7 = Some(float.st_space[7] as _);

        registers.xmm0 = Some(float.xmm_space[0] as _);
        registers.xmm1 = Some(float.xmm_space[1] as _);
        registers.xmm2 = Some(float.xmm_space[2] as _);
        registers.xmm3 = Some(float.xmm_space[3] as _);
        registers.xmm4 = Some(float.xmm_space[4] as _);
        registers.xmm5 = Some(float.xmm_space[5] as _);
        registers.xmm6 = Some(float.xmm_space[6] as _);
        registers.xmm7 = Some(float.xmm_space[7] as _);
        registers.xmm8 = Some(float.xmm_space[8] as _);
        registers.xmm9 = Some(float.xmm_space[9] as _);
        registers.xmm10 = Some(float.xmm_space[10] as _);
        registers.xmm11 = Some(float.xmm_space[11] as _);
        registers.xmm12 = Some(float.xmm_space[12] as _);
        registers.xmm13 = Some(float.xmm_space[13] as _);
        registers.xmm14 = Some(float.xmm_space[14] as _);
        registers.xmm15 = Some(float.xmm_space[15] as _);

        registers.mxcsr = Some(float.mxcsr);
        // registers.fs_base = Some(int.fs_base as _);
        // registers.gs_base = Some(int.gs_base as _);
        registers.orig_rax = Some(int.rax as _);

        Ok(registers)
    }

    fn setregs(&self, registers: &Registers) -> Result<()> {
        let mut int = syscall::IntRegisters::default();
        let mut float = syscall::FloatRegisters::default();

        int.r15 = registers.r15.map(|r| r as _).unwrap_or(int.r15);
        int.r14 = registers.r14.map(|r| r as _).unwrap_or(int.r14);
        int.r13 = registers.r13.map(|r| r as _).unwrap_or(int.r13);
        int.r12 = registers.r12.map(|r| r as _).unwrap_or(int.r12);
        int.rbp = registers.rbp.map(|r| r as _).unwrap_or(int.rbp);
        int.rbx = registers.rbx.map(|r| r as _).unwrap_or(int.rbx);
        int.r11 = registers.r11.map(|r| r as _).unwrap_or(int.r11);
        int.r10 = registers.r10.map(|r| r as _).unwrap_or(int.r10);
        int.r9 = registers.r9.map(|r| r as _).unwrap_or(int.r9);
        int.r8 = registers.r8.map(|r| r as _).unwrap_or(int.r8);
        int.rax = registers.rax.map(|r| r as _).unwrap_or(int.rax);
        int.rcx = registers.rcx.map(|r| r as _).unwrap_or(int.rcx);
        int.rdx = registers.rdx.map(|r| r as _).unwrap_or(int.rdx);
        int.rsi = registers.rsi.map(|r| r as _).unwrap_or(int.rsi);
        int.rdi = registers.rdi.map(|r| r as _).unwrap_or(int.rdi);
        int.rip = registers.rip.map(|r| r as _).unwrap_or(int.rip);
        int.cs = registers.cs.map(|r| r as _).unwrap_or(int.cs);
        int.rflags = registers.eflags.map(|r| r as _).unwrap_or(int.rflags);
        int.rsp = registers.rsp.map(|r| r as _).unwrap_or(int.rsp);
        int.ss = registers.ss.map(|r| r as _).unwrap_or(int.ss);
        // int.ds = registers.ds.map(|r| r as _).unwrap_or(int.ds);
        // int.es = registers.es.map(|r| r as _).unwrap_or(int.es);
        int.fs = registers.fs.map(|r| r as _).unwrap_or(int.fs);
        // int.gs = registers.gs.map(|r| r as _).unwrap_or(int.gs);

        float.fcw = registers.fctrl.map(|r| r as _).unwrap_or(float.fcw);
        float.fop = registers.fop.map(|r| r as _).unwrap_or(float.fop);

        float.st_space[0] = registers.st0.map(|r| r as _).unwrap_or(float.st_space[0]);
        float.st_space[1] = registers.st1.map(|r| r as _).unwrap_or(float.st_space[1]);
        float.st_space[2] = registers.st2.map(|r| r as _).unwrap_or(float.st_space[2]);
        float.st_space[3] = registers.st3.map(|r| r as _).unwrap_or(float.st_space[3]);
        float.st_space[4] = registers.st4.map(|r| r as _).unwrap_or(float.st_space[4]);
        float.st_space[5] = registers.st5.map(|r| r as _).unwrap_or(float.st_space[5]);
        float.st_space[6] = registers.st6.map(|r| r as _).unwrap_or(float.st_space[6]);
        float.st_space[7] = registers.st7.map(|r| r as _).unwrap_or(float.st_space[7]);

        float.xmm_space[0] = registers.xmm0.map(|r| r as _).unwrap_or(float.xmm_space[0]);
        float.xmm_space[1] = registers.xmm1.map(|r| r as _).unwrap_or(float.xmm_space[1]);
        float.xmm_space[2] = registers.xmm2.map(|r| r as _).unwrap_or(float.xmm_space[2]);
        float.xmm_space[3] = registers.xmm3.map(|r| r as _).unwrap_or(float.xmm_space[3]);
        float.xmm_space[4] = registers.xmm4.map(|r| r as _).unwrap_or(float.xmm_space[4]);
        float.xmm_space[5] = registers.xmm5.map(|r| r as _).unwrap_or(float.xmm_space[5]);
        float.xmm_space[6] = registers.xmm6.map(|r| r as _).unwrap_or(float.xmm_space[6]);
        float.xmm_space[7] = registers.xmm7.map(|r| r as _).unwrap_or(float.xmm_space[7]);
        float.xmm_space[8] = registers.xmm8.map(|r| r as _).unwrap_or(float.xmm_space[8]);
        float.xmm_space[9] = registers.xmm9.map(|r| r as _).unwrap_or(float.xmm_space[9]);
        float.xmm_space[10] = registers
            .xmm10
            .map(|r| r as _)
            .unwrap_or(float.xmm_space[10]);
        float.xmm_space[11] = registers
            .xmm11
            .map(|r| r as _)
            .unwrap_or(float.xmm_space[11]);
        float.xmm_space[12] = registers
            .xmm12
            .map(|r| r as _)
            .unwrap_or(float.xmm_space[12]);
        float.xmm_space[13] = registers
            .xmm13
            .map(|r| r as _)
            .unwrap_or(float.xmm_space[13]);
        float.xmm_space[14] = registers
            .xmm14
            .map(|r| r as _)
            .unwrap_or(float.xmm_space[14]);
        float.xmm_space[15] = registers
            .xmm15
            .map(|r| r as _)
            .unwrap_or(float.xmm_space[15]);

        float.mxcsr = registers.mxcsr.unwrap_or(float.mxcsr);
        // int.fs_base = registers.fs_base.map(|r| r as _).unwrap_or(int.fs_base);
        // int.gs_base = registers.gs_base.map(|r| r as _).unwrap_or(int.gs_base);
        int.rax = registers.orig_rax.map(|r| r as _).unwrap_or(int.rax);

        let mut tracer = self.tracer.borrow_mut();
        e!(tracer.regs.set_int(&strace::IntRegisters(int)));
        e!(tracer.regs.set_float(&strace::FloatRegisters(float)));

        Ok(())
    }

    fn getmem(&self, src: usize, dest: &mut [u8]) -> Result<usize> {
        // TODO: Don't report errors when able to read part of requested?
        // Also implement this in the Redox kernel perhaps
        let mut tracer = self.tracer.borrow_mut();
        e!(tracer.mem.read(src as *const u8, dest));
        Ok(dest.len())
    }

    fn setmem(&self, src: &[u8], dest: usize) -> Result<()> {
        let mut tracer = self.tracer.borrow_mut();
        e!(tracer.mem.write(src, dest as *mut u8));
        Ok(())
    }

    fn step(&self, _signal: Option<u8>) -> Result<Option<u64>> {
        if self.next(Flags::STOP_SINGLESTEP)? == ProcessState::Running {
            let mut tracer = self.tracer.borrow_mut();

            let rip = e!(tracer.regs.get_int()).rip;
            Ok(Some(rip as _))
        } else {
            Ok(None)
        }
    }

    fn cont(&self, _signal: Option<u8>) -> Result<()> {
        self.next(Flags::empty())?;
        Ok(())
    }

    fn path(&self, pid: usize) -> Result<Vec<u8>> {
        let mut path = e!(fs::read(format!("proc:{}/exe", pid)));

        // Strip out "file:" so GDB doesn't interpret it as a URL.
        if path.starts_with(&b"file:"[..]) {
            path.drain(0..5);
        }

        Ok(path)
    }
}
impl Drop for Os {
    fn drop(&mut self) {
        let _ = syscall::kill(self.pid, SIGTERM);
    }
}
