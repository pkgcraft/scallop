use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::os::unix::io::RawFd;
use std::path::Path;
use std::{env, mem, process, ptr};

use nix::{
    fcntl::OFlag,
    libc::snprintf,
    sys::mman::{mmap, msync, shm_open, shm_unlink, MapFlags, MsFlags, ProtFlags},
    sys::{signal, stat::Mode},
    unistd::{ftruncate, getpid, Pid},
};
use once_cell::sync::Lazy;

use crate::{bash, builtins, error, source, Error, Result};

#[derive(Debug)]
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
            bash::set_shell_name(name.as_ptr() as *mut _);
            bash::lib_error_handlers(Some(error::bash_error), Some(error::bash_warning));
            bash::lib_init();
        }

        // forcibly create shm file and store global pid
        Lazy::force(&PID);
        Lazy::force(&SHM);

        Shell { _name: name }
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
    fn drop(&mut self) {
        if !is_subshell() {
            self.reset();
            // ignore unlinking errors
            let _ = shm_unlink(SHM.name.as_str());
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

#[derive(Debug)]
struct Shm {
    name: String,
    size: usize,
    fd: RawFd,
}

impl Default for Shm {
    fn default() -> Self {
        let name = format!("/scallop-{}", *PID);
        let size: usize = 4096;
        let flag = OFlag::O_CREAT | OFlag::O_TRUNC | OFlag::O_RDWR;
        let mode = Mode::S_IWUSR | Mode::S_IRUSR;
        let fd = shm_open(name.as_str(), flag, mode).expect("failed opening shared memory");
        ftruncate(fd, size as i64).expect("failed truncating shared memory");
        Shm { name, size, fd }
    }
}

static SHM: Lazy<Shm> = Lazy::new(Default::default);

/// Inject an error into bash.
pub fn error<S: AsRef<str>>(err: S) -> Result<()> {
    let err = err.as_ref();
    if err.len() > SHM.size - 1 {
        return Err(Error::Base(format!(
            "error message larger than {} bytes",
            SHM.size - 1,
        )));
    }

    unsafe {
        let ptr = mmap(
            ptr::null_mut(),
            SHM.size,
            ProtFlags::PROT_WRITE,
            MapFlags::MAP_SHARED,
            SHM.fd,
            0,
        )
        .map_err(|e| Error::Base(format!("failed mmap shared memory: {}", e)))?;
        let data = CString::new(err).unwrap();
        snprintf(ptr as *mut _, SHM.size, data.as_ptr());
        msync(ptr as *mut _, SHM.size, MsFlags::MS_SYNC)
            .map_err(|e| Error::Base(format!("failed msync shared memory: {}", e)))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Shell;
    use crate::variables::*;

    use rusty_fork::rusty_fork_test;

    rusty_fork_test! {
        #[test]
        fn reset() {
            let sh = Shell::new("sh", None);
            bind("VAR", "1", None, None).unwrap();
            assert_eq!(string_value("VAR").unwrap(), "1");
            sh.reset();
            assert_eq!(string_value("VAR"), None);
        }
    }
}
