#![warn(unreachable_pub)]
#![deny(unsafe_op_in_unsafe_fn)]

use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::path::Path;
use std::{env, mem, process, ptr};

pub mod bash;
pub mod builtins;
pub mod command;
pub mod error;
pub mod functions;
pub mod source;
pub mod traits;
pub mod variables;

pub use self::error::{Error, Result};

pub struct Shell {
    _name: CString,
}

impl Shell {
    /// Create and initialize the shell for general use.
    pub fn new<S: AsRef<str>>(name: S, builtins: Option<Vec<&'static builtins::Builtin>>) -> Self {
        if let Some(builtins) = builtins {
            builtins::register(builtins);
        }

        // initialize bash for library usage
        let name = CString::new(name.as_ref()).unwrap();
        unsafe {
            bash::shell_name = name.as_ptr() as *mut _;
            bash::lib_init(Some(error::bash_error), Some(error::bash_warning));
        }

        Shell { _name: name }
    }

    /// Reset the shell back to a pristine state.
    #[inline]
    pub fn reset(&mut self) {
        unsafe { bash::lib_reset() };
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
            .map(|(key, val)| format!("{}={}", key, val))
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
    #[inline]
    fn drop(&mut self) {
        self.reset()
    }
}
