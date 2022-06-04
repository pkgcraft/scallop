use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

use crate::builtins::ExecStatus;
use crate::command::cmd_scope;
use crate::error::ok_or_error;
use crate::{bash, Result};

/// Run the `local` builtin with the given arguments.
pub fn local<S: AsRef<str>>(args: &[S]) -> Result<ExecStatus> {
    let arg_strs: Vec<CString> = args
        .iter()
        .map(|s| CString::new(s.as_ref()).unwrap())
        .collect();
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

/// Run the `set` builtin with the given arguments.
pub fn set<S1: AsRef<str>, S2: AsRef<str>>(opts: &[S1], args: &[S2]) -> Result<ExecStatus> {
    let mut arg_strs = Vec::<CString>::new();
    arg_strs.extend(opts.iter().map(|s| CString::new(s.as_ref()).unwrap()));
    arg_strs.extend(args.iter().map(|s| CString::new(s.as_ref()).unwrap()));
    let mut arg_ptrs: Vec<*mut c_char> = arg_strs.iter().map(|s| s.as_ptr() as *mut _).collect();
    arg_ptrs.push(ptr::null_mut());
    let args = arg_ptrs.as_ptr() as *mut _;

    unsafe {
        // TODO: add better support for converting string vectors/iterators to WordLists
        let words = bash::strvec_to_word_list(args, 0, 0);
        cmd_scope("set", || {
            bash::set_builtin(words);
        });
    }

    ok_or_error()
}

/// Run the `shopt` builtin with the given arguments.
pub fn shopt<S1: AsRef<str>, S2: AsRef<str>>(opts: &[S1], args: &[S2]) -> Result<ExecStatus> {
    let mut arg_strs = Vec::<CString>::new();
    arg_strs.extend(opts.iter().map(|s| CString::new(s.as_ref()).unwrap()));
    arg_strs.extend(args.iter().map(|s| CString::new(s.as_ref()).unwrap()));
    let mut arg_ptrs: Vec<*mut c_char> = arg_strs.iter().map(|s| s.as_ptr() as *mut _).collect();
    arg_ptrs.push(ptr::null_mut());
    let args = arg_ptrs.as_ptr() as *mut _;

    unsafe {
        // TODO: add better support for converting string vectors/iterators to WordLists
        let words = bash::strvec_to_word_list(args, 0, 0);
        cmd_scope("shopt", || {
            bash::shopt_builtin(words);
        });
    }

    ok_or_error()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::functions::bash_func;
    use crate::variables::{bind, string_value};
    use crate::Shell;

    use rusty_fork::rusty_fork_test;

    rusty_fork_test! {
        #[test]
        fn test_local() {
            let _sh = Shell::new("sh", None);
            bind("VAR", "outer", None, None).unwrap();
            bash_func("func_name", || {
                local(&["VAR=inner"]).unwrap();
                assert_eq!(string_value("VAR").unwrap(), "inner");
            });
            assert_eq!(string_value("VAR").unwrap(), "outer");
        }
    }
}
