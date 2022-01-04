#![warn(unreachable_pub)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod bash;
pub mod error;

pub use self::error::{Error, Result};
