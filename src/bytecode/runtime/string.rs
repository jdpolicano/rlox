use crate::bytecode::gc::allocator::Heap;
use crate::bytecode::gc::trace::Trace;
use crate::bytecode::runtime::object::LoxObject;

pub struct LoxString(Box<str>);

impl Trace for LoxString {
    fn trace<T: Trace>(&self, _: &mut Heap<T>) {}
}
