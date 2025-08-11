use std::fmt;

use crate::interpreter::runtime::object::LoxObject;

#[derive(Debug, Clone, PartialEq)]
pub enum Control {
    Break,
    Continue,
    Return(LoxObject),
}

impl Control {
    pub fn type_str(&self) -> &str {
        match self {
            Self::Break => "break",
            Self::Continue => "continue",
            Self::Return(_) => "return",
        }
    }

    pub fn new_return(v: LoxObject) -> Self {
        Self::Return(v)
    }

    pub fn is_return(&self) -> bool {
        match self {
            Self::Return(_) => true,
            _ => false,
        }
    }

    pub fn is_break(&self) -> bool {
        match self {
            Self::Break => true,
            _ => false,
        }
    }

    pub fn is_continue(&self) -> bool {
        match self {
            Self::Continue => true,
            _ => false,
        }
    }
}

impl fmt::Display for Control {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Break | Self::Continue => Ok(()),
            Self::Return(v) => write!(f, "return({})", v),
        }
    }
}
