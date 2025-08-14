use crate::bytecode::gc::trace::Trace;

// A safe wrapper over GcBox<T>
pub struct Gc<T: Trace> {
    ptr: *mut GcBox<T>,
}

impl<T: Trace> std::ops::Deref for Gc<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.ptr).obj }
    }
}

impl<T: Trace> std::ops::DerefMut for Gc<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut (*self.ptr).obj }
    }
}

// impl<T: Trace> Drop for Gc<T> {
//     fn drop(&mut self) {
//         // You might not want to drop immediately:
//         // Instead, let the GC handle deallocation.
//         // So you could leave this empty or log the drop.
//     }
// }

pub struct GcBox<T> {
    marked: bool,
    obj: T,
}

// The heap that stores objects
pub struct Heap<T: Trace> {
    objects: Vec<Box<GcBox<T>>>,
}

impl<T: Trace> Heap<T> {
    // Allocate an object and add it to the heap
    fn allocate(&mut self, obj: T) -> *mut GcBox<T> {
        let gc_obj = Box::new(GcBox { marked: false, obj });
        let raw_ptr = Box::into_raw(gc_obj);
        self.objects.push(unsafe { Box::from_raw(raw_ptr) });
        raw_ptr
    }

    // Mark phase: Traverse all roots and mark reachable objects
    fn mark(&mut self, roots: &[&GcBox<T>]) {
        for root in roots {
            self.mark_object(*root);
        }
    }

    fn mark_object(&mut self, obj: &GcBox<T>) {
        if obj.marked {
            return;
        }
        unsafe {
            // Mark the current object
            let obj_ptr = (obj as *const GcBox<T>) as *mut GcBox<T>;
            (*obj_ptr).marked = true;
            (*obj_ptr).obj.trace(self)
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
    fn collect_garbage(&mut self, roots: &[&GcBox<T>]) {
        self.mark(roots);
        self.sweep();
    }
}
