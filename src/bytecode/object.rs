use crate::bytecode::error::{BinOpError, BinOpSide, ErrorObject, LoxError, TypeError};
use std::{
    fmt,
    ops::{Add, Div, Mul, Neg, Sub},
};

#[derive(Debug, Clone)]
pub enum LoxObject {
    Number(f64),
    Error(Box<ErrorObject>),
}

impl LoxObject {
    pub fn binop_error(op_err: BinOpError) -> Self {
        Self::Error(Box::new(ErrorObject::from(op_err)))
    }
}

impl fmt::Display for LoxObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{}", n),
            Self::Error(e) => write!(f, "{}", e),
        }
    }
}

impl Add for LoxObject {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(a), Self::Number(b)) => Self::Number(a + b),
            (Self::Number(_), _) => {
                LoxObject::binop_error(BinOpError::AddOpFailure(BinOpSide::Rhs))
            }
            _ => LoxObject::binop_error(BinOpError::AddOpFailure(BinOpSide::Lhs)),
        }
    }
}

impl Sub for LoxObject {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(a), Self::Number(b)) => Self::Number(a - b),
            (Self::Number(_), _) => {
                LoxObject::binop_error(BinOpError::SubOpFailure(BinOpSide::Rhs))
            }
            _ => LoxObject::binop_error(BinOpError::SubOpFailure(BinOpSide::Lhs)),
        }
    }
}

impl Mul for LoxObject {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(a), Self::Number(b)) => Self::Number(a * b),
            (Self::Number(_), _) => {
                LoxObject::binop_error(BinOpError::MulOpFailure(BinOpSide::Rhs))
            }
            _ => LoxObject::binop_error(BinOpError::MulOpFailure(BinOpSide::Lhs)),
        }
    }
}

impl Div for LoxObject {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(a), Self::Number(b)) => {
                if b == 0.0 {
                    LoxObject::binop_error(BinOpError::DivByZero)
                } else {
                    Self::Number(a / b)
                }
            }
            (Self::Number(_), _) => {
                LoxObject::binop_error(BinOpError::DivOpFailure(BinOpSide::Rhs))
            }
            _ => LoxObject::binop_error(BinOpError::DivOpFailure(BinOpSide::Lhs)),
        }
    }
}

impl Neg for LoxObject {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            Self::Number(n) => Self::Number(-n),
            _ => LoxObject::binop_error(BinOpError::NegOpFailure),
        }
    }
}
