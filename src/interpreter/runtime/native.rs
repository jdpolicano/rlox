use super::eval::Eval;
use super::object::LoxObject;
use crate::interpreter::lox::Lox;
use crate::interpreter::runtime::error::LoxError;
use crate::interpreter::runtime::error::NativeError;
use crate::interpreter::runtime::error::RuntimeError;
use std::time::{SystemTime, UNIX_EPOCH};

pub type NativeFn = fn(&mut Lox, Vec<LoxObject>) -> Result<Eval, RuntimeError>;

pub fn setup_native(runtime: &mut Lox) {
    runtime.bind_local("clock", lox_native_object(clock));
}

pub fn clock(_: &mut Lox, _: Vec<LoxObject>) -> Result<Eval, RuntimeError> {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => Ok(LoxObject::from(n.as_secs_f64()).into()),
        Err(_) => {
            let msg = "clock() SystemTime before UNIX EPOCH".to_string();
            let inner = NativeError::SystemError(msg);
            Err(RuntimeError::from(LoxError::from(inner)))
        }
    }
}

fn lox_native_object(func: NativeFn) -> LoxObject {
    LoxObject::Native(func)
}
