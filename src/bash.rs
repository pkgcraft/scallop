mod internal {
    #![allow(non_camel_case_types)]
    #![allow(non_upper_case_globals)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    #![allow(unreachable_pub)]
    #![allow(clippy::all)]
    // ignore warning from bindgen-generated struct alignment tests
    // https://github.com/rust-lang/rust-bindgen/issues/1651
    #![allow(deref_nullptr)]

    use std::os::raw::{c_char, c_int};

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

    include!(concat!(env!("OUT_DIR"), "/bash-bindings.rs"));

    // Provide external access to various builtins that aren't explicitly exported.
    extern "C" {
        pub fn local_builtin(list: *mut WordList) -> c_int;
    }
}

pub(crate) use internal::*;
// export Builtin for scallop-builtins to use
pub use internal::Builtin;
