use std::fmt::{Debug, Display, Formatter};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum Error {
    IllegalParameterWrite(u16),
    IllegalParameterRead(u16),
    IllegalOpcode(u16),
    StackOverflow,
    StackUnderflow,
    IO(std::io::Error),
    InvalidDataLength(usize),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IllegalParameterWrite(p) => write!(f, "illegal parameter write - {}", p),
            Self::IllegalParameterRead(p) => write!(f, "illegal parameter read - {}", p),
            Self::IllegalOpcode(op) => write!(f, "illegal opcode - {}", op),
            Self::StackOverflow => write!(f, "stack overflow"),
            Self::StackUnderflow => write!(f, "stack underflow"),
            Self::IO(e) => write!(f, "{}", e.to_string()),
            Self::InvalidDataLength(len) => write!(f, "invalid data length - {} bytes", len),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::IO(e)
    }
}