use std::ffi::CString;
use std::ptr;
use std::str::FromStr;

use crate::bash;
use crate::Error;

#[derive(Debug)]
pub struct Command {
    ptr: *mut bash::Command,
}

impl Command {
    #[inline]
    pub fn new<S: AsRef<str>>(s: S, flags: Option<i32>) -> crate::Result<Self> {
        let cmd = Command::from_str(s.as_ref())?;
        if let Some(flags) = flags {
            unsafe { (*cmd.ptr).flags |= flags };
        }
        Ok(cmd)
    }

    #[inline]
    pub fn execute(&self) {
        unsafe { bash::execute_command(self.ptr) };
    }
}

impl Drop for Command {
    #[inline]
    fn drop(&mut self) {
        unsafe { bash::dispose_command(self.ptr) };
    }
}

impl FromStr for Command {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cmd_ptr = CString::new(s).unwrap().into_raw();
        let name_ptr = CString::new("from_str").unwrap().into_raw();
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

            // deallocate strings
            drop(CString::from_raw(cmd_ptr));
            drop(CString::from_raw(name_ptr));

            // clean up global command
            bash::dispose_command(bash::GLOBAL_COMMAND);
            bash::GLOBAL_COMMAND = ptr::null_mut();

            // restore input stream
            bash::pop_stream();
        }

        Ok(Command { ptr: cmd })
    }
}
