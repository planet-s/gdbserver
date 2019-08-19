use gdb_protocol::{packet::{CheckedPacket, Kind}, io::GdbServer};
use structopt::StructOpt;

use std::io::prelude::*;

mod os;

use os::{Os, Registers, Status, Target};

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

// fn get_hex_int(buf: &[u8]) -> Result<(&[u8], usize)> {
//     let nondigit = buf.iter().position(|b| !(b as char).is_digit(16));
//     let (digits, trail) = buf.split_at(nondigit);
//     let string = std::str::from_utf8(&digits)?;
//     Ok((&trail, string.parse()?))
// }

fn encode_status(tracee: &mut Os) -> CheckedPacket {
    let mut bytes = Vec::new();
    match tracee.status() {
        Status::Exited(status) => write!(bytes, "W{:02X}", status).unwrap(),
        Status::Signaled(status) => write!(bytes, "X{:02X}", status).unwrap(),
        Status::Stopped(status) => write!(bytes, "T{:02X}", status).unwrap(),
    }
    CheckedPacket::from_data(Kind::Packet, bytes)
}

fn main() -> Result<()> {
    let mut opt = Opt::from_args();
    let mut stream = GdbServer::listen(opt.addr)?;

    opt.args.insert(0, opt.program.clone());
    let mut tracee = Os::new(opt.program, opt.args)?;

    while let Some(packet) = stream.next_packet()? {
        println!(
            "-> {:?} {:?}",
            packet.kind,
            std::str::from_utf8(&packet.data)
        );

        stream.dispatch(&match packet.data.first() {
            // Maybe extended mode isn't that good, as it'll allow GDB to read the whole filesystem
            // Some(b'!') => {
            //     CheckedPacket::from_data(Kind::Packet, b"OK".to_vec())
            // },
            Some(b'?') => encode_status(&mut tracee),
            Some(b'g') => {
                let mut out = Vec::new();
                match tracee.getregs() {
                    Ok(regs) => regs.encode(&mut out)?,
                    Err(errno) => write!(out, "E{:02X}", errno).unwrap()
                }
                CheckedPacket::from_data(Kind::Packet, out)
            },
            Some(b'G') => {
                let mut out = Vec::new();
                let regs = Registers::decode(&packet.data[1..])?;
                match tracee.setregs(&regs) {
                    Ok(()) => write!(out, "OK").unwrap(),
                    Err(errno) => write!(out, "E{:02X}", errno).unwrap(),
                }
                CheckedPacket::from_data(Kind::Packet, out)
            },
            Some(b'm') => {
                let data = &packet.data[1..];
                let sep = memchr::memchr(b',', &data).ok_or("gdb didn't send a memory length")?;

                let (addr, len) = data.split_at(sep);
                let addr = usize::from_str_radix(std::str::from_utf8(&addr)?, 16)?;
                let len = usize::from_str_radix(std::str::from_utf8(&len[1..])?, 16)?;

                let mut out = Vec::new();

                let mut bytes = vec![0; len];
                match tracee.getmem(addr, &mut bytes) {
                    Ok(read) => for byte in &bytes[..read] {
                        write!(out, "{:02X}", byte).unwrap();
                    },
                    Err(errno) => write!(out, "E{:02X}", errno).unwrap(),
                }
                CheckedPacket::from_data(Kind::Packet, out)
            },
            Some(b'M') => {
                let data = &packet.data[1..];
                let sep1 = memchr::memchr(b',', &data).ok_or("gdb didn't send a memory length")?;

                let (addr, rest) = data.split_at(sep1);
                let sep2 = memchr::memchr(b':', &rest).ok_or("gdb didn't send memory content")?;
                let (_, content) = rest.split_at(sep2);

                let addr = usize::from_str_radix(std::str::from_utf8(&addr)?, 16)?;
                // let len = usize::from_str_radix(std::str::from_utf8(&len[1..])?, 16)?;

                let mut bytes = Vec::new();
                for byte in content[1..].chunks(2) {
                    bytes.push(u8::from_str_radix(std::str::from_utf8(&byte)?, 16)?);
                }

                let mut out = Vec::new();
                match tracee.setmem(&bytes, addr) {
                    Ok(()) => write!(out, "OK").unwrap(),
                    Err(errno) => write!(out, "E{:02X}", errno).unwrap(),
                }
                CheckedPacket::from_data(Kind::Packet, out)
            },
            Some(b'X') => {
                let data = &packet.data[1..];
                let sep1 = memchr::memchr(b',', &data).ok_or("gdb didn't send a memory length")?;

                let (addr, rest) = data.split_at(sep1);
                let sep2 = memchr::memchr(b':', &rest).ok_or("gdb didn't send memory content")?;
                let (_, content) = rest.split_at(sep2);

                let addr = usize::from_str_radix(std::str::from_utf8(&addr)?, 16)?;
                // let len = usize::from_str_radix(std::str::from_utf8(&len[1..])?, 16)?;

                let mut out = Vec::new();
                match tracee.setmem(&content[1..], addr) {
                    Ok(()) => write!(out, "OK").unwrap(),
                    Err(errno) => write!(out, "E{:02X}", errno).unwrap(),
                }
                CheckedPacket::from_data(Kind::Packet, out)
            },
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
                        Some(b'?') => CheckedPacket::from_data(Kind::Packet, b"vCont;s;c;S;C;r".to_vec()),
                        Some(b';') => match data.get(1) {
                            Some(b's') => {
                                tracee.step(None)?;
                                encode_status(&mut tracee)
                            },
                            Some(b'S') => {
                                let slice = data.get(2..4).ok_or("gdb didn't send a signal")?;
                                let signal = u8::from_str_radix(std::str::from_utf8(&slice)?, 16)?;
                                tracee.step(Some(signal))?;
                                encode_status(&mut tracee)
                            },
                            Some(b'c') => {
                                tracee.cont(None)?;
                                encode_status(&mut tracee)
                            },
                            Some(b'C') => {
                                let slice = data.get(2..4).ok_or("gdb didn't send a signal")?;
                                let signal = u8::from_str_radix(std::str::from_utf8(&slice)?, 16)?;
                                tracee.cont(Some(signal))?;
                                encode_status(&mut tracee)
                            },
                            Some(b'r') => {
                                let data = &data[2..];

                                let sep1 = memchr::memchr(b',', data).ok_or("gdb didn't send an end value")?;
                                let (start, end) = data.split_at(sep1);
                                let sep2 = memchr::memchr(b':', end).unwrap_or_else(|| end.len());

                                let start = u64::from_str_radix(std::str::from_utf8(start)?, 16)?;
                                let end = u64::from_str_radix(std::str::from_utf8(&end[1..sep2])?, 16)?;

                                tracee.resume(start..end)?;
                                encode_status(&mut tracee)
                            },
                            _ => CheckedPacket::empty()
                        },
                        _ => CheckedPacket::empty()
                    }
                } else if packet.data[1..].starts_with(b"Kill") {
                    break;
                } else {
                    CheckedPacket::empty()
                }
            },
            _ => CheckedPacket::empty(),
        })?;
    }

    Ok(())
}
