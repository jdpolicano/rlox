use crate::bytecode::gc::obj::{Header, Obj};
use crate::bytecode::gc::trace::Trace;
use std::ptr::NonNull;

// A safe wrapper over GcObj<T>
pub struct GcBox(Obj);

impl std::ops::Deref for GcBox {
    type Target = Header;
    fn deref(&self) -> &Self::Target {
        unsafe { (*self).0.as_ref() }
    }
}

impl std::ops::DerefMut for GcBox {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { (*self).0.as_mut() }
    }
}

// The heap that stores objects
pub struct Heap<T: Trace<Cx = Heap<T>> + ?Sized> {
    objects: Vec<Box<GcObj<T>>>,
}

impl<T: Trace<Cx = Heap<T>> + ?Sized> Heap<T> {
    // Allocate an object and add it to the heap
    pub fn allocate(&mut self, obj: T) -> GcBox<T> {
        let gc_obj = Box::new(GcObj { marked: false, obj });
        let raw_ptr = Box::into_raw(gc_obj);
        self.objects.push(unsafe { Box::from_raw(raw_ptr) });
        GcBox {
            ptr: NonNull::new(raw_ptr).unwrap(),
        }
    }

    fn mark_object(&mut self, gc_box: &GcBox<T>) {
        let mut obj_ptr = gc_box.ptr;
        unsafe {
            if obj_ptr.as_ref().marked {
                return;
            }
            // Mark the current object
            obj_ptr.as_mut().marked = true;
            obj_ptr.as_mut().obj.trace(self)
        }
    }

    // Mark phase: Traverse all roots and mark reachable objects
    fn mark_roots(&mut self, roots: &[&GcBox<T>]) {
        for root in roots {
            self.mark_object(*root);
        }
    }

    // Sweep phase: Remove all unmarked objects
    fn sweep(&mut self) {
        self.objects.retain_mut(|obj_box| {
            if !obj_box.marked {
                // deallocate the object here if needed
                false // remove object
            } else {
                // Reset for the next cycle
                obj_box.marked = false;
                true
            }
        });
    }

    // Trigger GC process
    pub fn collect_garbage(&mut self, roots: &[&GcBox<T>]) {
        self.mark_roots(roots);
        self.sweep();
    }
}

// Allocate an object and add it to the heap
