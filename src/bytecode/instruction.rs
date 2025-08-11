use std::fmt;

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum OpCode {
    Return,
    Constant,
    ConstantLong,
    Negate,
    Add,
    Sub,
    Mul,
    Div,
    Unknown,
}

impl fmt::Debug for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Return => write!(f, "RETURN"),
            Self::Constant => write!(f, "CONSTANT"),
            Self::ConstantLong => write!(f, "CONSTANT_LONG"),
            Self::Negate => write!(f, "NEGATE"),
            Self::Add => write!(f, "ADD"),
            Self::Sub => write!(f, "SUB"),
            Self::Mul => write!(f, "MUL"),
            Self::Div => write!(f, "DIV"),
            Self::Unknown => write!(f, "ERR: UNKNOWN!"),
        }
    }
}

pub struct OpConversionError;

impl From<u8> for OpCode {
    fn from(value: u8) -> Self {
        match value {
            0 => OpCode::Return,
            1 => OpCode::Constant,
            2 => OpCode::ConstantLong,
            3 => OpCode::Negate,
            4 => OpCode::Add,
            5 => OpCode::Sub,
            6 => OpCode::Mul,
            7 => OpCode::Div,
            _ => OpCode::Unknown,
        }
    }
}

impl From<&u8> for OpCode {
    fn from(value: &u8) -> Self {
        OpCode::from(*value)
    }
}

impl OpCode {
    pub fn num_args(&self) -> usize {
        match self {
            Self::Constant => 1,
            Self::ConstantLong => 2,
            Self::Return => 0,
            Self::Add | Self::Sub | Self::Mul | Self::Div => 0,
            _ => 0,
        }
    }
}
