use std::ffi::CString;
use std::path::Path;

use bitflags::bitflags;
use once_cell::sync::Lazy;

use crate::{bash, Error, Result};

bitflags! {
    /// Flag values used with source::string() for altering string evaluation.
    pub struct Eval: u32 {
        const NONE = 0;
        const NON_INTERACTIVE = bash::SEVAL_NONINT;
        const INTERACTIVE = bash::SEVAL_INTERACT;
        const NO_HISTORY = bash::SEVAL_NOHIST;
        const NO_FREE = bash::SEVAL_NOFREE;
        const RESET_LINE = bash::SEVAL_RESETLINE;
        const PARSE_ONLY = bash::SEVAL_PARSEONLY;
        const NO_LONG_JUMP = bash::SEVAL_NOLONGJMP;
        const FUNCDEF = bash::SEVAL_FUNCDEF;
        const ONE_COMMAND = bash::SEVAL_ONECMD;
        const NO_HISTORY_EXPANSION = bash::SEVAL_NOHISTEXP;
    }
}

static FILE_STR: Lazy<CString> = Lazy::new(|| CString::new("scallop::source::string").unwrap());

pub fn string<S: AsRef<str>>(s: S) -> Result<i32> {
    let ret: i32;
    let file_ptr = FILE_STR.as_ptr();
    let s = s.as_ref();
    let c_str = CString::new(s).unwrap();
    let str_ptr = c_str.as_ptr() as *mut _;

    unsafe {
        ret = bash::evalstring(str_ptr, file_ptr, Eval::NO_FREE.bits() as i32);
    }

    match ret {
        0 => Ok(0),
        _ => return Err(Error::new(format!("failed sourcing string: {}", s))),
    }
}

pub fn file<P: AsRef<Path>>(path: &P) -> Result<i32> {
    let ret: i32;
    let path = path.as_ref();
    let c_str = CString::new(path.to_str().unwrap()).unwrap();
    let str_ptr = c_str.as_ptr();

    unsafe {
        ret = bash::source_file(str_ptr, 0);
    }

    match ret {
        0 => Ok(0),
        _ => return Err(Error::new(format!("failed sourcing file: {:?}", path))),
    }
}
