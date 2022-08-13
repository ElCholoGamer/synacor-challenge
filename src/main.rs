use clap::Parser;
use std::fs;
use synacor_challenge::SynacorVM;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Binary to execute
    filename: String,
}

fn main() {
    let args = Args::parse();

    let bin = fs::read(&args.filename).expect("Unable to read file");

    let mut vm = SynacorVM::new();
    vm.load_binary(bin);

    vm.run().unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
    });
}
