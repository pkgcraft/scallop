use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

use crate::error::ok_or_error;
use crate::{bash, Result};

#[derive(Debug)]
pub struct Function<'a> {
    name: String,
    func: &'a mut bash::ShellVar,
}

impl Function<'_> {
    /// Execute a given shell function.
    pub fn execute(&mut self, args: &[&str]) -> Result<()> {
        let args = [&[self.name.as_str()], args].concat();
        let arg_strs: Vec<CString> = args.iter().map(|s| CString::new(*s).unwrap()).collect();
        let mut arg_ptrs: Vec<*mut c_char> =
            arg_strs.iter().map(|s| s.as_ptr() as *mut _).collect();
        arg_ptrs.push(ptr::null_mut());
        let args = arg_ptrs.as_ptr() as *mut _;
        unsafe {
            let words = bash::strvec_to_word_list(args, 0, 0);
            bash::execute_shell_function(self.func, words);
        }
        ok_or_error()
    }
}

/// Find a given shell function.
pub fn find(name: &str) -> Option<Function> {
    let func_name = CString::new(name).unwrap();
    let func = unsafe { bash::find_function(func_name.as_ptr()).as_mut() };
    func.map(|f| Function {
        name: name.to_string(),
        func: f,
    })
}

#[cfg(test)]
mod tests {
    use crate::variables::string_value;
    use crate::{functions, source, Shell};

    use rusty_fork::rusty_fork_test;

    rusty_fork_test! {
        #[test]
        fn find() {
            let _sh = Shell::new("sh", None);
            assert!(functions::find("foo").is_none());
            source::string("foo() { :; }").unwrap();
            assert!(functions::find("foo").is_some());
        }

        #[test]
        fn execute() {
            let _sh = Shell::new("sh", None);
            assert_eq!(string_value("VAR"), None);
            source::string("foo() { VAR=1; }").unwrap();
            let mut func = functions::find("foo").unwrap();
            func.execute(&[]).unwrap();
            assert_eq!(string_value("VAR").unwrap(), "1");
        }
    }
}
