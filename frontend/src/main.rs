use std::error::Error;
use clap::Parser;
use colored::Colorize;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use frontend::TerminalVM;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Binary to execute
    filename: PathBuf,

    /// Load the provided file as VM state
    #[clap(short, long)]
    load_state: bool,

    /// Start in debug mode
    #[clap(short, long)]
    debug: bool,

    /// Debug breakpoints
    #[clap(short, long)]
    breakpoints: Vec<String>,

    /// Disassemble binary to the provided file
    #[clap(long)]
    disassemble: bool,

    /// Output file for disassembly
    #[clap(short, long)]
    output: Option<PathBuf>,
}

fn main() {
    let args: Args = Args::parse();

    run(args).unwrap_or_else(|e| {
        eprintln!("{} {}", "Error:".red().bold(), e.to_string().red());
    });
}

fn run(args: Args) -> Result<(), Box<dyn Error>> {
    let buf = fs::read(&args.filename)?;
    let bin = frontend::to_u16_vec(&buf);

    if args.disassemble {
        println!("Disassembling...");

        let path = args.output.unwrap_or_else(|| {
            let mut p = args.filename;
            p.set_extension("s");
            p
        });
        let mut file = File::create(&path)?;
        backend::disassemble(&bin, &mut file)?;

        println!("Done.");
        return Ok(());
    }

    let mut vm = TerminalVM::new();

    if args.load_state {
        vm.load_state_buf(&buf)?;
        print!("{}", "VM state loaded".green());
    } else {
        vm.load_binary(&bin);
    }

    let breakpoints = args.breakpoints.iter()
        .map(|s| u16::from_str_radix(s, 16))
        .collect::<Result<Vec<_>, _>>()?;

    let mut output_file = args.output.map(|path| File::create(path).unwrap());

    vm.set_debug(args.debug);
    vm.run(&breakpoints, &mut output_file)?;
    Ok(())
}
