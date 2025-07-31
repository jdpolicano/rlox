use super::eval::Eval;
use super::object::LoxObject;
use crate::interpreter::lox::Lox;
use crate::interpreter::runtime::error::NativeError;
use std::time::{SystemTime, UNIX_EPOCH};

pub type NativeFn = fn(&mut Lox, &[Eval]) -> Result<Eval, NativeError>;

pub fn setup_native(runtime: &mut Lox) {
    runtime.bind_local("clock", lox_native_object(clock));
}

pub fn clock(_: &mut Lox, _: &[Eval]) -> Result<Eval, NativeError> {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => Ok(Eval::from(n.as_secs_f64())),
        Err(_) => Err(NativeError::SystemError(
            "clock() SystemTime before UNIX EPOCH".to_string(),
        )),
    }
}

fn lox_native_object(func: NativeFn) -> LoxObject {
    LoxObject::Native(func)
}
