use std::ffi::CString;
use std::ptr;
use std::str::FromStr;

use once_cell::sync::Lazy;

use crate::bash;
use crate::Error;

#[derive(Debug)]
pub struct Command {
    ptr: *mut bash::Command,
}

impl Command {
    pub fn new<S: AsRef<str>>(s: S, flags: Option<i32>) -> crate::Result<Self> {
        let cmd = Command::from_str(s.as_ref())?;
        if let Some(flags) = flags {
            unsafe { (*cmd.ptr).flags |= flags };
        }
        Ok(cmd)
    }

    pub fn execute(&self) -> crate::Result<i32> {
        match unsafe { bash::execute_command(self.ptr) } {
            0 => Ok(0),
            n => Err(Error::new(format!("command failed: {}", n))),
        }
    }
}

impl Drop for Command {
    #[inline]
    fn drop(&mut self) {
        unsafe { bash::dispose_command(self.ptr) };
    }
}

static COMMAND_MARKER: Lazy<CString> = Lazy::new(|| CString::new("Command::from_str").unwrap());

impl FromStr for Command {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cmd_str = CString::new(s).unwrap();
        let cmd_ptr = cmd_str.as_ptr() as *mut _;
        let name_ptr = COMMAND_MARKER.as_ptr();
        let cmd: *mut bash::Command;

        unsafe {
            // save input stream
            bash::push_stream(1);

            // parse command from string
            bash::with_input_from_string(cmd_ptr, name_ptr);
            cmd = match bash::parse_command() {
                0 => bash::copy_command(bash::GLOBAL_COMMAND),
                _ => return Err(Error::new(format!("failed parsing: {}", s))),
            };

            // clean up global command
            bash::dispose_command(bash::GLOBAL_COMMAND);
            bash::GLOBAL_COMMAND = ptr::null_mut();

            // restore input stream
            bash::pop_stream();
        }

        Ok(Command { ptr: cmd })
    }
}
