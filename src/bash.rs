use std::collections::HashSet;

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
