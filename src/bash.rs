use std::collections::HashSet;

use crate::variables::string_value;

mod internal;

// export bash API for external usage
pub use internal::*;

/// Return the set of currently enabled shell options.
pub fn shell_opts() -> HashSet<String> {
    let opts = string_value("BASHOPTS").unwrap();
    opts.split(':').map(|s| s.to_string()).collect()
}
