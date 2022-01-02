use std::os::raw::{c_char, c_int};

include!(concat!(env!("OUT_DIR"), "/bash-bindings.rs"));

type BuiltinFnPtr = unsafe extern "C" fn(list: *mut WordList) -> c_int;

#[repr(C)]
pub struct Builtin {
    pub name: *const c_char,
    pub function: BuiltinFnPtr,
    pub flags: c_int,
    pub long_doc: *const *const c_char,
    pub short_doc: *const c_char,
    pub handle: *mut c_char,
}
