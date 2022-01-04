#![warn(unreachable_pub)]
#![deny(unsafe_op_in_unsafe_fn)]

use std::ffi::{CStr, CString};

pub mod bindings;
pub mod builtins;
pub mod error;
pub mod traits;

pub use self::error::{Error, Result};

/// Get the currently running command name.
#[inline]
pub fn current_command() -> &'static str {
    unsafe {
        CStr::from_ptr(bindings::this_command_name)
            .to_str()
            .unwrap()
    }
}

/// Get the string value of a given variable name.
pub fn string_value(name: &str) -> Option<&str> {
    let name = CString::new(name).unwrap();
    match unsafe { bindings::get_string_value(name.as_ptr()) } {
        s if s.is_null() => None,
        s => Some(unsafe { CStr::from_ptr(s).to_str().unwrap() }),
    }
}
