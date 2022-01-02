use std::ffi::{CStr, CString};

pub mod builtins;
pub mod command;

include!(concat!(env!("OUT_DIR"), "/bash-bindings.rs"));

/// Get the currently running bash command name.
#[inline]
pub unsafe fn current_command() -> &'static str {
    unsafe { CStr::from_ptr(this_command_name).to_str().unwrap() }
}

/// Get the string value of a given variable name.
pub fn string_value(name: &str) -> Option<&str> {
    let name = CString::new(name).unwrap().into_raw();
    match unsafe { get_string_value(name) } {
        s if s.is_null() => None,
        s => Some(unsafe { CStr::from_ptr(s).to_str().unwrap() }),
    }
}
