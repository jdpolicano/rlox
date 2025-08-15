use crate::bytecode::gc::heap::Heap;

pub trait Trace {
    fn trace(&self, cx: &mut Heap);
}
