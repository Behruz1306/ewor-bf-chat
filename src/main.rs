use bf_chat::bf;
use clap::Parser;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "bf-run", about = "brainfuck interpreter with optional BFA syscalls")]
struct Args {
    /// enable BFA mode (. triggers syscalls, cells 0-7 = call frame)
    #[arg(long)]
    bfa: bool,

    program: PathBuf,
}

fn main() {
    let args = Args::parse();
    let src = fs::read_to_string(&args.program).unwrap_or_else(|e| {
        eprintln!("could not read {}: {e}", args.program.display());
        std::process::exit(1);
    });

    if let Err(e) = bf::run(&src, args.bfa) {
        eprintln!("runtime error: {e:?}");
        std::process::exit(1);
    }
}
