use crate::lang::tree::ast;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    Number(f64),
    String(Rc<String>),
    Boolean(bool),
    Nil,
}

impl From<ast::Literal> for Primitive {
    fn from(value: ast::Literal) -> Self {
        match value {
            ast::Literal::Boolean { value, .. } => Primitive::Boolean(value),
            ast::Literal::String { value, .. } => Primitive::String(value),
            ast::Literal::Number { value, .. } => Primitive::Number(value),
            ast::Literal::Nil { .. } => Primitive::Nil,
        }
    }
}

impl From<&ast::Literal> for Primitive {
    fn from(value: &ast::Literal) -> Self {
        match value {
            ast::Literal::Boolean { value, .. } => Primitive::Boolean(*value),
            ast::Literal::String { value, .. } => Primitive::String(value.clone()),
            ast::Literal::Number { value, .. } => Primitive::Number(*value),
            ast::Literal::Nil { .. } => Primitive::Nil,
        }
    }
}

impl From<f64> for Primitive {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<bool> for Primitive {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<&str> for Primitive {
    fn from(value: &str) -> Self {
        Self::String(Rc::new(value.to_string()))
    }
}

impl From<String> for Primitive {
    fn from(value: String) -> Self {
        Self::String(Rc::new(value))
    }
}

impl fmt::Display for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Primitive::Number(n) => write!(f, "{}", n),
            Primitive::String(s) => write!(f, "{}", s),
            Primitive::Boolean(b) => write!(f, "{}", b),
            Primitive::Nil => write!(f, "nil"),
        }
    }
}

impl Primitive {
    pub fn truthy(&self) -> bool {
        match self {
            Primitive::Boolean(b) => *b,
            Primitive::Nil => false,
            Primitive::Number(n) if *n == 0f64 => false,
            _ => true,
        }
    }

    pub fn type_str(&self) -> &'static str {
        match self {
            Primitive::Boolean(_) => "boolean",
            Primitive::Number(_) => "number",
            Primitive::String(_) => "string",
            Primitive::Nil => "nil",
        }
    }
}
