use std::collections::LinkedList;
use clap::Parser;
use colored::Colorize;
use std::fs;
use synacor_challenge::{Status, SynacorVM, Result, Error, Event};
use synacor_challenge::util;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Binary to execute
    #[clap(short, long)]
    bin: Option<String>,

    /// VM data file
    #[clap(short, long)]
    state: Option<String>,

    /// Debug mode
    #[clap(short, long)]
    debug: bool,
}

fn main() {
    let args: Args = Args::parse();

    run(args).unwrap_or_else(|e| {
        eprintln!("{} {}", "Error:".red().bold(), e.to_string().red());
    });
}

fn run(args: Args) -> Result<()> {
    let mut vm = SynacorVM::new();

    if let Some(filename) = &args.state {
        let buf = fs::read(filename).map_err(|e| Error::IO(e))?;
        let data = util::u8_array_to_u16(&buf);
        synacor_challenge::load_vm_state(&mut vm, &data)?;

        println!("{}", "Save state loaded".cyan());
    } else if let Some(filename) = args.bin {
        let buf = fs::read(&filename).map_err(|e| Error::IO(e))?;
        let bin = util::u8_array_to_u16(&buf);

        vm.load_binary(&bin);
    } else {
        println!("No binary or state file provided. Use --help for more.");
        return Ok(());
    }

    let mut queue = LinkedList::new();
    let state_path = args.state.unwrap_or("state.bin".into());

    loop {
        let status = vm.step()?;

        if let Some(ev) = vm.pull_event() {
            match ev {
                Event::Output(val) => print!("{}", val as char),
                Event::Input(dest) => {
                    if queue.len() == 0 {
                        synacor_challenge::save_vm_state(&vm, &state_path)?;

                        if args.debug {
                            println!("{} {}", "PC: ".cyan(), format!("{:04X}", vm.pc() - 2).cyan());
                        }

                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input).unwrap();

                        for char in input.trim().bytes() {
                            queue.push_back(char);
                        }

                        queue.push_back(b'\n');
                    }

                    vm.write_input(dest, queue.pop_front().unwrap_or(b'\n'))?;
                }
            }
        }

        if let Status::Halt = status { break; }
    }
    Ok(())
}
