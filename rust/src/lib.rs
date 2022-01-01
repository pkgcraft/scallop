#![warn(unreachable_pub)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod bash;
pub mod error;

pub use self::error::{Error, Result};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
