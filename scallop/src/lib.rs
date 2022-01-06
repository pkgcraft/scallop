#![warn(unreachable_pub)]
#![deny(unsafe_op_in_unsafe_fn)]

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::slice;

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

/// Get the value of an array for a given variable name.
pub fn array_to_vec(name: &str) -> Result<Vec<&str>> {
    let var_name = CString::new(name).unwrap();
    let array_ptr: *mut bash::Array =
        match unsafe { bash::find_variable(var_name.as_ptr()).as_ref() } {
            None => return Err(Error::new(format!("undefined variable: {}", name))),
            Some(v) => match (v.attributes as u32 & bash::att_array) != 0 {
                true => v.value as *mut bash::Array,
                false => return Err(Error::new(format!("variable is not an array: {}", name))),
            },
        };
    let mut count: i32 = 0;
    let array: *mut *mut c_char = unsafe { bash::array_to_argv(array_ptr, &mut count) };
    let values = unsafe { slice::from_raw_parts(array, count as usize) };
    let vec = values
        .iter()
        .map(|s| unsafe { CStr::from_ptr(*s).to_str().unwrap() })
        .collect();
    Ok(vec)
}
