use std::ffi::CStr;
use std::os::raw::c_char;

pub mod builtins;
pub mod command;

/// Global variables exposed by bash.
extern "C" {
    static this_command_name: *mut c_char;
}

/// Get the currently running bash command name.
#[inline]
pub unsafe fn current_command() -> &'static str {
    unsafe { CStr::from_ptr(this_command_name).to_str().unwrap() }
}
