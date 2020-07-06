use structopt::StructOpt;
use gdbserver::Opt;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    gdbserver::main(Opt::from_args())
}
