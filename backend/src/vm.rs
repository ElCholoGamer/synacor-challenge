use crate::{Result, Error, Stack};

pub const STACK_LEN: usize = 0x1000;

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Continue,
    Halt,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Output(u8),
    Input(u16),
}

#[derive(Debug, Clone)]
pub struct SynacorVM {
    memory: [u16; 0x8000],
    registers: [u16; 8],
    stack: Stack<u16, STACK_LEN>,
    pc: u16,
    event: Option<Event>
}

impl SynacorVM {
    pub fn new() -> Self {
        Self {
            memory: [0; 0x8000],
            registers: [0; 8],
            stack: Stack::new(),
            pc: 0,
            event: None,
        }
    }

    pub fn load_binary(&mut self, bin: &[u16]) {
        for (i, &val) in bin.iter().enumerate() {
            self.memory[i] = val;
        }
    }

    pub fn step(&mut self) -> Result<Status> {
        let opcode = self.read_pc();

        match opcode {
            0 => return Ok(Status::Halt), // halt
            1 => { // set
                let reg = self.read_pc();
                let val = self.read_param_value()?;
                self.write_register(reg, val)?;
            }
            2 => { // push
                let val = self.read_param_value()?;
                self.stack.push(val)?;
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
                self.stack.push(self.pc)?;
                self.pc = addr;
            }
            18 => self.pc = self.stack.pop()?, // ret
            19 => { // out
                let val = self.read_param_value()?;
                self.event = Some(Event::Output(val as u8));
            }
            20 => { // in
                let reg = self.read_pc();
                self.event = Some(Event::Input(reg));
            }
            21 => {} // noop
            _ => return Err(Error::IllegalOpcode(opcode))
        }
        Ok(Status::Continue)
    }

    pub fn write_input(&mut self, dest: u16, val: u8) -> Result<()> {
        self.write_register(dest, val as u16)
    }

    pub fn pull_event(&mut self) -> Option<Event> {
        std::mem::replace(&mut self.event, None)
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
            Ok(self.registers[(val & 0x7FFF) as usize])
        } else {
            Err(Error::IllegalParameterRead(val))
        }
    }

    fn write_register(&mut self, dest: u16, val: u16) -> Result<()> {
        if dest < 0x8000 || dest >= 0x8008 {
            Err(Error::IllegalParameterWrite(val))
        } else {
            self.registers[(dest & 0x7FFF) as usize] = val;
            Ok(())
        }
    }

    pub fn pc(&self) -> u16 { self.pc }

    pub fn stack(&self) -> &Stack<u16, STACK_LEN> { &self.stack }

    pub fn memory(&self) -> &[u16; 0x8000] { &self.memory }

    pub fn registers(&self) -> &[u16; 8] { &self.registers }

    pub fn pc_mut(&mut self) -> &mut u16 { &mut self.pc }

    pub fn stack_mut(&mut self) -> &mut Stack<u16, STACK_LEN> { &mut self.stack }

    pub fn memory_mut(&mut self) -> &mut [u16; 0x8000] { &mut self.memory }

    pub fn registers_mut(&mut self) -> &mut [u16; 8] { &mut self.registers }
}