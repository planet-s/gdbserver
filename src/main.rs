use gdb_remote_protocol::{
    Error,
    Handler,
    MemoryRegion,
    ProcessType,
    StopReason,
};
use structopt::StructOpt;

use std::{
    net::{TcpListener, TcpStream},
    io::prelude::*,
};

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

pub type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

pub struct App {
    tracee: Os,
}
impl Handler for App {
    fn attached(&self, _pid: Option<u64>) -> Result<ProcessType, Error> {
        Ok(ProcessType::Created)
    }
    fn halt_reason(&self) -> Result<StopReason, Error> {
        Ok(self.tracee.status())
    }
    fn read_general_registers(&self) -> Result<Vec<u8>, Error> {
        let regs = self.tracee.getregs().map_err(|e| Error::Error(e as _))?;

        let mut bytes = Vec::new();
        regs.encode(&mut bytes);

        Ok(bytes)
    }
    fn write_general_registers(&self, content: &[u8]) -> Result<(), Error> {
        let regs = Registers::decode(content);
        self.tracee.setregs(&regs).map_err(|e| Error::Error(e as _))?;
        Ok(())
    }
    fn read_memory(&self, region: MemoryRegion) -> Result<Vec<u8>, Error> {
        let mut buf = vec![0; region.length as usize];
        self.tracee.getmem(region.address as usize, &mut buf).map_err(|e| Error::Error(e as _))?;
        Ok(buf)
    }
    fn write_memory(&self, address: u64, bytes: &[u8]) -> Result<(), Error> {
        self.tracee.setmem(bytes, address as usize).map_err(|e| Error::Error(e as _))?;
        Ok(())
    }
}

fn main() -> Result<()> {
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
    });

    /*
        stream.dispatch(&match packet.data.first() {
            Some(b'v') => {
                // if packet.data[1..].starts_with(b"Run") {
                //     let mut cursor = &packet.data[1..];
                //     loop {
                //         let arg_end = memchr::memchr(b';', &cursor);
                //         println!("Arg: {:?}", std::str::from_utf8(&cursor[..arg_end.unwrap_or_else(|| cursor.len())]));
                //         match arg_end {
                //             Some(arg_end) => cursor = &cursor[arg_end+1..],
                //             None => break,
                //         }
                //     }
                //     CheckedPacket::from_data(Kind::Packet, b"W00".to_vec())
                if packet.data[1..].starts_with(b"Cont") {
                    let data = &packet.data[1 + 4..];
                    match data.first() {
                        Some(b'?') => {
                            CheckedPacket::from_data(Kind::Packet, b"vCont;s;c;S;C;r".to_vec())
                        }
                        Some(b';') => match data.get(1) {
                            Some(b's') => {
                                tracee.step(None)?;
                                encode_status(&mut tracee)
                            }
                            Some(b'S') => {
                                let slice = data.get(2..4).ok_or("gdb didn't send a signal")?;
                                let signal = u8::from_str_radix(std::str::from_utf8(&slice)?, 16)?;
                                tracee.step(Some(signal))?;
                                encode_status(&mut tracee)
                            }
                            Some(b'c') => {
                                tracee.cont(None)?;
                                encode_status(&mut tracee)
                            }
                            Some(b'C') => {
                                let slice = data.get(2..4).ok_or("gdb didn't send a signal")?;
                                let signal = u8::from_str_radix(std::str::from_utf8(&slice)?, 16)?;
                                tracee.cont(Some(signal))?;
                                encode_status(&mut tracee)
                            }
                            Some(b'r') => {
                                let data = &data[2..];

                                let sep1 = memchr::memchr(b',', data)
                                    .ok_or("gdb didn't send an end value")?;
                                let (start, end) = data.split_at(sep1);
                                let sep2 = memchr::memchr(b':', end).unwrap_or_else(|| end.len());

                                let start = u64::from_str_radix(std::str::from_utf8(start)?, 16)?;
                                let end =
                                    u64::from_str_radix(std::str::from_utf8(&end[1..sep2])?, 16)?;

                                tracee.resume(start..end)?;
                                encode_status(&mut tracee)
                            }
                            _ => CheckedPacket::empty(),
                        },
                        _ => CheckedPacket::empty(),
                    }
    }
    */

    Ok(())
}
