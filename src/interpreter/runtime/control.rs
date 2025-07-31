use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Control {
    Break,
    Continue,
    // Return(Value) - saving this for when we implement functions :)
}

impl Control {
    pub fn type_str(&self) -> &str {
        match self {
            Self::Break => "break",
            Self::Continue => "continue",
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
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Break | Self::Continue => Ok(()),
        }
    }
}
