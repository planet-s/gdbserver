use structopt::StructOpt;
use std::{
    path::Path,
    process::Command,
};

#[derive(Debug, StructOpt)]
struct Opt {
    /// The program that should be debugged
    program: String,
    /// The arguments of the program
    args: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();

    // We're not using randomness simply to avoid the dependency lol. This hacky
    // solution is for sure not race-condition proof, and trust me, I know. In
    // the future, gdb might be able to communicate using gdbserver using stdio.
    // Or maybe, the "chan:" scheme can generate a random name for us that
    // avoids all the overhead of syscalls. It doesn't really matter: Just know
    // that this is temporary.

    let mut counter = 0;
    let mut path;
    loop {
        path = format!("chan:gdb-redox-{}", counter);
        counter += 1;

        if !Path::new(&path).exists() {
            break;
        }
    }

    let child_result = Command::new("gdb")
        .args(&["-ex", &format!("target remote {}", path)])
        .spawn();

    let mut child = match child_result {
        Ok(child) => child,
        Err(err) => {
            eprintln!("Failed to launch gdb, is it installed?");
            return Err(err.into());
        }
    };

    let res = gdbserver::main(gdbserver::Opt {
        addr: path,
        kind: String::from("unix"),

        program: opt.program,
        args: opt.args,
    });

    let _ = child.wait();

    res
}
