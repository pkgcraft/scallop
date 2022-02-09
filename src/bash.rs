use std::collections::HashSet;

use crate::variables::string_value;

mod internal;

pub(crate) use internal::*;
// export Builtin for scallop-builtins to use
pub use internal::Builtin;

/// Return the set of currently enabled shell options.
pub fn shell_opts() -> HashSet<String> {
    let opts = string_value("BASHOPTS").unwrap();
    opts.split(':').map(|s| s.to_string()).collect()
}
