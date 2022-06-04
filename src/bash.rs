use std::collections::HashSet;
use std::ffi::CStr;

use once_cell::sync::Lazy;

use crate::variables::string_value;

mod internal;

// export bash API for external usage
pub use internal::*;

/// Return the set of enabled shell options used with the `set` builtin.
pub fn set_opts() -> HashSet<String> {
    let opts = string_value("SHELLOPTS").unwrap();
    opts.split(':').map(|s| s.to_string()).collect()
}

/// Return the set of enabled shell options used with `shopt` builtin.
pub fn shopt_opts() -> HashSet<String> {
    let opts = string_value("BASHOPTS").unwrap();
    opts.split(':').map(|s| s.to_string()).collect()
}

/// Return the set of all shell options used with the `set` builtin.
pub static SET_OPTS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let opt_ptrs = unsafe { get_set_options() };
    let mut opts = HashSet::new();
    let mut i = 0;
    unsafe {
        while let Some(p) = (*opt_ptrs.offset(i)).as_ref() {
            opts.insert(CStr::from_ptr(p).to_str().unwrap());
            i += 1;
        }
    }
    opts
});

/// Return the set of all shell options used with the `shopt` builtin.
pub static SHOPT_OPTS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let opt_ptrs = unsafe { get_shopt_options() };
    let mut opts = HashSet::new();
    let mut i = 0;
    unsafe {
        while let Some(p) = (*opt_ptrs.offset(i)).as_ref() {
            opts.insert(CStr::from_ptr(p).to_str().unwrap());
            i += 1;
        }
    }
    opts
});
