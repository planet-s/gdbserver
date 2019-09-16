use super::Registers;
use crate::Result;

use std::{
    cell::Cell,
    ffi::CString,
    io, iter,
    mem::{self, MaybeUninit},
    ptr,
};

use gdb_remote_protocol::{Error, StopReason};

pub struct Os {
    pid: libc::pid_t,
    last_status: Cell<libc::c_int>,
}

trait FromOsError: Sized {
    fn from_os_error(errno: libc::c_int) -> Self;
}

impl FromOsError for Error {
    fn from_os_error(errno: libc::c_int) -> Self {
        Error::Error(errno as u8)
    }
}
impl FromOsError for io::Error {
    fn from_os_error(errno: libc::c_int) -> Self {
        io::Error::from_raw_os_error(errno as i32)
    }
}
impl FromOsError for Box<dyn std::error::Error> {
    fn from_os_error(errno: libc::c_int) -> Self {
        Box::new(io::Error::from_os_error(errno))
    }
}

macro_rules! e {
    ($result:expr) => {{
        let result = $result;
        if result == -1 {
            return Err(FromOsError::from_os_error(*libc::__errno_location()));
        }
        result
    }};
}

fn getmem<G, E>(mut src: usize, dest: &mut [u8], mut get: G) -> Result<usize, E>
where
    G: FnMut(usize) -> Result<usize, E>,
{
    for chunk in dest.chunks_mut(mem::size_of::<usize>()) {
        let bytes = get(src)?.to_ne_bytes();
        chunk.copy_from_slice(&bytes[..chunk.len()]);

        src += mem::size_of::<usize>();
    }
    Ok(dest.len())
}
fn setmem<G, S, E>(src: &[u8], mut dest: usize, mut get: G, mut set: S) -> Result<(), E>
where
    G: FnMut(usize) -> Result<usize, E>,
    S: FnMut(usize, usize) -> Result<(), E>,
{
    let mut iter = src.chunks_exact(mem::size_of::<usize>());

    for chunk in iter.by_ref() {
        let mut bytes = [0; mem::size_of::<usize>()];
        bytes.copy_from_slice(chunk);
        let word = usize::from_ne_bytes(bytes);
        set(dest, word)?;

        dest += mem::size_of::<usize>();
    }

    let mut bytes = get(dest)?.to_ne_bytes();
    let rest = iter.remainder();
    bytes[..rest.len()].copy_from_slice(rest);
    let word = usize::from_ne_bytes(bytes);
    set(dest, word)?;

    Ok(())
}

impl super::Target for Os {
    fn new(program: String, args: Vec<String>) -> Result<Self, Box<dyn std::error::Error>> {
        let program = CString::new(program)?.into_raw();
        let args = args
            .into_iter()
            .map(|s| CString::new(s).map(|s| s.into_raw() as *const _))
            .chain(iter::once(Ok(ptr::null())))
            .collect::<Result<Vec<*const libc::c_char>, _>>()?;

        unsafe {
            let pid = libc::fork();

            if pid == 0 {
                // Must not drop any memory, not unwind (panic).
                if libc::ptrace(libc::PTRACE_TRACEME) < 0 {
                    eprintln!(
                        "ptrace(PTRACE_TRACEME) failed: {:?}",
                        io::Error::last_os_error()
                    );
                    libc::exit(1);
                }
                if libc::raise(libc::SIGSTOP) < 0 {
                    eprintln!("raise(SIGSTOP) failed: {:?}", io::Error::last_os_error());
                    libc::exit(1);
                }
                if libc::execvp(program, args.as_ptr()) < 0 {
                    eprintln!("execv(...) failed: {:?}", io::Error::last_os_error());
                    libc::exit(1);
                }
                eprintln!("execv(...) should not be able to succeed");
                libc::exit(1);
            } else {
                // Drop variables only the child needed
                CString::from_raw(program);
                for arg in args {
                    if !arg.is_null() {
                        // We originally get mutable memory, don't worry!
                        CString::from_raw(arg as *mut _);
                    }
                }

                // Wait for tracee to stop
                let mut status = 0;
                e!(libc::waitpid(pid, &mut status, 0));

                // Skip until post-execve
                e!(libc::ptrace(libc::PTRACE_CONT, pid, 0, 0));
                e!(libc::waitpid(pid, &mut status, 0));

                Ok(Os {
                    pid,
                    last_status: Cell::from(status),
                })
            }
        }
    }

