use std::collections::LinkedList;
use clap::Parser;
use colored::Colorize;
use std::fs;
use std::fs::File;
use synacor_challenge::{Status, SynacorVM, Result, Error, Event};
use synacor_challenge::util;
use synacor_challenge::disassembler;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Binary to execute
    #[clap(short, long)]
    bin: Option<String>,

    /// VM data file
    #[clap(long)]
    state: Option<String>,

    /// Debug mode
    #[clap(short, long)]
    debug: bool,

    /// Disassemble binary to the provided file
    #[clap(long)]
    disassemble: Option<String>,
}

fn main() {
    let args: Args = Args::parse();

    run(args).unwrap_or_else(|e| {
        eprintln!("{} {}", "Error:".red().bold(), e.to_string().red());
    });
}

fn run(args: Args) -> Result<()> {
    if let Some(out_path) = args.disassemble {
        if let Some(filename) = &args.bin {
            println!("Disassembling...");

            let buf = fs::read(&filename).map_err(|e| Error::IO(e))?;
            let bin = util::u8_array_to_u16(&buf);

            let mut file = File::create(out_path).map_err(|e| Error::IO(e))?;
            disassembler::disassemble(&bin, &mut file).map_err(|e| Error::IO(e))?;

            println!("Done.");
        } else {
            eprintln!("{}", "No binary file provided".red());
        }

        return Ok(());
    }

    let mut vm = SynacorVM::new();
    let mut last_lines = Vec::<String>::new();

    if let Some(filename) = &args.state {
        let buf = fs::read(filename).map_err(|e| Error::IO(e))?;
        let data = util::u8_array_to_u16(&buf);
        let initial_output = synacor_challenge::load_vm_state(&mut vm, &data)?;
        last_lines = initial_output.split('\n').map(|s| s.into()).collect();

        println!("{}", "Save state loaded".green());
        println!("{}", initial_output.cyan());
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
    let mut current_line = String::new();

    loop {
        if vm.memory()[vm.pc() as usize] == 20 && queue.len() == 0 {
            if args.debug {
                println!("{} {}", "PC: ".yellow(), format!("{:04X}", vm.pc() - 2).cyan());
            }
        }

        let status = vm.step()?;

        if let Some(event) = vm.pull_event() {
            match event {
                Event::Output(val) => {
                    let val = val as char;
                    print!("{}", val);

                    if val == '\n' {
                        last_lines.push(current_line.clone());

                        if last_lines.len() > 20 {
                            last_lines.drain(..1);
                        }
                        current_line.clear();
                    } else {
                        current_line.push(val);
                    }
                }
                Event::Input(dest) => {
                    if queue.len() == 0 {
                        let mut input = String::new();

                        while input.len() == 0 {
                            std::io::stdin().read_line(&mut input).unwrap();

                            if input.trim() == "save" {
                                synacor_challenge::save_vm_state(&vm, &state_path, last_lines.join("\n"))?;
                                input.clear();
                                println!("{}", "VM state saved".green());
                            }
                        }

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
