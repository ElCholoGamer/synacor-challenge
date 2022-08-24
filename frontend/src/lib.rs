use std::collections::VecDeque;
use std::{fs, mem, io, cmp, process};
use std::io::Write;
use backend::{disassembler, Result, SynacorVM, Event};
use backend::vm::STACK_LEN;
use colored::Colorize;

const SAVE_DATA_LEN: usize = 0x800A + STACK_LEN;
const COMMAND_PREFIX: char = ':';

#[derive(Debug)]
pub struct TerminalVM {
    vm: SynacorVM,
    saved: bool,
    input_queue: VecDeque<u8>,
    last_command: Option<Vec<String>>,
    save_state: Option<SynacorVM>,
    pc_history: LimitedQueue<u16>,
    debug: bool,
}

impl TerminalVM {
    pub fn new() -> Self {
        Self {
            vm: SynacorVM::new(),
            saved: true,
            input_queue: VecDeque::new(),
            last_command: None,
            save_state: None,
            pc_history: LimitedQueue::new(0x1000),
            debug: false,
        }
    }

    pub fn load_state_buf(&mut self, buf: &[u8]) -> Result<(), &'static str> {
        deserialize_vm(buf, &mut self.vm)?;
        self.write_input("look");
        Ok(())
    }

    pub fn load_binary(&mut self, bin: &[u16]) {
        self.vm.load_binary(bin);
    }

    pub fn run(&mut self, breakpoints: &[u16], output: &mut Option<impl Write>) -> Result<()> {
        loop {
            self.pc_history.push(self.vm.pc());

            if let Some(out) = output {
                let (assembly, _) = disassembler::to_assembly_instruction(self.vm.pc() as usize, self.vm.memory());
                writeln!(out, "{:04X}    {}", self.vm.pc(), assembly).expect("could not write output");
            }

            if breakpoints.contains(&self.vm.pc()) {
                self.debug = true;
                println!();
                println!("{}", "Breakpoint reached, debug mode enabled.".cyan());
            }

            if self.debug { self.show_debug(); }

            let status = self.vm.step()?;
            match status {
                Some(Event::Halt) => break,
                Some(Event::Output(val)) => print!("{}", val as char),
                Some(Event::Input(dest)) => {
                    while self.input_queue.is_empty() {
                        let mut input = String::new();
                        io::stdin().read_line(&mut input).unwrap();
                        input = input.trim().into();

                        if !input.starts_with(':') {
                            self.write_input(&input);
                            self.saved = false;
                            continue;
                        }

                        let words = split_words(input);
                        self.handle_command(words).unwrap_or_else(print_err);
                    }

                    self.vm.write_input(dest, self.input_queue.pop_front().unwrap())?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn set_debug(&mut self, debug: bool) { self.debug = debug; }

    fn handle_command(&mut self, mut words: Vec<String>) -> Result<(), &str> {
        if words[0] == ":." {
            match mem::replace(&mut self.last_command, None) {
                Some(prev_words) => {
                    println!("{} {}", "Repeating".cyan(), prev_words.join(" ").bold().cyan());
                    words = prev_words;
                }
                None => return Err("no command to repeat"),
            }
        }

        self.last_command = words.clone().into();
        match &words[0][1..] {
            "s" => { // save (file)
                let filename = words.get(1).ok_or("no filename provided")?;

                let buf = serialize_vm(&self.vm);
                fs::write(filename, buf).map_err(|_| "could not write to file")?;
                self.saved = true;
                println!("{}", "VM state saved.".green());
            }
            "l" => { // load (file)
                let filename = words.get(1).ok_or("no filename provided")?;
                let buf = fs::read(&filename).map_err(|_| "could not read file")?;

                self.load_state_buf(&buf)?;
                *self.vm.pc_mut() += 2;
                print!("{}", "Save state loaded".green());
            }
            "qs" => { // quick save
                self.save_state = Some(self.vm.clone());
                println!("{}", "State saved.".green());
            }
            "ql" => { // quick load
                self.vm = self.save_state.clone().ok_or("no save state available")?;
                self.write_input("look");
                print!("{}", "Save state loaded".green());
            }
            "d" => { // debug
                if !self.debug {
                    self.debug = true;
                    println!("{}", "Debug mode enabled.".cyan());
                } else {
                    self.debug = false;
                    println!("{}", "Debug mode disabled.".yellow());
                }
            }
            "h" => { // (pc) history
                let limit_input = words.get(1).ok_or("no limit provided")?.parse().map_err(|_| "invalid limit")?;
                let limit = cmp::min(limit_input, self.pc_history.len());

                let history = self.pc_history.contents();

                println!("{}", "PC history:".yellow());
                println!("{}", format!("{:04X?}", &history[history.len() - limit..]).yellow());
            }
            "q!" => process::exit(0), // quit (no confirm)
            "q" => { // quit (force)
                if !self.saved {
                    return Err("VM state has not been saved!");
                }

                process::exit(0);
            }
            _ => return Err("unknown command"),
        }

        Ok(())
    }

    fn show_debug(&mut self) {
        while self.debug {
            let (assembly, _) = disassembler::to_assembly_instruction(self.vm.pc() as usize, self.vm.memory());
            println!();
            println!("{}", assembly.bold().cyan());
            println!("{} {}", "PC:".yellow().bold(), format!("{:04X}", self.vm.pc()).yellow());
            println!("{} {}", "Registers:".yellow().bold(), format!("{:04X?}", self.vm.registers()).yellow());
            println!("{} {}", "Stack:".yellow().bold(), format!("{:04X?}", self.vm.stack().contents()).yellow());

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            input = input.trim().into();

            if input.is_empty() { return; }

            if !input.starts_with(COMMAND_PREFIX) {
                println!("{}", "Please type a command.".red());
                continue;
            }

            let words = split_words(input);
            self.handle_command(words).unwrap_or_else(print_err);
        }
    }

    fn write_input(&mut self, input: &str) {
        for b in input.bytes() {
            self.input_queue.push_back(b);
        }

        self.input_queue.push_back(b'\n');
    }
}

#[derive(Debug, Clone)]
pub struct LimitedQueue<T> {
    contents: Vec<T>,
}

impl<T> LimitedQueue<T> {
    pub fn new(max_length: usize) -> Self {
        Self {
            contents: Vec::with_capacity(max_length),
        }
    }

    pub fn push(&mut self, val: T) {
        if self.contents.len() == self.contents.capacity() {
            self.contents.drain(..1);
        }

        self.contents.push(val);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.contents.pop()
    }

    pub fn contents(&self) -> &Vec<T> { &self.contents }

    pub fn peek_last(&self) -> Option<&T> { self.contents.last() }

    pub fn len(&self) -> usize { self.contents.len() }
}

pub fn to_u8_vec(src: &[u16]) -> Vec<u8> {
    let mut vec = Vec::with_capacity(src.len() * 2);

    for &val in src {
        vec.push(val as u8);
        vec.push((val >> 8) as u8);
    }

    vec
}

pub fn to_u16_vec(src: &[u8]) -> Vec<u16> {
    let mut vec = Vec::with_capacity(src.len() / 2);

    for chunk in src.chunks(2) {
        let hi = *chunk.get(1).unwrap_or(&0);
        let lo = chunk[0];
        vec.push(((hi as u16) << 8) | (lo as u16))
    }

    vec
}

fn split_words(s: String) -> Vec<String> {
    s.split_whitespace().map(|s| s.into()).collect()
}

fn serialize_vm(vm: &SynacorVM) -> Vec<u8> {
    let mut data = [0; SAVE_DATA_LEN];

    data[0] = vm.pc();
    data[0x1..0x9].clone_from_slice(vm.registers());
    data[0x9..0x8009].clone_from_slice(vm.memory());
    data[0x8009] = vm.stack().pointer() as u16;
    data[0x800A..SAVE_DATA_LEN].clone_from_slice(vm.stack().full_contents());
    to_u8_vec(&data)
}

fn deserialize_vm(buf: &[u8], vm: &mut SynacorVM) -> Result<(), &'static str> {
    if buf.len() < SAVE_DATA_LEN * 2 {
        return Err("Invalid data length");
    }

    let data = to_u16_vec(buf);
    *vm.pc_mut() = data[0];
    vm.registers_mut().clone_from_slice(&data[0x1..0x9]);
    vm.memory_mut().clone_from_slice(&data[0x9..0x8009]);
    *vm.stack_mut().pointer_mut() = data[0x8009] as usize;
    vm.stack_mut().full_contents_mut().clone_from_slice(&data[0x800A..SAVE_DATA_LEN]);
    Ok(())
}

fn print_err(e: &str) {
    println!("{} {}", "Error:".bold().red(), e.red());
}