    fn status(&self) -> StopReason {
        unsafe {
            if libc::WIFEXITED(self.last_status.get()) {
                StopReason::Exited(
                    self.pid as _,
                    libc::WEXITSTATUS(self.last_status.get()) as _,
                )
            } else if libc::WIFSIGNALED(self.last_status.get()) {
                StopReason::ExitedWithSignal(
                    self.pid as _,
                    libc::WTERMSIG(self.last_status.get()) as _,
                )
            } else if libc::WIFSTOPPED(self.last_status.get()) {
                StopReason::Signal(libc::WSTOPSIG(self.last_status.get()) as _)
            } else {
                unimplemented!("TODO: Implement status {}", self.last_status.get());
            }
        }
    }

    fn pid(&self) -> u32 {
        return self.pid as _;
    }

    fn getregs(&self) -> Result<Registers> {
        let int = unsafe {
            let mut int: MaybeUninit<libc::user_regs_struct> = MaybeUninit::uninit();
            e!(libc::ptrace(
                libc::PTRACE_GETREGS,
                self.pid,
                0,
                int.as_mut_ptr()
            ));
            int.assume_init()
        };

        let float = unsafe {
            let mut float: MaybeUninit<libc::user_fpregs_struct> = MaybeUninit::uninit();
            e!(libc::ptrace(
                libc::PTRACE_GETFPREGS,
                self.pid,
                0,
                float.as_mut_ptr()
            ));
            float.assume_init()
        };

        let mut registers = Registers::default();
        registers.r15 = Some(int.r15);
        registers.r14 = Some(int.r14);
        registers.r13 = Some(int.r13);
        registers.r12 = Some(int.r12);
        registers.rbp = Some(int.rbp);
        registers.rbx = Some(int.rbx);
        registers.r11 = Some(int.r11);
        registers.r10 = Some(int.r10);
        registers.r9 = Some(int.r9);
        registers.r8 = Some(int.r8);
        registers.rax = Some(int.rax);
        registers.rcx = Some(int.rcx);
        registers.rdx = Some(int.rdx);
        registers.rsi = Some(int.rsi);
        registers.rdi = Some(int.rdi);
        registers.rip = Some(int.rip);
        registers.cs = Some(int.cs as _);
        registers.eflags = Some(int.eflags as _);
        registers.rsp = Some(int.rsp);
        registers.ss = Some(int.ss as _);
        registers.ds = Some(int.ds as _);
        registers.es = Some(int.es as _);
        registers.fs = Some(int.fs as _);
        registers.gs = Some(int.gs as _);

        registers.fctrl = Some(float.cwd as _);
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
        registers.fs_base = Some(int.fs_base as _);
        registers.gs_base = Some(int.gs_base as _);
        registers.orig_rax = Some(int.orig_rax as _);

        Ok(registers)
    }

    fn setregs(&self, registers: &Registers) -> Result<()> {
        let mut int: libc::user_regs_struct = unsafe { MaybeUninit::zeroed().assume_init() };
        let mut float: libc::user_fpregs_struct = unsafe { MaybeUninit::zeroed().assume_init() };

        int.r15 = registers.r15.unwrap_or(int.r15);
        int.r14 = registers.r14.unwrap_or(int.r14);
        int.r13 = registers.r13.unwrap_or(int.r13);
        int.r12 = registers.r12.unwrap_or(int.r12);
        int.rbp = registers.rbp.unwrap_or(int.rbp);
        int.rbx = registers.rbx.unwrap_or(int.rbx);
        int.r11 = registers.r11.unwrap_or(int.r11);
        int.r10 = registers.r10.unwrap_or(int.r10);
        int.r9 = registers.r9.unwrap_or(int.r9);
        int.r8 = registers.r8.unwrap_or(int.r8);
        int.rax = registers.rax.unwrap_or(int.rax);
        int.rcx = registers.rcx.unwrap_or(int.rcx);
        int.rdx = registers.rdx.unwrap_or(int.rdx);
        int.rsi = registers.rsi.unwrap_or(int.rsi);
        int.rdi = registers.rdi.unwrap_or(int.rdi);
        int.rip = registers.rip.unwrap_or(int.rip);
        int.cs = registers.cs.map(|r| r as _).unwrap_or(int.cs);
        int.eflags = registers.eflags.map(|r| r as _).unwrap_or(int.eflags);
        int.rsp = registers.rsp.unwrap_or(int.rsp);
        int.ss = registers.ss.map(|r| r as _).unwrap_or(int.ss);
        int.ds = registers.ds.map(|r| r as _).unwrap_or(int.ds);
        int.es = registers.es.map(|r| r as _).unwrap_or(int.es);
        int.fs = registers.fs.map(|r| r as _).unwrap_or(int.fs);
        int.gs = registers.gs.map(|r| r as _).unwrap_or(int.gs);

        float.cwd = registers.fctrl.map(|r| r as _).unwrap_or(float.cwd);
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
        int.fs_base = registers.fs_base.map(|r| r as _).unwrap_or(int.fs_base);
        int.gs_base = registers.gs_base.map(|r| r as _).unwrap_or(int.gs_base);
        int.orig_rax = registers.orig_rax.map(|r| r as _).unwrap_or(int.orig_rax);

        unsafe {
            e!(libc::ptrace(libc::PTRACE_SETREGS, self.pid, 0, &int));
            e!(libc::ptrace(libc::PTRACE_SETFPREGS, self.pid, 0, &float));
        }

        Ok(())
    }

