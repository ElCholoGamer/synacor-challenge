use std::path::Path;

pub mod error;
pub mod vm;
pub mod util;
pub mod disassembler;

pub use error::{Error, Result};
pub use vm::{SynacorVM, Status, Event};
pub use disassembler::disassemble;

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

    pub fn pointer(&self) -> usize { self.pointer }

    pub fn contents_mut(&mut self) -> &mut [T; S] { &mut self.contents }

    pub fn pointer_mut(&mut self) -> &mut usize { &mut self.pointer }

    pub fn len(&self) -> usize {
        S
    }
}

