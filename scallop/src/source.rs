use std::ffi::CString;

use bitflags::bitflags;
use once_cell::sync::Lazy;

use crate::{bash, Error, Result};

bitflags! {
    /// Flag values used with parse_and_execute() and related commands.
    struct Eval: u32 {
        const NON_INTERACTIVE = bash::SEVAL_NONINT;
        const INTERACTIVE = bash::SEVAL_INTERACT;
        const NO_HISTORY = bash::SEVAL_NOHIST;
        const NO_STR_FREE = bash::SEVAL_NOFREE;
        const RESET_LINE = bash::SEVAL_RESETLINE;
        const PARSE_ONLY = bash::SEVAL_PARSEONLY;
        const NO_LONG_JUMP = bash::SEVAL_NOLONGJMP;
        const FUNCDEF = bash::SEVAL_FUNCDEF;
        const ONE_COMMAND = bash::SEVAL_ONECMD;
        const NO_HISTORY_EXPANSION = bash::SEVAL_NOHISTEXP;
    }
}

static FILE_STR: Lazy<CString> = Lazy::new(|| CString::new("source::string").unwrap());

pub fn string<S: AsRef<str>>(s: S) -> Result<i32> {
    let ret: i32;
    let file_ptr = FILE_STR.as_ptr();
    let s = s.as_ref();
    let c_str = CString::new(s).unwrap();
    let str_ptr = c_str.as_ptr() as *mut _;

    unsafe {
        ret = bash::evalstring(str_ptr, file_ptr, Eval::NO_STR_FREE.bits() as i32);
    }

    match ret {
        0 => Ok(0),
        _ => return Err(Error::new(format!("failed sourcing string: {}", s))),
    }
}
