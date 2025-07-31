use super::control::Control;
use super::error::LoxError;
use super::object::LoxObject;
use std::fmt;

pub type EvalResult = Result<Eval, LoxError>;

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

    pub fn type_str(&self) -> &str {
        match self {
            Self::Ctrl(ctrl) => ctrl.type_str(),
            Self::Object(obj) => obj.type_str(),
        }
    }

    pub fn unwrap_to_obj(&self) -> Result<LoxObject, LoxError> {
        match self {
            Self::Ctrl(ctrl) => {
                let msg = format!("")
            },
            Self::Object(obj) => obj.type_str(),
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

fn type_error(ctx: &str, type_str: &str, view: View) -> LoxError {
    let msg = if ctx.len() > 0 {
        format!("unexpected type '{}' {}", type_str, ctx)
    } else {
        format!("unexpected type '{}'", type_str)
    };
    LoxError::TypeError { msg, view }
}

fn unwrap_to_type_error(ctx: &str, eval: Eval, view: View) -> Result<LoxObject, LoxError> {
    match eval {
        Eval::Object(obj) => Ok(obj),
        Eval::Ctrl(ctrl) => Err(type_error(ctx, ctrl.type_str(), view)),
    }
}
