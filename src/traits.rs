use std::ffi::CStr;

use crate::bash::WordList;

/// Support conversion from a given object into a Vec<T>.
pub trait IntoVec {
    /// Convert a given object into a Vec<&str>.
    fn into_vec<'a>(self) -> Vec<&'a str>;
}

impl IntoVec for *mut WordList {
    fn into_vec<'a>(self) -> Vec<&'a str> {
        let mut list = self;
        let mut strings = Vec::new();

        while !list.is_null() {
            let word = unsafe { (*(*list).word).word };
            let val = unsafe { CStr::from_ptr(word).to_str().unwrap() };
            strings.push(val);
            list = unsafe { (*list).next };
        }

        strings
    }
}
