use crate::interpreter::lox::Lox;
use crate::interpreter::runtime::error::LoxError;
use crate::interpreter::runtime::error::NativeError;
use crate::interpreter::runtime::eval::Eval;
use crate::interpreter::runtime::object::LoxObject;
use std::time::{SystemTime, UNIX_EPOCH};

pub type NativeFn = fn(&mut Lox, Vec<LoxObject>) -> Result<Eval, LoxError>;

pub fn setup_native(runtime: &mut Lox) {
    runtime.set_global("clock", LoxObject::Native(clock));
    runtime.set_global("string", LoxObject::Native(to_string));
}

pub fn clock(_lox: &mut Lox, _args: Vec<LoxObject>) -> Result<Eval, LoxError> {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => Ok(LoxObject::from(n.as_secs_f64()).into()),
        Err(_) => {
            let msg = "clock() SystemTime before UNIX EPOCH".to_string();
            let inner = NativeError::SystemError(msg);
            Err(LoxError::from(inner))
        }
    }
}

pub fn to_string(_lox: &mut Lox, args: Vec<LoxObject>) -> Result<Eval, LoxError> {
    if args.len() != 1 {
        let err = NativeError::InvalidArguments("to_string() takes only one argument".to_string());
        return Err(LoxError::from(err).into());
    }
    Ok(Eval::Object(LoxObject::from(args[0].to_string())))
}
