use std::collections::HashMap;
use std::ffi::CString;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::os::raw::{c_char, c_int};
use std::{mem, ptr};

use once_cell::sync::Lazy;

use crate::bash;
use crate::traits::IntoVec;
use crate::{current_command, Result};

pub mod profile;

// pkgcraft specific builtins
#[cfg(feature = "pkg")]
pub mod pkg;
// export pkgcraft builtins
#[cfg(feature = "pkg")]
pub use pkg::*;

type BuiltinFn = fn(&[&str]) -> Result<i32>;

#[derive(Clone, Copy)]
pub struct Builtin {
    pub name: &'static str,
    pub func: BuiltinFn,
    pub help: &'static str,
    pub usage: &'static str,
    pub exit_on_error: bool,
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
    pub fn run(self, args: &[&str]) -> Result<i32> {
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
            function: run,
            flags: 1,
            long_doc,
            short_doc,
            handle: ptr::null_mut(),
        }
    }
}

static BUILTINS: Lazy<HashMap<&'static str, &'static Builtin>> = Lazy::new(|| {
    let mut builtins: Vec<&Builtin> = vec![&profile::BUILTIN];
    if cfg!(feature = "pkgcraft") {
        builtins.extend([
            &pkg::die::BUILTIN,
            &pkg::export_functions::BUILTIN,
            &pkg::nonfatal::BUILTIN,
            &pkg::has::BUILTIN,
            &pkg::hasv::BUILTIN,
            &pkg::ver_cut::BUILTIN,
            &pkg::ver_rs::BUILTIN,
            &pkg::ver_test::BUILTIN,
        ]);
    }

    builtins.iter().map(|&b| (b.name, b)).collect()
});

/// Builtin function wrapper converting between rust and C types.
///
/// # Safety
/// This should only be used when registering an external builtin.
#[no_mangle]
unsafe extern "C" fn run(list: *mut bash::WordList) -> c_int {
    // get the current running command name
    let cmd = current_command().expect("failed getting current command");
    // find its matching rust function and execute it
    let builtin = BUILTINS
        .get(cmd)
        .unwrap_or_else(|| panic!("unknown builtin: {}", cmd));
    let args = unsafe { list.into_vec().unwrap() };

    match builtin.run(args.as_slice()) {
        Ok(ret) => ret,
        Err(e) => {
            eprintln!("{}: error: {}", cmd, e);
            match builtin.exit_on_error {
                false => -1,
                // TODO: this should probably call the exit builtin
                true => std::process::exit(1),
            }
        }
    }
}
