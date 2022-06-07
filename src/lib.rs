#![warn(unreachable_pub)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod bash;
pub mod builtins;
pub mod command;
pub mod error;
pub mod functions;
pub mod shell;
pub mod source;
pub mod variables;

pub use self::error::{Error, Result};
pub use shell::Shell;
