use std::ffi::CString;

use once_cell::sync::Lazy;

use crate::{bash, Error, Result};

static FILE_STR: Lazy<CString> = Lazy::new(|| CString::new("source::string").unwrap());

pub fn string<S: AsRef<str>>(s: S) -> Result<i32> {
    let ret: i32;
    let file_ptr = FILE_STR.as_ptr();
    let s = s.as_ref();
    let c_str = CString::new(s).unwrap();
    let str_ptr = c_str.as_ptr() as *mut _;

    unsafe {
        ret = bash::evalstring(str_ptr, file_ptr, bash::SEVAL_NOFREE as i32);
    }

    match ret {
        0 => Ok(0),
        _ => return Err(Error::new(format!("failed sourcing string: {}", s))),
    }
}
