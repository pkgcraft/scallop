use std::ffi::CString;
use std::ptr;
use std::str::FromStr;

use crate::bindings;
use crate::Error;

#[derive(Debug)]
pub struct Command {
    ptr: *mut bindings::Command,
}

unsafe impl Send for Command {}

impl Command {
    pub fn execute(&self) {
        unsafe { bindings::execute_command(self.ptr) };
    }
}

impl Drop for Command {
    fn drop(&mut self) {
        unsafe { bindings::dispose_command(self.ptr) };
    }
}

impl FromStr for Command {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cmd_str = CString::new(s).unwrap().into_raw();
        let name_ptr = CString::new("from_str").unwrap().into_raw();
        let cmd_ptr: *mut bindings::Command;

        unsafe {
            bindings::with_input_from_string(cmd_str, name_ptr);
            cmd_ptr = match bindings::parse_command() {
                0 => bindings::copy_command(bindings::GLOBAL_COMMAND),
                _ => return Err(Error::new(format!("failed parsing: {}", s))),
            };

            // clean up global command
            bindings::dispose_command(bindings::GLOBAL_COMMAND);
            bindings::GLOBAL_COMMAND = ptr::null_mut();

            // restore parser input source
            if bindings::STARTUP_STATE == 1 {
                bindings::with_input_from_stdin();
            }
        }

        Ok(Command { ptr: cmd_ptr })
    }
}
