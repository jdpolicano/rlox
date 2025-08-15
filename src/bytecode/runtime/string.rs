use crate::bytecode::gc::heap::Heap;
use crate::bytecode::gc::trace::Trace;
use crate::bytecode::runtime::object::LoxObject;

pub struct LoxString([u8]);

impl Trace for LoxString {
    type Cx = Heap<Self>;
    fn trace(&self, _: &mut Self::Cx) {}
}
