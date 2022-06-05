use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::path::Path;
use std::{env, mem, process, ptr};

use nix::{
    sys::signal,
    unistd::{getpid, Pid},
};
use once_cell::sync::Lazy;

use crate::{bash, builtins, error, source, Error, Result};

#[derive(Debug)]
pub struct Shell {
    _name: CString,
}

impl Shell {
    /// Create and initialize the shell for general use.
    pub fn new(name: &str) -> Self {
        // initialize bash for library usage
        let name = CString::new(name).unwrap();
        unsafe {
            bash::set_shell_name(name.as_ptr() as *mut _);
            bash::lib_error_handlers(Some(error::bash_error), Some(error::bash_warning));
            if bash::lib_init() != 0 {
                panic!("failed initializing bash");
            }
        }

        // force main pid initialization
        Lazy::force(&PID);

        Shell { _name: name }
    }

    pub fn builtins<I>(&self, builtins: I)
    where
        I: IntoIterator<Item = &'static builtins::Builtin> + Copy,
    {
        builtins::register(builtins);
    }

    /// Reset the shell back to a pristine state.
    #[inline]
    pub fn reset(&self) {
        unsafe { bash::lib_reset() };
    }

    /// Return the main process value.
    pub fn pid(&self) -> &'static Pid {
        &PID
    }

    /// Start an interactive shell session.
    pub fn interactive(&self) {
        let argv_strs: Vec<CString> = env::args().map(|s| CString::new(s).unwrap()).collect();
        let mut argv_ptrs: Vec<*mut c_char> =
            argv_strs.iter().map(|s| s.as_ptr() as *mut _).collect();
        argv_ptrs.push(ptr::null_mut());
        let argv = argv_ptrs.as_ptr() as *mut _;
        let argc: c_int = argv_strs.len().try_into().unwrap();
        mem::forget(argv_strs);
        mem::forget(argv_ptrs);

        let env_strs: Vec<CString> = env::vars()
            .map(|(key, val)| format!("{key}={val}"))
            .map(|s| CString::new(s).unwrap())
            .collect();
        let mut env_ptrs: Vec<*mut c_char> =
            env_strs.iter().map(|s| s.as_ptr() as *mut _).collect();
        env_ptrs.push(ptr::null_mut());
        let env = env_ptrs.as_ptr() as *mut _;
        mem::forget(env_strs);
        mem::forget(env_ptrs);

        let ret: i32;
        unsafe {
            ret = bash::bash_main(argc, argv, env);
        }
        process::exit(ret)
    }

    #[inline]
    pub fn source_file<P: AsRef<Path>>(&mut self, path: &P) -> Result<builtins::ExecStatus> {
        source::file(path)
    }
}

impl Drop for Shell {
    fn drop(&mut self) {
        if !is_subshell() {
            self.reset();
        }
    }
}

static PID: Lazy<Pid> = Lazy::new(getpid);

/// Returns true if currently operating in a subshell, false otherwise.
pub fn is_subshell() -> bool {
    *PID != getpid()
}

/// Send a signal to the main bash process.
pub fn kill<T: Into<Option<signal::Signal>>>(signal: T) -> Result<()> {
    signal::kill(*PID, signal.into()).map_err(|e| Error::Base(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::Shell;
    use crate::variables::*;

    use rusty_fork::rusty_fork_test;

    rusty_fork_test! {
        #[test]
        fn test_reset() {
            let sh = Shell::new("sh");
            bind("VAR", "1", None, None).unwrap();
            assert_eq!(string_value("VAR").unwrap(), "1");
            sh.reset();
            assert_eq!(string_value("VAR"), None);
        }
    }
}
