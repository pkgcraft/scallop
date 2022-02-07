use std::ffi::{CStr, CString};
use std::ptr;
use std::str::FromStr;

use bitflags::bitflags;
use once_cell::sync::Lazy;

use crate::bash;
use crate::Error;

bitflags! {
    /// Flag values used with commands.
    pub struct Flags: u32 {
        const NONE = 0;
        const WANT_SUBSHELL = bash::CMD_WANT_SUBSHELL;
        const FORCE_SUBSHELL = bash::CMD_FORCE_SUBSHELL;
        const INVERT_RETURN = bash::CMD_INVERT_RETURN;
        const IGNORE_RETURN = bash::CMD_IGNORE_RETURN;
        const NO_FUNCTIONS = bash::CMD_NO_FUNCTIONS;
        const INHIBIT_EXPANSION = bash::CMD_INHIBIT_EXPANSION;
        const NO_FORK = bash::CMD_NO_FORK;
    }
}

#[derive(Debug)]
pub struct Command {
    ptr: *mut bash::Command,
}

impl Command {
    pub fn new<S: AsRef<str>>(s: S, flags: Option<Flags>) -> crate::Result<Self> {
        let cmd = Command::from_str(s.as_ref())?;
        if let Some(flags) = flags {
            unsafe { (*cmd.ptr).flags |= flags.bits() as i32 };
        }
        Ok(cmd)
    }

    pub fn execute(&self) -> crate::Result<()> {
        match unsafe { bash::execute_command(self.ptr) } {
            0 => Ok(()),
            _ => Err(Error::Base("command failed".into())),
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
                _ => return Err(Error::Base(format!("failed parsing: {}", s))),
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

/// Get the currently running command name if one exists.
#[inline]
pub fn current<'a>() -> Option<&'a str> {
    let cmd_ptr = unsafe { bash::CURRENT_COMMAND.as_ref() };
    cmd_ptr.map(|s| unsafe { CStr::from_ptr(s).to_str().unwrap() })
}
