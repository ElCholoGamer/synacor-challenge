use std::fmt::{Debug, Display, Formatter};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
    IllegalParameterWrite(u16),
    IllegalParameterRead(u16),
    IllegalOpcode(u16),
    StackUnderflow,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IllegalParameterWrite(p) => write!(f, "illegal parameter write - {}", p),
            Self::IllegalParameterRead(p) => write!(f, "illegal parameter read - {}", p),
            Self::IllegalOpcode(op) => write!(f, "illegal opcode - {}", op),
            Self::StackUnderflow => write!(f, "stack underflow"),
        }
    }
}

impl std::error::Error for Error {}
