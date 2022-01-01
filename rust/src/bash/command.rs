// Bindings for various types from bash/command.h.

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct WordDesc {
    pub word: *mut c_char,
    pub flags: c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct WordList {
    pub next: *mut WordList,
    pub word: *mut WordDesc,
}

pub unsafe trait IntoVec {
    unsafe fn into_vec<'a>(self) -> crate::Result<Vec<&'a str>>;
}

unsafe impl IntoVec for *mut WordList {
    unsafe fn into_vec<'a>(self) -> crate::Result<Vec<&'a str>> {
        let mut list = self;
        let mut vec = Vec::new();

        while !list.is_null() {
            let word = unsafe { (*(*list).word).word };
            let val = unsafe { CStr::from_ptr(word).to_str().unwrap() };
            vec.push(val);
            list = unsafe { (*list).next };
        }

        Ok(vec)
    }
}
