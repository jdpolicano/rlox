use std::ptr::NonNull;

pub type Obj = NonNull<Header>;

pub enum ObjectType {
    String,
}

#[repr(C)]
pub struct Header {
    obj_type: ObjectType,
    marked: bool,
    len: usize,
}

#[repr(C)]
pub struct Object<T: ?Sized> {
    header: Header,
    data: T,
}
