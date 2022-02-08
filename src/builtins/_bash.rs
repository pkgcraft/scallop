use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

use crate::command::cmd_scope;
use crate::error::ok_or_error;
use crate::{bash, Result};

/// Run the `local` builtin with the given arguments.
pub fn local(assign: &[&str]) -> Result<()> {
    let arg_strs: Vec<CString> = assign.iter().map(|s| CString::new(*s).unwrap()).collect();
    let mut arg_ptrs: Vec<*mut c_char> = arg_strs.iter().map(|s| s.as_ptr() as *mut _).collect();
    arg_ptrs.push(ptr::null_mut());
    let args = arg_ptrs.as_ptr() as *mut _;

    unsafe {
        // TODO: add better support for converting string vectors/iterators to WordLists
        let words = bash::strvec_to_word_list(args, 0, 0);
        cmd_scope("local", || {
            bash::local_builtin(words);
        });
    }

    ok_or_error()
}
