use std::fs;
use std::path::Path;
use backend::{Error, SynacorVM, Result, util};
use backend::vm::STACK_LEN;

const SAVE_DATA_LEN: usize = 0x800A + STACK_LEN;

#[derive(Debug)]
pub struct LimitedQueue<T> {
    contents: Vec<T>
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

    pub fn contents(&self) -> &Vec<T> { &self.contents }

    pub fn peek_last(&self) -> Option<&T> { self.contents.last() }

    pub fn len(&self) -> usize { self.contents.len() }
}

pub fn u16_array_to_u8(src: &[u16]) -> Vec<u8> {
    src.iter()
        .map(|v| [(v & 0xFF) as u8, ((v >> 8) & 0xFF) as u8])
        .collect::<Vec<[u8; 2]>>().concat()
}

pub fn u8_array_to_u16(src: &[u8]) -> Vec<u16> {
    src.chunks(2)
        .map(|chunk| util::concat_u16(*chunk.get(1).unwrap_or(&0), chunk[0]))
        .collect()
}

pub fn save_vm_state<P: AsRef<Path>>(vm: &SynacorVM, path: P, initial_output: String) -> Result<()> {
    let mut str_bytes: Vec<u8> = initial_output.bytes().collect();
    let mut data = [0; SAVE_DATA_LEN + 1];

    data[0] = vm.pc();
    data[0x1..0x9].clone_from_slice(vm.registers());
    data[0x9..0x8009].clone_from_slice(vm.memory());
    data[0x8009] = vm.stack().pointer() as u16;
    data[0x800A..SAVE_DATA_LEN].clone_from_slice(vm.stack().contents());
    data[SAVE_DATA_LEN] = str_bytes.len() as u16;

    let mut buf = u16_array_to_u8(&data);
    buf.append(&mut str_bytes);

    fs::write(path, buf).map_err(|e| Error::IO(e))
}

pub fn load_vm_state(vm: &mut SynacorVM, data: &[u16]) -> Result<String> {
    if data.len() < SAVE_DATA_LEN {
        return Err(Error::InvalidDataLength(data.len()));
    }

    *vm.pc_mut() = data[0];
    vm.registers_mut().clone_from_slice(&data[0x1..0x9]);
    vm.memory_mut().clone_from_slice(&data[0x9..0x8009]);
    *vm.stack_mut().pointer_mut() = data[0x8009] as usize;
    vm.stack_mut().contents_mut().clone_from_slice(&data[0x800A..SAVE_DATA_LEN]);

    let str_size = data[SAVE_DATA_LEN];
    let u16_str_size = (str_size / 2) + (str_size & 1);
    let str_bytes = u16_array_to_u8(&data[SAVE_DATA_LEN + 1..SAVE_DATA_LEN + 1 + u16_str_size as usize]);

    Ok(String::from_utf8(str_bytes).unwrap())
}