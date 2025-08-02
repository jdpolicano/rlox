use super::control::Control;
use super::error::LoxError;
use super::error::RuntimeError;
use super::object::LoxObject;
use std::fmt;

pub type EvalResult = Result<Eval, RuntimeError>;

#[derive(Debug, Clone)]
pub enum Eval {
    Ctrl(Control),
    Object(LoxObject),
}

impl From<LoxObject> for Eval {
    fn from(value: LoxObject) -> Self {
        Self::Object(value)
    }
}

impl From<Control> for Eval {
    fn from(value: Control) -> Self {
        Self::Ctrl(value)
    }
}

impl From<f64> for Eval {
    fn from(value: f64) -> Self {
        Self::Object(value.into())
    }
}

impl fmt::Display for Eval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ctrl(ctrl) => write!(f, "{}", ctrl),
            Self::Object(obj) => write!(f, "{}", obj),
        }
    }
}

impl Eval {
    pub fn is_break(&self) -> bool {
        match self {
            Self::Ctrl(ctrl) => ctrl.is_break(),
            _ => false,
        }
    }

    pub fn is_continue(&self) -> bool {
        match self {
            Self::Ctrl(ctrl) => ctrl.is_continue(),
            _ => false,
        }
    }

    pub fn is_return(&self) -> bool {
        match self {
            Self::Ctrl(ctrl) => ctrl.is_return(),
            _ => false,
        }
    }

    pub fn is_control(&self) -> bool {
        match self {
            Self::Ctrl(_) => true,
            _ => false,
        }
    }

    pub fn truthy(&self) -> bool {
        match self {
            Self::Ctrl(_) => false,
            Self::Object(obj) => obj.truthy(),
        }
    }

    pub fn new_nil() -> Self {
        Self::Object(LoxObject::new_nil())
    }

    pub fn new_continue() -> Self {
        Self::Ctrl(Control::Continue)
    }

    pub fn new_break() -> Self {
        Self::Ctrl(Control::Break)
    }

    pub fn new_return(v: LoxObject) -> Self {
        Self::Ctrl(Control::new_return(v))
    }

    pub fn type_str(&self) -> &str {
        match self {
            Self::Ctrl(ctrl) => ctrl.type_str(),
            Self::Object(obj) => obj.type_str(),
        }
    }

    pub fn unwrap_return(self) -> Self {
        match self {
            Self::Ctrl(Control::Return(v)) => Self::Object(v),
            _ => self,
        }
    }

    pub fn with_object<F, T>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&LoxObject) -> T,
    {
        match self {
            Self::Ctrl(_) => None,
            Self::Object(obj) => Some(f(obj)),
        }
    }
}
