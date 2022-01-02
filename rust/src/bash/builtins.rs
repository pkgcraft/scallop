use std::collections::HashMap;
use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::ptr;

use once_cell::sync::Lazy;

use crate::Result;
use super::current_command;
use super::command::{IntoVec, WordList};

pub mod has;
pub mod hasv;
#[cfg(feature = "pkgcraft")]
pub mod ver_rs;

type BuiltinFn = fn(&[&str]) -> Result<i32>;

static BUILTINS: Lazy<HashMap<&'static str, (BuiltinFn, &str, &str)>> = Lazy::new(|| {
    let mut builtins: Vec<(&str, (BuiltinFn, &str, &str))> = [
        ("has", (has::has as BuiltinFn, has::SHORT_DOC, has::LONG_DOC)),
        ("hasv", (hasv::hasv as BuiltinFn, hasv::SHORT_DOC, hasv::LONG_DOC)),
    ].iter().cloned().collect();

    if cfg!(feature = "pkgcraft") {
        builtins.extend([
            ("ver_rs", (ver_rs::ver_rs as BuiltinFn, ver_rs::SHORT_DOC, ver_rs::LONG_DOC)),
        ]);
    }

    builtins.iter().cloned().collect()
});


pub type BuiltinFnPtr = unsafe extern "C" fn(list: *mut WordList) -> c_int;

#[repr(C)]
pub struct Builtin {
    pub name: *mut c_char,
    pub function: BuiltinFnPtr,
    pub flags: c_int,
    pub long_doc: *mut *const c_char,
    pub short_doc: *const c_char,
    pub handle: *mut c_char,
}

impl Builtin {
    fn disabled(name: &str) -> Self {
        let name = CString::new(name).unwrap().into_raw();
        Self {
            name: name,
            function: disabled,
            flags: 1,
            long_doc: ptr::null_mut(),
            short_doc: ptr::null_mut(),
            handle: ptr::null_mut(),
        }
    }

    pub fn register(name: &str) -> Self {
        let (_func, short_doc, long_doc) = match BUILTINS.get(name) {
            Some(item) => *item,
            None => return Self::disabled(name),
        };
        let name = CString::new(name).unwrap().into_raw();
        let short_doc = CString::new(short_doc).unwrap().into_raw();
        let long_doc: Vec<*mut c_char> = long_doc.split("\n")
            .map(|s| CString::new(s).unwrap().into_raw())
            .collect();
        let long_doc = Box::into_raw(long_doc.into_boxed_slice()).cast();
        Self {
            name: name,
            function: run,
            flags: 1,
            long_doc: long_doc,
            short_doc: short_doc,
            handle: ptr::null_mut(),
        }
    }
}

#[no_mangle]
pub(crate) unsafe extern "C" fn run(list: *mut WordList) -> c_int {
    // get the current running command name
    let cmd = unsafe { current_command() };
    // find its matching rust function and execute it
    let (func, _short_doc, _long_doc) = *BUILTINS.get(cmd).unwrap();
    let args = unsafe { list.into_vec().unwrap() };

    let ret = match func(args.as_slice()) {
        Ok(ret) => ret,
        Err(e) => {
            eprintln!("{}: error: {}", cmd, e);
            -1
        }
    };

    ret as c_int
}

#[no_mangle]
pub(crate) unsafe extern "C" fn disabled(_list: *mut WordList) -> c_int {
    // get the current running command name
    let cmd = unsafe { current_command() };
    eprintln!("error: missing plugin support: {}", cmd);
    -1 as c_int
}
