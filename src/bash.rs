use std::collections::HashSet;
use std::ffi::CStr;

use once_cell::sync::Lazy;

use crate::variables::string_value;

mod internal;

// export bash API for external usage
pub use internal::*;

/// Return the set of enabled shell options used with the `set` builtin.
pub fn set_opts() -> HashSet<String> {
    let opts = string_value("SHELLOPTS").unwrap_or_default();
    opts.split(':').map(|s| s.to_string()).collect()
}

/// Return the set of enabled shell options used with `shopt` builtin.
pub fn shopt_opts() -> HashSet<String> {
    let opts = string_value("BASHOPTS").unwrap_or_default();
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

#[cfg(test)]
mod tests {
    use crate::builtins::{set, shopt};

    use super::*;

    #[test]
    fn test_set_opts() {
        // noexec option exists
        assert!(SET_OPTS.contains("noexec"));
        // but isn't currently enabled
        assert!(!set_opts().contains("noexec"));
        // enable it
        set::enable(&["noexec"]).unwrap();
        // and now it's currently enabled
        assert!(set_opts().contains("noexec"));
        // disable it
        set::disable(&["noexec"]).unwrap();
        assert!(!set_opts().contains("noexec"));
    }

    #[test]
    fn test_shopt_opts() {
        // autocd option exists
        assert!(SHOPT_OPTS.contains("autocd"));
        // but isn't currently enabled
        assert!(!shopt_opts().contains("autocd"));
        // enable it
        shopt::enable(&["autocd"]).unwrap();
        // and now it's currently enabled
        assert!(shopt_opts().contains("autocd"));
        // disable it
        shopt::disable(&["autocd"]).unwrap();
        assert!(!shopt_opts().contains("autocd"));
    }
}
