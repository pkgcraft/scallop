#![warn(unreachable_pub)]
#![deny(unsafe_op_in_unsafe_fn)]

use std::ffi::{CStr, CString};

pub mod bash;
pub mod builtins;
pub mod command;
pub mod error;
pub mod source;
pub mod traits;
pub mod variables;

pub use self::error::{Error, Result};

/// Get the currently running command name if one exists.
#[inline]
pub fn current_command<'a>() -> Option<&'a str> {
    let cmd_ptr = unsafe { bash::CURRENT_COMMAND };
    match cmd_ptr.is_null() {
        true => None,
        false => {
            let cmd = unsafe { CStr::from_ptr(cmd_ptr).to_str().unwrap() };
            Some(cmd)
        }
    }
}

/// Get the string value of a given variable name.
pub fn string_value(name: &str) -> Option<&str> {
    let name = CString::new(name).unwrap();
    match unsafe { bash::get_string_value(name.as_ptr()) } {
        s if s.is_null() => None,
        s => Some(unsafe { CStr::from_ptr(s).to_str().unwrap() }),
    }
}
