use std::collections::{HashMap, HashSet};
use std::ffi::{CStr, CString};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::os::raw::{c_char, c_int};
use std::process::ExitStatus;
use std::sync::RwLock;
use std::{mem, process, ptr};

use bitflags::bitflags;
use nix::sys::signal;
use once_cell::sync::Lazy;

use crate::shell::{is_subshell, kill};
use crate::traits::IntoVec;
use crate::{bash, command, Error, Result};

pub mod _bash;
pub mod command_not_found_handle;
pub mod profile;

// export native bash builtins
pub use _bash::*;

pub type BuiltinFn = fn(&[&str]) -> Result<ExecStatus>;

bitflags! {
    /// Flag values describing builtin attributes.
    pub struct Attr: u32 {
        const NONE = 0;
        const ENABLED = bash::BUILTIN_ENABLED;
        const STATIC = bash::STATIC_BUILTIN;
        const ASSIGNMENT = bash::ASSIGNMENT_BUILTIN;
        const LOCALVAR = bash::LOCALVAR_BUILTIN;
    }
}

#[derive(Clone, Copy)]
pub struct Builtin {
    pub name: &'static str,
    pub func: BuiltinFn,
    pub help: &'static str,
    pub usage: &'static str,
}

impl fmt::Debug for Builtin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Builtin").field("name", &self.name).finish()
    }
}

impl PartialEq for Builtin {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Builtin {}

impl Hash for Builtin {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Builtin {
    #[inline]
    pub fn run(&self, args: &[&str]) -> Result<ExecStatus> {
        (self.func)(args)
    }
}

/// Convert a Builtin to its C equivalent.
impl From<Builtin> for bash::Builtin {
    fn from(builtin: Builtin) -> bash::Builtin {
        let name_str = CString::new(builtin.name).unwrap();
        let name = name_str.as_ptr();
        mem::forget(name_str);

        let short_doc_str = CString::new(builtin.usage).unwrap();
        let short_doc = short_doc_str.as_ptr();
        mem::forget(short_doc_str);

        let mut long_doc_ptr: Vec<*mut c_char> = builtin
            .help
            .split('\n')
            .map(|s| CString::new(s).unwrap().into_raw())
            .collect();
        long_doc_ptr.push(ptr::null_mut());
        let long_doc = long_doc_ptr.as_ptr();
        mem::forget(long_doc_ptr);

        bash::Builtin {
            name,
            function: Some(run_builtin),
            flags: Attr::ENABLED.bits() as i32,
            long_doc,
            short_doc,
            handle: ptr::null_mut(),
        }
    }
}

// Enable or disable a given list of builtins.
fn toggle_status<S: AsRef<str>>(builtins: &[S], enable: bool) -> Result<Vec<&str>> {
    let mut unknown = Vec::<&str>::new();
    let mut toggled = Vec::<&str>::new();
    for name in builtins {
        let name = name.as_ref();
        let builtin_name = CString::new(name).unwrap();
        let builtin_ptr = builtin_name.as_ptr() as *mut _;
        match unsafe { bash::builtin_address_internal(builtin_ptr, 1).as_mut() } {
            Some(b) => {
                let enabled = (b.flags & Attr::ENABLED.bits() as i32) == 1;
                if enabled != enable {
                    toggled.push(name);
                    match enable {
                        true => b.flags |= Attr::ENABLED.bits() as i32,
                        false => b.flags &= !Attr::ENABLED.bits() as i32,
                    }
                }
            }
            None => unknown.push(name),
        }
    }

    match unknown.is_empty() {
        true => Ok(toggled),
        false => Err(Error::Base(format!("unknown builtins: {}", unknown.join(", ")))),
    }
}

/// Disable a given list of builtins by name.
#[inline]
pub fn disable<S: AsRef<str>>(builtins: &[S]) -> Result<Vec<&str>> {
    toggle_status(builtins, false)
}

/// Enable a given list of builtins by name.
#[inline]
pub fn enable<S: AsRef<str>>(builtins: &[S]) -> Result<Vec<&str>> {
    toggle_status(builtins, true)
}

/// Get the sets of enabled and disabled shell builtins.
pub fn shell_builtins() -> (HashSet<String>, HashSet<String>) {
    let mut enabled = HashSet::new();
    let mut disabled = HashSet::new();
    unsafe {
        let end = (bash::NUM_SHELL_BUILTINS - 1) as isize;
        for i in 0..end {
            let builtin = *bash::SHELL_BUILTINS.offset(i);
            // builtins with null functions are stubs for reserved keywords
            if builtin.function.is_some() {
                let name = String::from(CStr::from_ptr(builtin.name).to_str().unwrap());
                if (builtin.flags & Attr::ENABLED.bits() as i32) == 1 {
                    enabled.insert(name);
                } else {
                    disabled.insert(name);
                }
            }
        }
    }
    (enabled, disabled)
}

#[derive(Debug)]
pub struct ScopedBuiltins {
    enabled: Vec<String>,
    disabled: Vec<String>,
}

/// Enable/disable builtins, automatically reverting their status when leaving scope.
impl ScopedBuiltins {
    pub fn new<S: AsRef<str>>(builtins: (&[S], &[S])) -> Result<Self> {
        let (add, sub) = builtins;
        Ok(ScopedBuiltins {
            enabled: enable(add)?.into_iter().map(|s| s.into()).collect(),
            disabled: disable(sub)?.into_iter().map(|s| s.into()).collect(),
        })
    }
}

impl Drop for ScopedBuiltins {
    fn drop(&mut self) {
        if !self.enabled.is_empty() {
            disable(&self.enabled).unwrap_or_else(|_| panic!("failed disabling builtins"));
        }
        if !self.disabled.is_empty() {
            enable(&self.disabled).unwrap_or_else(|_| panic!("failed enabling builtins"));
        }
    }
}

#[derive(Debug, Default)]
pub struct ScopedOptions {
    set: Vec<String>,
    unset: Vec<String>,
}

/// Enable/disable shell options, automatically reverting their status when leaving scope.
impl ScopedOptions {
    pub fn new() -> Self {
        ScopedOptions::default()
    }

