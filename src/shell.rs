use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::path::Path;
use std::{env, mem, process, ptr};

use nix::{
    sys::signal,
    unistd::{getpid, Pid},
};
use once_cell::sync::{Lazy, OnceCell};

use crate::{bash, builtins, error, source, Error, Result};

#[derive(Debug)]
pub struct Shell {
    _name: CString,
}

impl Shell {
    /// Initialize the shell for library use.
    pub fn init() {
        SHELL
            .set(Shell::_init())
            .expect("failed initializing shell");
    }

    fn _init() -> Self {
        let shm: *mut c_char;
        let name = CString::new("scallop").unwrap();
        unsafe {
            bash::set_shell_name(name.as_ptr() as *mut _);
            bash::lib_error_handlers(Some(error::bash_error), Some(error::bash_warning));
            shm = bash::lib_init(4096) as *mut c_char;
            if shm.is_null() {
                panic!("failed initializing bash");
            }
            SHM.set(shm).expect("shell already initialized");
        }

        // force main pid initialization
        Lazy::force(&PID);

        Shell { _name: name }
    }

    /// Create an error message in shared memory.
    pub(crate) fn set_shm_error(msg: &str) {
        let msg = CString::new(msg).unwrap();
        let data = msg.into_bytes_with_nul();
        unsafe {
            let addr = *SHM.get().expect("uninitialized shell");
            ptr::copy_nonoverlapping(data.as_ptr(), addr as *mut u8, 4096);
        }
    }

    /// Raise an error from shared memory if one exists.
    pub(crate) fn raise_shm_error() {
        unsafe {
            // Note that this is ignored if the shell wasn't initialized, e.g. using scallop as a
            // shared library for dynamic bash builtins.
            if let Some(ptr) = SHM.get() {
                error::bash_error(*ptr);
                ptr::write_bytes(*ptr, b'\0', 4096);
            }
        }
    }

    /// Reset the shell back to a pristine state.
    pub fn reset() {
        unsafe { bash::lib_reset() };
    }

    /// Return the main process value.
    pub fn pid() -> &'static Pid {
        &PID
    }

    /// Start an interactive shell session.
    pub fn interactive() {
        let mut argv_ptrs: Vec<_> = env::args()
            .map(|s| CString::new(s).unwrap().into_raw())
            .collect();
        let argc: c_int = argv_ptrs.len().try_into().unwrap();
        argv_ptrs.push(ptr::null_mut());
        argv_ptrs.shrink_to_fit();
        let argv = argv_ptrs.as_mut_ptr();
        mem::forget(argv_ptrs);

        let mut env_ptrs: Vec<_> = env::vars()
            .map(|(key, val)| format!("{key}={val}"))
            .map(|s| CString::new(s).unwrap().into_raw())
            .collect();
        env_ptrs.push(ptr::null_mut());
        env_ptrs.shrink_to_fit();
        let env = env_ptrs.as_mut_ptr();
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

static PID: Lazy<Pid> = Lazy::new(getpid);
static SHELL: OnceCell<Shell> = OnceCell::new();
static mut SHM: OnceCell<*mut c_char> = OnceCell::new();

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
    use super::*;
    use crate::variables::*;

    #[test]
    fn test_reset() {
        Shell::init();
        bind("VAR", "1", None, None).unwrap();
        assert_eq!(string_value("VAR").unwrap(), "1");
        Shell::reset();
        assert_eq!(string_value("VAR"), None);
    }
}
