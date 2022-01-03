use std::ffi::{CStr, CString};

pub mod bindings;
pub mod builtins;

/// Get the currently running bash command name.
#[inline]
pub fn current_command() -> &'static str {
    unsafe {
        CStr::from_ptr(bindings::this_command_name)
            .to_str()
            .unwrap()
    }
}

/// Get the string value of a given variable name.
pub fn string_value(name: &str) -> Option<&str> {
    let name = CString::new(name).unwrap();
    match unsafe { bindings::get_string_value(name.as_ptr()) } {
        s if s.is_null() => None,
        s => Some(unsafe { CStr::from_ptr(s).to_str().unwrap() }),
    }
}

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

unsafe impl IntoVec for *mut bindings::WordList {
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
