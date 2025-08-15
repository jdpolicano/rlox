use crate::bytecode::gc::heap::GcBox;
use crate::bytecode::gc::trace::Trace;
use crate::bytecode::runtime::error::ErrorObject;
use crate::bytecode::runtime::string::LoxString;
use crate::bytecode::runtime::value::LoxValue;
use std::ops::{Add, Div, Mul, Neg, Sub};

pub enum LoxObject {
    Value(LoxValue),
    String(GcBox<LoxString>),
    // for now we will make error's a first class
    // type, and then maybe later add a global runtime object
    // wrapper for creating custom types.
    Error(GcBox<ErrorObject>),
}
