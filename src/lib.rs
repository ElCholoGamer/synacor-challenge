use std::path::Path;
use std::fs;

pub mod error;
pub mod vm;
pub mod util;
pub mod disassembler;

pub use error::{Error, Result};
pub use vm::{SynacorVM, Status, Event};
use crate::vm::STACK_LEN;

pub const SAVE_DATA_LEN: usize = 0x800A + STACK_LEN;

#[derive(Debug, Clone)]
pub struct Stack<T: Default + Copy, const S: usize> {
    pointer: usize,
    contents: [T; S],
}

impl<T: Default + Copy, const S: usize> Stack<T, S> {
    pub fn new() -> Self {
        Self {
            pointer: 0,
            contents: [T::default(); S],
        }
    }

    pub fn push(&mut self, val: T) -> Result<()> {
        if self.pointer >= S {
            Err(Error::StackOverflow)
        } else {
            self.contents[self.pointer] = val;
            self.pointer += 1;
            Ok(())
        }
    }

    pub fn pop(&mut self) -> Result<T> {
        if self.pointer == 0 {
            Err(Error::StackUnderflow)
        } else {
            self.pointer -= 1;
            Ok(std::mem::replace(&mut self.contents[self.pointer], T::default()))
        }
    }

    pub fn contents(&self) -> &[T; S] { &self.contents }

    pub fn pointer(&self) -> &usize { &self.pointer }

    pub fn contents_mut(&mut self) -> &mut [T; S] { &mut self.contents }

    pub fn pointer_mut(&mut self) -> &mut usize { &mut self.pointer }

    pub fn len(&self) -> usize {
        S
    }
}

pub fn save_vm_state<P: AsRef<Path>>(vm: &SynacorVM, path: P) -> Result<()> {
    let mut data = [0; SAVE_DATA_LEN];
    data[0] = vm.pc();
    data[0x1..0x9].clone_from_slice(vm.registers());
    data[0x9..0x8009].clone_from_slice(vm.memory());
    data[0x8009] = vm.stack().pointer as u16;
    data[0x800A..].clone_from_slice(vm.stack().contents());

    let buf = util::u16_array_to_u8(&data);
    fs::write(path, buf).map_err(|e| Error::IO(e))
}

pub fn load_vm_state(vm: &mut SynacorVM, data: &[u16]) -> Result<()> {
    if data.len() != SAVE_DATA_LEN {
        return Err(Error::InvalidDataLength(data.len()));
    }

    *vm.pc_mut() = data[0];
    vm.registers_mut().clone_from_slice(&data[0x1..0x9]);
    vm.memory_mut().clone_from_slice(&data[0x9..0x8009]);
    *vm.stack_mut().pointer_mut() = data[0x8009] as usize;
    vm.stack_mut().contents_mut().clone_from_slice(&data[0x800A..SAVE_DATA_LEN]);

    Ok(())
}

