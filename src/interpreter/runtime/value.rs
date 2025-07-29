use crate::lang::tree::ast;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum LoxPrimitive {
    Number(f64),
    String(Rc<String>),
    Boolean(bool),
    Nil,
}

impl From<ast::Literal> for LoxPrimitive {
    fn from(value: ast::Literal) -> Self {
        match value {
            ast::Literal::Boolean { value, view: _ } => LoxPrimitive::Boolean(value),
            ast::Literal::String { value, view: _ } => LoxPrimitive::String(value),
            ast::Literal::Number { value, view: _ } => LoxPrimitive::Number(value),
            ast::Literal::Nil { view: _ } => LoxPrimitive::Nil,
        }
    }
}

impl From<&ast::Literal> for LoxPrimitive {
    fn from(value: &ast::Literal) -> Self {
        match value {
            ast::Literal::Boolean { value, view: _ } => LoxPrimitive::Boolean(*value),
            ast::Literal::String { value, view: _ } => LoxPrimitive::String(value.clone()),
            ast::Literal::Number { value, view: _ } => LoxPrimitive::Number(*value),
            ast::Literal::Nil { view: _ } => LoxPrimitive::Nil,
        }
    }
}

impl From<f64> for LoxPrimitive {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<bool> for LoxPrimitive {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<&str> for LoxPrimitive {
    fn from(value: &str) -> Self {
        Self::String(Rc::new(value.to_string()))
    }
}

impl From<String> for LoxPrimitive {
    fn from(value: String) -> Self {
        Self::String(Rc::new(value))
    }
}

impl fmt::Display for LoxPrimitive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoxPrimitive::Number(n) => write!(f, "{}", n),
            LoxPrimitive::String(s) => write!(f, "{}", s),
            LoxPrimitive::Boolean(b) => write!(f, "{}", b),
            LoxPrimitive::Nil => write!(f, "nil"),
        }
    }
}

impl LoxPrimitive {
    pub fn truthy(&self) -> bool {
        match self {
            LoxPrimitive::Boolean(b) => *b,
            LoxPrimitive::Nil => false,
            _ => true,
        }
    }

    pub fn type_str(&self) -> &'static str {
        match self {
            LoxPrimitive::Boolean(_) => "boolean",
            LoxPrimitive::Number(_) => "number",
            LoxPrimitive::String(_) => "string",
            LoxPrimitive::Nil => "nil",
        }
    }
}

#[derive(Debug, Clone)]
pub enum LoxValue {
    Primitive(LoxPrimitive),
}

impl From<ast::Literal> for LoxValue {
    fn from(value: ast::Literal) -> Self {
        Self::Primitive(value.into())
    }
}

impl From<&ast::Literal> for LoxValue {
    fn from(value: &ast::Literal) -> Self {
        Self::Primitive(value.into())
    }
}

impl From<f64> for LoxValue {
    fn from(value: f64) -> Self {
        Self::Primitive(value.into())
    }
}

impl From<bool> for LoxValue {
    fn from(value: bool) -> Self {
        Self::Primitive(value.into())
    }
}

impl From<&str> for LoxValue {
    fn from(value: &str) -> Self {
        Self::Primitive(value.into())
    }
}

impl From<(&str, &str)> for LoxValue {
    fn from(value: (&str, &str)) -> Self {
        let mut container = String::with_capacity(value.0.len() + value.1.len());
        container.push_str(value.0);
        container.push_str(value.1);
        Self::Primitive(container.into())
    }
}

impl fmt::Display for LoxValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoxValue::Primitive(prim) => write!(f, "{}", prim),
        }
    }
}

impl PartialEq for LoxValue {
    fn eq(&self, other: &LoxValue) -> bool {
        match (self, other) {
            (LoxValue::Primitive(a), LoxValue::Primitive(b)) => a.eq(b),
            _ => false,
        }
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl LoxValue {
    pub fn is_number(&self) -> bool {
        match self {
            LoxValue::Primitive(LoxPrimitive::Number(_)) => true,
            _ => false,
        }
    }

    pub fn is_string(&self) -> bool {
        match self {
            LoxValue::Primitive(LoxPrimitive::String(_)) => true,
            _ => false,
        }
    }

    pub fn is_boolean(&self) -> bool {
        match self {
            LoxValue::Primitive(LoxPrimitive::Boolean(_)) => true,
            _ => false,
        }
    }

    pub fn is_nil(&self) -> bool {
        match self {
            LoxValue::Primitive(LoxPrimitive::Nil) => true,
            _ => false,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        if let LoxValue::Primitive(LoxPrimitive::Number(n)) = self {
            Some(*n)
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<&String> {
        if let LoxValue::Primitive(LoxPrimitive::String(s)) = self {
            Some(s)
        } else {
            None
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        if let LoxValue::Primitive(LoxPrimitive::Boolean(b)) = self {
            Some(*b)
        } else {
            None
        }
    }

    pub fn as_nil(&self) -> Option<()> {
        if let LoxValue::Primitive(LoxPrimitive::Nil) = self {
            Some(())
        } else {
            None
        }
    }

    pub fn truthy(&self) -> bool {
        match self {
            LoxValue::Primitive(prim) => prim.truthy(),
        }
    }

    pub fn type_str(&self) -> &'static str {
        match self {
            LoxValue::Primitive(p) => p.type_str(),
        }
    }
}
