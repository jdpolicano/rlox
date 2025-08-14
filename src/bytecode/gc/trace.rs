use crate::bytecode::gc::allocator::Heap;

pub trait Trace {
    fn trace<T: Trace>(&self, heap: &mut Heap<T>);
}
