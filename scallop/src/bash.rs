use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::{env, mem, ptr};

mod internal {
    #![allow(non_camel_case_types)]
    #![allow(non_upper_case_globals)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    #![allow(unreachable_pub)]
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/bash-bindings.rs"));
}

pub(crate) use internal::*;

type BuiltinFnPtr = unsafe extern "C" fn(list: *mut WordList) -> c_int;

// Manually define builtin struct since bindgen doesn't support non-null function pointers yet.
// Wrapping the function pointer field member in Option<fn> causes bash to segfault when loading
// a struct during an `enable -f /path/to/lib.so builtin` call.
//
// Related upstream issue: https://github.com/rust-lang/rust-bindgen/issues/1278
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Builtin {
    pub name: *const c_char,
    pub function: BuiltinFnPtr,
    pub flags: c_int,
    pub long_doc: *const *mut c_char,
    pub short_doc: *const c_char,
    pub handle: *mut c_char,
}

/// Start an interactive shell session.
pub fn shell() -> i32 {
    let argv_strs: Vec<CString> = env::args().map(|s| CString::new(s).unwrap()).collect();
    let mut argv_ptrs: Vec<*mut c_char> = argv_strs.iter().map(|s| s.as_ptr() as *mut _).collect();
    argv_ptrs.push(ptr::null_mut());
    let argv = argv_ptrs.as_ptr() as *mut _;
    let argc: c_int = argv_strs.len().try_into().unwrap();
    mem::forget(argv_strs);
    mem::forget(argv_ptrs);

    let env_strs: Vec<CString> = env::vars()
        .map(|(key, val)| format!("{}={}", key, val))
        .map(|s| CString::new(s).unwrap())
        .collect();
    let mut env_ptrs: Vec<*mut c_char> = env_strs.iter().map(|s| s.as_ptr() as *mut _).collect();
    env_ptrs.push(ptr::null_mut());
    let env = env_ptrs.as_ptr() as *mut _;
    mem::forget(env_strs);
    mem::forget(env_ptrs);

    unsafe { bash_main(argc, argv, env) }
}
