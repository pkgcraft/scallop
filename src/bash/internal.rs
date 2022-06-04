#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unreachable_pub)]
#![allow(clippy::all)]
// ignore warning from bindgen-generated struct alignment tests
// https://github.com/rust-lang/rust-bindgen/issues/1651
#![allow(deref_nullptr)]

use std::os::raw::c_int;

include!(concat!(env!("OUT_DIR"), "/bash-bindings.rs"));

// Provide external access to builtins since they aren't explicitly exported.
extern "C" {
    pub fn local_builtin(list: *mut WordList) -> c_int;
    pub fn shopt_builtin(list: *mut WordList) -> c_int;
}
