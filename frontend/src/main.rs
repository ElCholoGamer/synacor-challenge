use std::collections::LinkedList;
use clap::Parser;
use colored::Colorize;
use std::fs;
use std::fs::File;
use backend::{Status, SynacorVM, Result, Error, Event};
use backend::util;
use frontend::LimitedQueue;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Binary to execute
    #[clap(short, long)]
    bin: Option<String>,

    /// VM data file
    #[clap(short, long)]
    state: Option<String>,

    /// Disassemble binary to the provided file
    #[clap(short, long)]
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
            backend::disassemble(&bin, &mut file).map_err(|e| Error::IO(e))?;

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
        let initial_output = backend::load_vm_state(&mut vm, &data)?;
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

    let mut input_queue = LinkedList::new();
    let mut current_line = String::new();
    let mut pc_history = LimitedQueue::new(0x1000);
    let mut last_command: Option<String> = None;

    'main: loop {
        pc_history.push(vm.pc());

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
                    if input_queue.len() == 0 {
                        let mut input = String::new();
                        let mut saved = false;

                        loop {
                            input.clear();
                            std::io::stdin().read_line(&mut input).unwrap();
                            input = input.trim().into();

                            if !input.starts_with('!') { break; }

                            if input == "!!" {
                                match std::mem::replace(&mut last_command, None) {
                                    Some(val) => {
                                        println!("{} {}", "Repeating".cyan(), val.cyan());
                                        input = val;
                                    },
                                    None => {
                                        println!("{}", "No command to repeat".red());
                                        continue;
                                    }
                                }
                            }

                            last_command = Some(input.clone());

                            let words: Vec<&str> = input.trim().split_whitespace().collect();

                            match &words[0][1..] {
                                "save" => {
                                    let file = match words.get(1) {
                                        Some(f) => *f,
                                        None => {
                                            println!("{}", "No filename provided".red());
                                            continue;
                                        },
                                    };

                                    *vm.pc_mut() -= 2;
                                    backend::save_vm_state(&vm, file, last_lines.join("\n"))?;
                                    *vm.pc_mut() += 2;
                                    saved = true;
                                    println!("{}", "VM state saved".green());
                                }
                                "debug" => {
                                    let stack = vm.stack();
                                    println!("{} {}", "PC:".yellow(), format!("{:04X}", pc_history.peek_last().unwrap_or(&0)).yellow());
                                    println!("{} {}", "Registers:".yellow(), util::format_hex_slice(vm.registers(), ", ").yellow());
                                    println!("{} {} {}", "Stack:".yellow(), util::format_hex_slice(&stack.contents()[..stack.pointer()], " - ").yellow(), "<--".yellow());
                                }
                                "history" => {
                                    let limit_str = match words.get(1) {
                                        Some(s) => s,
                                        None => {
                                            println!("{}", "No limit provided".red());
                                            continue;
                                        }
                                    };

                                    let limit = std::cmp::min(limit_str.parse().unwrap_or(10), pc_history.len());
                                    let history = pc_history.contents();

                                    println!("{}", "PC history:".yellow());
                                    println!("{}", util::format_hex_slice(&history[history.len() - limit..], ", ").yellow());
                                }
                                "exit" => {
                                    let confirm = match words.get(1) {
                                        Some(val) => *val == "nosave",
                                        None => false,
                                    };

                                    if !confirm && !saved {
                                        println!("{}", "VM state has not been saved!".red());
                                        continue;
                                    }

                                    break 'main;
                                }
                                _ => println!("{}", "Unknown command".red()),
                            }
                        }

                        for char in input.bytes() {
                            input_queue.push_back(char);
                        }

                        input_queue.push_back(b'\n');
                    }

                    vm.write_input(dest, input_queue.pop_front().unwrap_or(b'\n'))?;
                }
            }
        }

        if let Status::Halt = status { break; }
    }
    Ok(())
}
