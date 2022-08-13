use std::collections::LinkedList;
use std::io::Read;

pub mod error;

pub use error::{Error,Result};

#[derive(Debug, Clone)]
pub struct Stack<T> {
    values: LinkedList<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Self {
            values: LinkedList::new(),
        }
    }

    pub fn push(&mut self, val: T) {
        self.values.push_back(val);
    }

    pub fn pop(&mut self) -> Result<T> {
        self.values.pop_back().ok_or(Error::StackUnderflow)
    }
}

#[derive(Debug, Clone)]
pub struct Input {

}

#[derive(Debug, Clone)]
pub struct SynacorVM {
    memory: [u16; 0x8000],
    registers: [u16; 8],
    stack: Stack<u16>,
    pc: u16,
    input_queue: LinkedList<char>,
}

impl SynacorVM {
    pub fn new() -> Self {
        Self {
            memory: [0; 0x8000],
            registers: [0; 8],
            stack: Stack::new(),
            pc: 0,
            input_queue: LinkedList::new(),
        }
    }

    pub fn load_binary(&mut self, bin: Vec<u8>) {
        for (i, chunk) in bin.chunks(2).enumerate() {
            let lo = chunk[0];
            let hi = *chunk.get(1).unwrap_or(&0);
            self.memory[i] = ((hi as u16) << 8) | (lo as u16);
        }
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            let opcode = self.read_pc();
            match opcode {
                0 => break, // halt
                1 => { // set
                    let reg = self.read_pc();
                    let val = self.read_param_value()?;
                    self.write_register(reg, val)?;
                }
                2 => { // push
                    let val = self.read_param_value()?;
                    self.stack.push(val);
                }
                3 => { // pop
                    let reg = self.read_pc();
                    let val = self.stack.pop()?;
                    self.write_register(reg, val)?;
                }
                4 => { // eq
                    let reg = self.read_pc();
                    let a = self.read_param_value()?;
                    let b = self.read_param_value()?;
                    self.write_register(reg, if a == b { 1 } else { 0 })?;
                }
                5 => { // gt
                    let reg = self.read_pc();
                    let a = self.read_param_value()?;
                    let b = self.read_param_value()?;
                    self.write_register(reg, if a > b { 1 } else { 0 })?;
                }
                6 => self.pc = self.read_param_value()?, // jmp
                7 => { // jt
                    let val = self.read_param_value()?;
                    let addr = self.read_param_value()?;
                    if val != 0 {
                        self.pc = addr;
                    }
                }
                8 => { // jf
                    let val = self.read_param_value()?;
                    let addr = self.read_param_value()?;
                    if val == 0 {
                        self.pc = addr;
                    }
                }
                9 => { // add
                    let reg = self.read_pc();
                    let a = self.read_param_value()?;
                    let b = self.read_param_value()?;
                    self.write_register(reg, (a + b) % 0x8000)?;
                }
                10 => { // mult
                    let reg = self.read_pc();
                    let a = self.read_param_value()?;
                    let b = self.read_param_value()?;
                    self.write_register(reg, a.wrapping_mul(b) % 0x8000)?;
                }
                11 => { // mod
                    let reg = self.read_pc();
                    let a = self.read_param_value()?;
                    let b = self.read_param_value()?;
                    self.write_register(reg, a % b)?;
                }
                12 => { // and
                    let reg = self.read_pc();
                    let a = self.read_param_value()?;
                    let b = self.read_param_value()?;
                    self.write_register(reg, a & b)?;
                }
                13 => { // or
                    let reg = self.read_pc();
                    let a = self.read_param_value()?;
                    let b = self.read_param_value()?;
                    self.write_register(reg, a | b)?;
                }
                14 => { // not
                    let reg = self.read_pc();
                    let val = self.read_param_value()?;
                    self.write_register(reg, !val & 0x7FFF)?;
                }
                15 => { // rmem
                    let reg = self.read_pc();
                    let addr = self.read_param_value()?;
                    self.write_register(reg, self.memory[addr as usize])?;
                }
                16 => { // wmem
                    let addr = self.read_param_value()?;
                    let val = self.read_param_value()?;
                    self.memory[addr as usize] = val;
                }
                17 => { // call
                    let addr = self.read_param_value()?;
                    self.stack.push(self.pc);
                    self.pc = addr;
                }
                18 => self.pc = self.stack.pop()?, // ret
                19 => { // out
                    let val = self.read_param_value()?;
                    print!("{}", val as u8 as char);
                }
                21 => {} // noop
                _ => return Err(Error::IllegalOpcode(opcode))
            }
        }
        Ok(())
    }

    fn read_pc(&mut self) -> u16 {
        let val = self.memory[self.pc as usize];
        self.pc = (self.pc + 1) & 0x7FFF;
        val
    }

    fn read_param_value(&mut self) -> Result<u16> {
        let val = self.read_pc();
        if val < 0x8000 {
            Ok(val)
        } else if val < 0x8008 {
            Ok(self.registers[(val - 0x8000) as usize])
        } else {
            Err(Error::IllegalParameterRead(val))
        }
    }

    fn write_register(&mut self, dest: u16, val: u16) -> Result<()> {
        if dest < 0x8000 || dest >= 0x8008 {
            Err(Error::IllegalParameterWrite(val))
        } else {
            self.registers[(dest - 0x8000) as usize] = val;
            Ok(())
        }
    }
}