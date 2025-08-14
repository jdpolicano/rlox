use crate::bytecode::gc::allocator::{Gc, Heap};
use crate::bytecode::gc::trace::Trace;
use crate::bytecode::runtime::error::ErrorObject;
use crate::bytecode::runtime::string::LoxString;
use crate::bytecode::runtime::value::LoxValue;
use std::ops::{Add, Div, Mul, Neg, Sub};

pub enum LoxObject {
    Value(LoxValue),
    String(Gc<LoxString>),
    // for now we will make error's a first class
    // type, and then maybe later add a global runtime object
    // wrapper for creating custom types.
    Error(Gc<ErrorObject>),
}

impl Trace for LoxObject {
    fn trace<T: Trace>(&self, heap: &mut Heap<T>) {
        match self {
            Self::String(s) => s.trace(heap),
            _ => {}
        }
    }
}
