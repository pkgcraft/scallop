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

use crate::shell::{is_subshell, kill, Shell};
use crate::traits::*;
use crate::{bash, command, Error, Result};

mod _bash;
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

pub mod set {
    use super::*;

    pub fn enable<S: AsRef<str>>(opts: &[S]) -> Result<ExecStatus> {
        let args: Vec<_> = ["-o"]
            .into_iter()
            .chain(opts.iter().map(|s| s.as_ref()))
            .collect();
        set(&args)
    }

    pub fn disable<S: AsRef<str>>(opts: &[S]) -> Result<ExecStatus> {
        let args: Vec<_> = ["+o"]
            .into_iter()
            .chain(opts.iter().map(|s| s.as_ref()))
            .collect();
        set(&args)
    }
}

pub mod shopt {
    use super::*;

    pub fn enable<S: AsRef<str>>(opts: &[S]) -> Result<ExecStatus> {
        let args: Vec<_> = ["-s"]
            .into_iter()
            .chain(opts.iter().map(|s| s.as_ref()))
            .collect();
        shopt(&args)
    }

    pub fn disable<S: AsRef<str>>(opts: &[S]) -> Result<ExecStatus> {
        let args: Vec<_> = ["-u"]
            .into_iter()
            .chain(opts.iter().map(|s| s.as_ref()))
            .collect();
        shopt(&args)
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

impl fmt::Display for Builtin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
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
        let name = name_str.as_ptr() as *mut _;
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

type BuiltinFnPtr = unsafe extern "C" fn(list: *mut bash::WordList) -> c_int;

// Dynamically-loaded builtins require non-null function pointers since wrapping the function
// pointer field member in Option<fn> causes bash to segfault.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DynBuiltin {
    name: *const c_char,
    function: BuiltinFnPtr,
    flags: c_int,
    long_doc: *const *mut c_char,
    short_doc: *const c_char,
    handle: *mut c_char,
}

/// Convert a Builtin to the dynamically-loaded builtin format.
impl From<Builtin> for DynBuiltin {
    fn from(b: Builtin) -> Self {
        // first convert to the Option wrapped variant
        let b: bash::Builtin = b.into();
        // then convert to the dynamically-loaded variant
        DynBuiltin {
            name: b.name,
            function: b.function.unwrap(),
            flags: b.flags,
            long_doc: b.long_doc,
            short_doc: b.short_doc,
            handle: b.handle,
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

/// Toggle shell options, automatically reverting their status when leaving scope.
#[derive(Debug, Default)]
pub struct ScopedOptions {
    shopt_enabled: Vec<String>,
    shopt_disabled: Vec<String>,
    set_enabled: Vec<String>,
    set_disabled: Vec<String>,
}

impl ScopedOptions {
    /// Enable shell options.
    pub fn enable<'a, I>(&mut self, options: I) -> Result<()>
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut unknown = vec![];
        let enabled_shopt = bash::shopt_opts();
        let enabled_set = bash::set_opts();

        for opt in options {
            match (bash::SET_OPTS.contains(opt), bash::SHOPT_OPTS.contains(opt)) {
                (true, false) if !enabled_set.contains(opt) => {
                    set::enable(&[opt])?;
                    self.set_enabled.push(opt.into());
                }
                (false, true) if !enabled_shopt.contains(opt) => {
                    shopt::enable(&[opt])?;
                    self.shopt_enabled.push(opt.into());
                }
                _ => unknown.push(opt),
            }
        }

        match unknown.is_empty() {
            true => Ok(()),
            false => Err(Error::Base(format!("unknown options: {}", unknown.join(", ")))),
        }
    }

    /// Disable shell options.
    pub fn disable<'a, I>(&mut self, options: I) -> Result<()>
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut unknown = vec![];
        let enabled_shopt = bash::shopt_opts();
        let enabled_set = bash::set_opts();

        for opt in options {
            match (bash::SET_OPTS.contains(opt), bash::SHOPT_OPTS.contains(opt)) {
                (true, false) if enabled_set.contains(opt) => {
                    set::disable(&[opt])?;
                    self.set_disabled.push(opt.into());
                }
                (false, true) if enabled_shopt.contains(opt) => {
                    shopt::disable(&[opt])?;
                    self.shopt_disabled.push(opt.into());
                }
                _ => unknown.push(opt),
            }
        }

        match unknown.is_empty() {
            true => Ok(()),
            false => Err(Error::Base(format!("unknown options: {}", unknown.join(", ")))),
        }
    }
}

impl Drop for ScopedOptions {
    fn drop(&mut self) {
        if !self.shopt_enabled.is_empty() {
            shopt::disable(&self.shopt_enabled).expect("failed unsetting shopt options");
        }
        if !self.shopt_disabled.is_empty() {
            shopt::enable(&self.shopt_disabled).expect("failed setting shopt options");
        }
        if !self.set_enabled.is_empty() {
            set::disable(&self.set_enabled).expect("failed unsetting set options");
        }
        if !self.set_disabled.is_empty() {
            set::enable(&self.set_disabled).expect("failed setting set options");
        }
    }
}

/// Register builtins into the internal list for use.
pub fn register<I>(builtins: I)
where
    I: IntoIterator<Item = &'static Builtin> + Copy,
{
    unsafe {
        // convert builtins into pointers
        let mut builtin_ptrs: Vec<_> = builtins
            .into_iter()
            .map(|b| Box::into_raw(Box::new((*b).into())))
            .collect();

        // add builtins to bash's internal list
        let len: i32 = builtin_ptrs.len().try_into().unwrap();
        bash::register_builtins(builtin_ptrs.as_mut_ptr(), len);

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
    Shell::set_shm_error(err);

    // TODO: send SIGTERM to background jobs (use jobs builtin)
    match is_subshell() {
        true => {
            kill(signal::Signal::SIGUSR1)?;
            process::exit(2);
        }
        false => Ok(ExecStatus::Error),
    }
}

/// Get the builtin matching the current running command if it exists.
pub fn running_builtin() -> Option<&'static Builtin> {
    command::current()
        .and_then(|c| BUILTINS.try_read().ok().map(|b| b.get(c).cloned()))
        .flatten()
}

/// Builtin function wrapper converting between rust and C types.
#[no_mangle]
extern "C" fn run_builtin(list: *mut bash::WordList) -> c_int {
    let builtin = running_builtin().expect("unknown builtin");
    let cmd = builtin.name;
    let words = list.into_words(false);
    let args: Vec<_> = words.into_iter().collect();

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

    #[test]
    fn toggle_status() {
        Shell::init();
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
        Shell::init();
        let (set, unset) = ("autocd", "sourcepath");

        assert!(!bash::shopt_opts().contains(set));
        assert!(bash::shopt_opts().contains(unset));
        {
            let mut opts = ScopedOptions::default();
            opts.enable([set]).unwrap();
            opts.disable([unset]).unwrap();
            assert!(bash::shopt_opts().contains(set));
            assert!(!bash::shopt_opts().contains(unset));
        }
        assert!(!bash::shopt_opts().contains(set));
        assert!(bash::shopt_opts().contains(unset));

        // toggle options in separate scope from ScopedOptions creation
        {
            let mut opts = ScopedOptions::default();
            // options aren't toggled
            assert!(!bash::shopt_opts().contains(set));
            assert!(bash::shopt_opts().contains(unset));
            {
                opts.enable([set]).unwrap();
                opts.disable([unset]).unwrap();
                // options are toggled
                assert!(bash::shopt_opts().contains(set));
                assert!(!bash::shopt_opts().contains(unset));
            }
            // options are still toggled
            assert!(bash::shopt_opts().contains(set));
            assert!(!bash::shopt_opts().contains(unset));
        }
        // options have been reverted
        assert!(!bash::shopt_opts().contains(set));
        assert!(bash::shopt_opts().contains(unset));
    }
}
