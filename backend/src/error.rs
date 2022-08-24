use std::fmt::{Debug, Display, Formatter};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum Error {
    IllegalParameterWrite(u16),
    IllegalParameterRead(u16),
    IllegalOpcode(u16),
    StackOverflow,
    StackUnderflow,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IllegalParameterWrite(p) => write!(f, "illegal register write - 0x{:04X}", p),
            Self::IllegalParameterRead(p) => write!(f, "illegal parameter read - 0x{:04X}", p),
            Self::IllegalOpcode(op) => write!(f, "illegal opcode - 0x{:04X}", op),
            Self::StackOverflow => write!(f, "stack overflow"),
            Self::StackUnderflow => write!(f, "stack underflow"),
        }
    }
}

impl std::error::Error for Error {}