    pub fn toggle<S: AsRef<str>>(&mut self, set: &[S], unset: &[S]) -> Result<()> {
        let enabled = bash::shell_opts();
        if !set.is_empty() {
            let set: Vec<String> = set
                .iter()
                .map(|s| s.as_ref().to_string())
                .filter(|s| !enabled.contains(s))
                .collect();
            shopt(&["-s"], &set)?;
            self.set.extend(set);
        }
        if !unset.is_empty() {
            let unset: Vec<String> = unset
                .iter()
                .map(|s| s.as_ref().to_string())
                .filter(|s| enabled.contains(s))
                .collect();
            shopt(&["-u"], &unset)?;
            self.unset.extend(unset);
        }
        Ok(())
    }
}

impl Drop for ScopedOptions {
    fn drop(&mut self) {
        if !self.set.is_empty() {
            shopt(&["-u"], &self.set).expect("failed unsetting options");
        }
        if !self.unset.is_empty() {
            shopt(&["-s"], &self.unset).expect("failed setting options");
        }
    }
}

/// Register builtins into the internal list for use.
pub fn register(builtins: Vec<&'static Builtin>) {
    unsafe {
        // convert builtins into pointers
        let mut builtin_ptrs: Vec<*mut bash::Builtin> = builtins
            .iter()
            .map(|&b| Box::into_raw(Box::new((*b).into())))
            .collect();

        // add builtins to bash's internal list
        let builtins_len: i32 = builtins.len().try_into().unwrap();
        bash::register_builtins(builtin_ptrs.as_mut_ptr(), builtins_len);

        // reclaim pointers for proper deallocation
        for b in builtin_ptrs {
            mem::drop(Box::from_raw(b));
        }
    }

    // add builtins to known mapping
    update_run_map(builtins);
}

static BUILTINS: Lazy<RwLock<HashMap<&'static str, &'static Builtin>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Add builtins to known mapping for run() wrapper to work as expected.
pub fn update_run_map<I>(builtins: I)
where
    I: IntoIterator<Item = &'static Builtin>,
{
    let mut builtin_map = BUILTINS.write().unwrap();
    builtin_map.extend(builtins.into_iter().map(|b| (b.name, b)));
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ExecStatus {
    Success,
    Failure(i32),
    Error,
}

impl From<ExecStatus> for i32 {
    fn from(exec: ExecStatus) -> i32 {
        match exec {
            ExecStatus::Success => bash::EXECUTION_SUCCESS as i32,
            ExecStatus::Failure(n) => n,
            ExecStatus::Error => bash::EX_LONGJMP as i32,
        }
    }
}

impl From<i32> for ExecStatus {
    fn from(ret: i32) -> ExecStatus {
        match ret {
            0 => ExecStatus::Success,
            n => ExecStatus::Failure(n),
        }
    }
}

impl From<&ExecStatus> for bool {
    fn from(exec: &ExecStatus) -> bool {
        matches!(exec, ExecStatus::Success)
    }
}

impl From<bool> for ExecStatus {
    fn from(value: bool) -> ExecStatus {
        match value {
            true => ExecStatus::Success,
            false => ExecStatus::Failure(1),
        }
    }
}

impl From<ExitStatus> for ExecStatus {
    fn from(status: ExitStatus) -> ExecStatus {
        match status.success() {
            true => ExecStatus::Success,
            false => ExecStatus::Failure(1),
        }
    }
}

/// Raise an error and reset the current bash process from within a builtin.
pub fn raise_error<S: AsRef<str>>(err: S) -> Result<ExecStatus> {
    let data = CString::new(err.as_ref()).unwrap();
    unsafe { bash::shm_error(data.as_ptr()) };

    // TODO: send SIGTERM to background jobs (use jobs builtin)
    match is_subshell() {
        true => {
            kill(signal::Signal::SIGUSR1)?;
            process::exit(2);
        }
        false => Ok(ExecStatus::Error),
    }
}

/// Builtin function wrapper converting between rust and C types.
#[no_mangle]
extern "C" fn run_builtin(list: *mut bash::WordList) -> c_int {
    // get the current running command name
    let cmd = command::current().expect("failed getting current command");
    // find its matching rust function and execute it
    let builtin_map = BUILTINS.read().unwrap();
    let builtin = builtin_map
        .get(cmd)
        .unwrap_or_else(|| panic!("unknown builtin: {cmd}"));
    let args = list.into_vec();

    match builtin.run(&args) {
        Ok(ret) => i32::from(ret),
        Err(e) => {
            match e {
                Error::Builtin(_) => eprintln!("{cmd}: error: {e}"),
                _ => eprintln!("{e}"),
            }
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Shell;

    use rusty_fork::rusty_fork_test;

    rusty_fork_test! {
        #[test]
        fn toggle_status() {
            let _sh = Shell::new("sh", None);

            // select a builtin to toggle
            let (enabled, disabled) = shell_builtins();
            assert!(!enabled.is_empty());
            let builtin = enabled.iter().next().unwrap();
            assert!(!disabled.contains(builtin));

            // disable the builtin
            disable(&[builtin]).unwrap();
            let (enabled, disabled) = shell_builtins();
            assert!(!enabled.contains(builtin));
            assert!(disabled.contains(builtin));

            // enable the builtin
            enable(&[builtin]).unwrap();
            let (enabled, disabled) = shell_builtins();
            assert!(enabled.contains(builtin));
            assert!(!disabled.contains(builtin));
        }

        #[test]
        fn scoped_options() {
            let _sh = Shell::new("sh", None);
            let (set, unset) = ("autocd", "sourcepath");

            assert!(!bash::shell_opts().contains(set));
            assert!(bash::shell_opts().contains(unset));
            {
                let mut opts = ScopedOptions::new();
                opts.toggle(&[set], &[unset]).unwrap();
                assert!(bash::shell_opts().contains(set));
                assert!(!bash::shell_opts().contains(unset));
            }
            assert!(!bash::shell_opts().contains(set));
            assert!(bash::shell_opts().contains(unset));

            // toggle options in separate scope from ScopedOptions creation
            {
                let mut opts = ScopedOptions::new();
                // options aren't toggled
                assert!(!bash::shell_opts().contains(set));
                assert!(bash::shell_opts().contains(unset));
                {
                    opts.toggle(&[set], &[unset]).unwrap();
                    // options are toggled
                    assert!(bash::shell_opts().contains(set));
                    assert!(!bash::shell_opts().contains(unset));
                }
                // options are still toggled
                assert!(bash::shell_opts().contains(set));
                assert!(!bash::shell_opts().contains(unset));
            }
            // options have been reverted
            assert!(!bash::shell_opts().contains(set));
            assert!(bash::shell_opts().contains(unset));
        }
    }
}