    fn getmem(&self, src: usize, dest: &mut [u8]) -> Result<usize> {
        // TODO: Don't report errors when able to read part of requested?
        // Also implement this in the Redox kernel perhaps
        getmem(src, dest, |addr| unsafe {
            Ok(e!(libc::ptrace(libc::PTRACE_PEEKDATA, self.pid, addr)) as usize)
        })
    }

    fn setmem(&self, src: &[u8], dest: usize) -> Result<()> {
        setmem(
            src,
            dest,
            |addr| unsafe { Ok(e!(libc::ptrace(libc::PTRACE_PEEKDATA, self.pid, addr)) as usize) },
            |addr, word| unsafe {
                e!(libc::ptrace(libc::PTRACE_POKEDATA, self.pid, addr, word));
                Ok(())
            },
        )
    }

    fn step(&self, signal: Option<u8>) -> Result<Option<u64>> {
        unsafe {
            e!(libc::ptrace(
                libc::PTRACE_SINGLESTEP,
                self.pid,
                0,
                signal.unwrap_or(0) as libc::c_uint
            ));
            let mut status = 0;
            e!(libc::waitpid(self.pid, &mut status, 0));
            self.last_status.set(status);

            Ok(
                if libc::WIFSTOPPED(self.last_status.get())
                    && libc::WSTOPSIG(self.last_status.get()) == libc::SIGTRAP
                {
                    let rip = e!(libc::ptrace(
                        libc::PTRACE_PEEKUSER,
                        self.pid,
                        libc::RIP as usize * mem::size_of::<usize>()
                    ));
                    Some(rip as u64)
                } else {
                    None
                },
            )
        }
    }

    fn cont(&self, signal: Option<u8>) -> Result<()> {
        unsafe {
            e!(libc::ptrace(
                libc::PTRACE_CONT,
                self.pid,
                0,
                signal.unwrap_or(0) as libc::c_uint
            ));
            let mut status = 0;
            e!(libc::waitpid(self.pid, &mut status, 0));
            self.last_status.set(status);
        }
        Ok(())
    }
}
impl Drop for Os {
    fn drop(&mut self) {
        unsafe {
            libc::kill(self.pid, libc::SIGTERM);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, mem};

    #[test]
    fn getmem() {
        const SOURCE: &[u8] = b"testing one two three";
        let mut dest = [0; 9];
        super::getmem(3, &mut dest, |addr| -> Result<usize, ()> {
            let mut bytes = [0; mem::size_of::<usize>()];
            bytes.copy_from_slice(&SOURCE[addr..addr + mem::size_of::<usize>()]);
            Ok(usize::from_ne_bytes(bytes))
        })
        .unwrap();
        assert_eq!(&dest, b"ting one ");
    }
    #[test]
    fn setmem() {
        let source = Cell::new(*b"testing one two three");
        let dest = b"XXXXXXXXX";
        super::setmem(
            dest,
            3,
            |addr| -> Result<usize, ()> {
                let mut bytes = [0; mem::size_of::<usize>()];
                bytes.copy_from_slice(&source.get()[addr..addr + mem::size_of::<usize>()]);
                Ok(usize::from_ne_bytes(bytes))
            },
            |addr, word| -> Result<(), ()> {
                let mut slice = source.get();
                slice[addr..addr + mem::size_of::<usize>()].copy_from_slice(&word.to_ne_bytes());
                source.set(slice);
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(&source.get(), b"tesXXXXXXXXXtwo three");
    }
}
