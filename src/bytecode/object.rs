use std::{
    fmt,
    ops::{Add, Div, Mul, Neg, Sub},
};

#[derive(Debug, Clone)]
pub enum LoxObject {
    Number(f64),
    Error(String),
}

impl fmt::Display for LoxObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{}", n),
            Self::Error(e) => write!(f, "ERROR: {}", e),
        }
    }
}

impl Add for LoxObject {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(a), Self::Number(b)) => Self::Number(a + b),
            _ => Self::Error("Cannot add non-number types".to_string()),
        }
    }
}

impl Sub for LoxObject {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(a), Self::Number(b)) => Self::Number(a - b),
            _ => Self::Error("Cannot subtract non-number types".to_string()),
        }
    }
}

impl Mul for LoxObject {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(a), Self::Number(b)) => Self::Number(a * b),
            _ => Self::Error("Cannot multiply non-number types".to_string()),
        }
    }
}

impl Div for LoxObject {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(a), Self::Number(b)) => {
                if b == 0.0 {
                    Self::Error("Division by zero".to_string())
                } else {
                    Self::Number(a / b)
                }
            }
            _ => Self::Error("Cannot divide non-number types".to_string()),
        }
    }
}

impl Neg for LoxObject {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            Self::Number(n) => Self::Number(-n),
            _ => Self::Error("Cannot convert to a number".to_string()),
        }
    }
}
