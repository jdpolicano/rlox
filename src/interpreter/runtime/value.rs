use crate::lang::tree::ast;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(Rc<String>),
    Boolean(bool),
    Nil,
}

impl From<ast::Literal> for Value {
    fn from(value: ast::Literal) -> Self {
        match value {
            ast::Literal::Boolean { value, .. } => Value::Boolean(value),
            ast::Literal::String { value, .. } => Value::String(value),
            ast::Literal::Number { value, .. } => Value::Number(value),
            ast::Literal::Nil { .. } => Value::Nil,
        }
    }
}

impl From<&ast::Literal> for Value {
    fn from(value: &ast::Literal) -> Self {
        match value {
            ast::Literal::Boolean { value, .. } => Value::Boolean(*value),
            ast::Literal::String { value, .. } => Value::String(value.clone()),
            ast::Literal::Number { value, .. } => Value::Number(*value),
            ast::Literal::Nil { .. } => Value::Nil,
        }
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(Rc::new(value.to_string()))
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(Rc::new(value))
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
        }
    }
}

impl Value {
    pub fn truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Nil => false,
            Value::Number(n) if *n == 0f64 => false,
            _ => true,
        }
    }

    pub fn type_str(&self) -> &'static str {
        match self {
            Value::Boolean(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Nil => "nil",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Control {
    Break,
    Continue,
    // Return(Value) - saving this for when we implement functions :)
}

impl Control {
    fn type_str(&self) -> &str {
        match self {
            Self::Break => "break",
            Self::Continue => "continue",
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

#[derive(Debug, Clone)]
pub enum LoxObject {
    Value(Value),
    Control(Control),
}

impl From<ast::Literal> for LoxObject {
    fn from(value: ast::Literal) -> Self {
        Self::Value(value.into())
    }
}

impl From<&ast::Literal> for LoxObject {
    fn from(value: &ast::Literal) -> Self {
        Self::Value(value.into())
    }
}

impl From<f64> for LoxObject {
    fn from(value: f64) -> Self {
        Self::Value(value.into())
    }
}

impl From<bool> for LoxObject {
    fn from(value: bool) -> Self {
        Self::Value(value.into())
    }
}

impl From<&str> for LoxObject {
    fn from(value: &str) -> Self {
        Self::Value(value.into())
    }
}

impl From<(&str, &str)> for LoxObject {
    fn from(value: (&str, &str)) -> Self {
        let mut container = String::with_capacity(value.0.len() + value.1.len());
        container.push_str(value.0);
        container.push_str(value.1);
        Self::Value(container.into())
    }
}

impl fmt::Display for LoxObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoxObject::Value(prim) => write!(f, "{}", prim),
            LoxObject::Control(ctrl) => write!(f, "{}", ctrl),
        }
    }
}

impl PartialEq for LoxObject {
    fn eq(&self, other: &LoxObject) -> bool {
        match (self, other) {
            (LoxObject::Value(a), LoxObject::Value(b)) => a.eq(b),
            _ => false,
        }
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl LoxObject {
    pub fn new_nil() -> Self {
        Self::Value(Value::Nil)
    }

    pub fn new_break() -> Self {
        Self::Control(Control::Break)
    }

    pub fn new_continue() -> Self {
        Self::Control(Control::Continue)
    }

    pub fn is_number(&self) -> bool {
        match self {
            LoxObject::Value(Value::Number(_)) => true,
            _ => false,
        }
    }

    pub fn is_string(&self) -> bool {
        match self {
            LoxObject::Value(Value::String(_)) => true,
            _ => false,
        }
    }

    pub fn is_boolean(&self) -> bool {
        match self {
            LoxObject::Value(Value::Boolean(_)) => true,
            _ => false,
        }
    }

    pub fn is_nil(&self) -> bool {
        match self {
            LoxObject::Value(Value::Nil) => true,
            _ => false,
        }
    }

    pub fn is_break(&self) -> bool {
        match self {
            Self::Control(ctrl) => *ctrl == Control::Break,
            _ => false,
        }
    }

    pub fn is_continue(&self) -> bool {
        match self {
            Self::Control(ctrl) => *ctrl == Control::Continue,
            _ => false,
        }
    }

    pub fn is_control(&self) -> bool {
        match self {
            Self::Control(_) => true,
            _ => false,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        if let LoxObject::Value(Value::Number(n)) = self {
            Some(*n)
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<&String> {
        if let LoxObject::Value(Value::String(s)) = self {
            Some(s)
        } else {
            None
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        if let LoxObject::Value(Value::Boolean(b)) = self {
            Some(*b)
        } else {
            None
        }
    }

    pub fn as_nil(&self) -> Option<()> {
        if let LoxObject::Value(Value::Nil) = self {
            Some(())
        } else {
            None
        }
    }

    pub fn truthy(&self) -> bool {
        match self {
            LoxObject::Value(prim) => prim.truthy(),
            _ => false,
        }
    }

    pub fn type_str(&self) -> &str {
        match self {
            LoxObject::Value(p) => p.type_str(),
            LoxObject::Control(ctrl) => ctrl.type_str(),
        }
    }
}
