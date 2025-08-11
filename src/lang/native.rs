/// trait Native defines the required signature a structure
/// must adhere to do to provide the full scope of native functions
/// any runtime must support.

pub trait Native<T> {
    fn clock(&mut self, _: &[T]) -> T;
}
