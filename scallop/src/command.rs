use std::ffi::CString;
use std::ptr;
use std::str::FromStr;

use crate::bash;
use crate::Error;

#[derive(Debug)]
pub struct Command {
    ptr: *mut bash::Command,
}

unsafe impl Send for Command {}

impl Command {
    pub fn execute(&self) {
        unsafe { bash::execute_command(self.ptr) };
    }
}

impl Drop for Command {
    fn drop(&mut self) {
        unsafe { bash::dispose_command(self.ptr) };
    }
}

impl FromStr for Command {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cmd_str = CString::new(s).unwrap().into_raw();
        let name_ptr = CString::new("from_str").unwrap().into_raw();
        let cmd_ptr: *mut bash::Command;

        unsafe {
            bash::with_input_from_string(cmd_str, name_ptr);
            cmd_ptr = match bash::parse_command() {
                0 => bash::copy_command(bash::GLOBAL_COMMAND),
                _ => return Err(Error::new(format!("failed parsing: {}", s))),
            };

            // clean up global command
            bash::dispose_command(bash::GLOBAL_COMMAND);
            bash::GLOBAL_COMMAND = ptr::null_mut();

            // restore parser input source for interactive sessions
            if bash::STARTUP_STATE == 1 {
                bash::with_input_from_stdin();
            }
        }

        Ok(Command { ptr: cmd_ptr })
    }
}
