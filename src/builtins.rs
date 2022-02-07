use std::collections::HashMap;
use std::ffi::CString;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::os::raw::{c_char, c_int};
use std::sync::RwLock;
use std::{mem, ptr};

use bitflags::bitflags;
use once_cell::sync::Lazy;

use crate::traits::IntoVec;
use crate::{bash, command, Error, Result};

pub mod command_not_found_handle;
pub mod profile;

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
            function: run_builtin,
            flags: Attr::ENABLED.bits() as i32,
            long_doc,
            short_doc,
            handle: ptr::null_mut(),
        }
    }
}

// Enable or disable a given list of builtins.
fn toggle_status(builtins: &[&str], enable: bool) -> Result<()> {
    let mut unknown: Vec<&str> = vec![];
    for name in builtins {
        let builtin_name = CString::new(*name).unwrap();
        let builtin_ptr = builtin_name.as_ptr() as *mut _;
        match unsafe { bash::builtin_address_internal(builtin_ptr, 1).as_mut() } {
            Some(b) => match enable {
                true => b.flags |= Attr::ENABLED.bits() as i32,
                false => b.flags &= !Attr::ENABLED.bits() as i32,
            },
            None => unknown.push(name),
        }
    }

    match unknown.is_empty() {
        true => Ok(()),
        false => Err(Error::Base(format!(
            "unknown builtins: {}",
            unknown.join(", ")
        ))),
    }
}

/// Disable a given list of builtins by name.
#[inline]
pub fn disable(builtins: &[&str]) -> Result<()> {
    toggle_status(builtins, false)
}

/// Enable a given list of builtins by name.
#[inline]
pub fn enable(builtins: &[&str]) -> Result<()> {
    toggle_status(builtins, true)
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
    Failure,
}

impl From<ExecStatus> for i32 {
    fn from(exec: ExecStatus) -> i32 {
        match exec {
            ExecStatus::Success => bash::EXECUTION_SUCCESS as i32,
            ExecStatus::Failure => bash::EXECUTION_FAILURE as i32,
        }
    }
}

impl From<&ExecStatus> for bool {
    fn from(exec: &ExecStatus) -> bool {
        match exec {
            ExecStatus::Success => true,
            ExecStatus::Failure => false,
        }
    }
}

impl From<bool> for ExecStatus {
    fn from(value: bool) -> ExecStatus {
        match value {
            true => ExecStatus::Success,
            false => ExecStatus::Failure,
        }
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
        .unwrap_or_else(|| panic!("unknown builtin: {}", cmd));
    let args = list.into_vec();

    match builtin.run(args.as_slice()) {
        Ok(ret) => ret as i32,
        Err(e) => {
            match e {
                Error::Builtin(_) => eprintln!("{}: error: {}", cmd, e),
                _ => eprintln!("{}", e),
            }
            ExecStatus::Failure as i32
        }
    }
}
