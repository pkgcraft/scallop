use std::ffi::CStr;

use crate::bash::WordList;

/// Support conversion from a given object into a Vec<T>.
///
/// # Safety
/// This assumes the object being converted contains valid data.
pub unsafe trait IntoVec {
    /// Convert a given object into a Vec<&str>.
    ///
    /// # Safety
    /// This assumes the object being converted contains valid strings.
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
