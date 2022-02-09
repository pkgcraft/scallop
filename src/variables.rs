use std::ffi::{CStr, CString};
use std::slice;

use bitflags::bitflags;

use crate::builtins::ExecStatus;
use crate::error::ok_or_error;
use crate::traits::IntoVec;
use crate::{bash, Error, Result};

bitflags! {
    /// Flags for various attributes a given variable can have.
    pub struct Attr: u32 {
        const NONE = 0;
        const EXPORTED = bash::att_exported;
        const READONLY = bash::att_readonly;
        const ARRAY = bash::att_array;
        const FUNCTION = bash::att_function;
        const INTEGER = bash::att_integer;
        const LOCAL = bash::att_local;
        const ASSOC = bash::att_assoc;
        const TRACE = bash::att_trace;
        const UPPERCASE = bash::att_uppercase;
        const LOWERCASE = bash::att_lowercase;
        const CAPCASE = bash::att_capcase;
        const NAMEREF = bash::att_nameref;
        const INVISIBLE = bash::att_invisible;
        const NO_UNSET = bash::att_nounset;
        const NO_ASSIGN = bash::att_noassign;
    }
}

bitflags! {
    /// Flag values controlling how assignment statements are treated.
    pub struct Assign: u32 {
        const NONE = 0;
        const APPEND = bash::ASS_APPEND;
        const LOCAL = bash::ASS_MKLOCAL;
        const GLOBAL = bash::ASS_MKGLOBAL;
        const NAMEREF = bash::ASS_NAMEREF;
        const FORCE = bash::ASS_FORCE;
        const CHKLOCAL = bash::ASS_CHKLOCAL;
        const NOEXPAND = bash::ASS_NOEXPAND;
        const NOEVAL = bash::ASS_NOEVAL;
        const NOLONGJMP = bash::ASS_NOLONGJMP;
        const NOINVIS = bash::ASS_NOINVIS;
    }
}

pub fn unbind<S: AsRef<str>>(name: S) -> Result<ExecStatus> {
    let name = name.as_ref();
    let cstr = CString::new(name).unwrap();
    unsafe {
        bash::check_unbind_variable(cstr.as_ptr());
    }
    ok_or_error()
}

pub fn bind<S1, S2>(
    name: S1,
    value: S2,
    flags: Option<Assign>,
    attrs: Option<Attr>,
) -> Result<ExecStatus>
where
    S1: AsRef<str>,
    S2: AsRef<str>,
{
    let name = CString::new(name.as_ref()).unwrap();
    let value = CString::new(value.as_ref()).unwrap();
    let val = value.as_ptr() as *mut _;
    let flags = flags.unwrap_or(Assign::NONE).bits() as i32;
    let var = unsafe { bash::bind_variable(name.as_ptr(), val, flags).as_mut() };
    if let Some(var) = var {
        if let Some(attrs) = attrs {
            var.attributes |= attrs.bits() as i32;
        }
    }
    ok_or_error()
}

pub fn bind_global<S: AsRef<str>>(
    name: S,
    value: S,
    flags: Option<Assign>,
    attrs: Option<Attr>,
) -> Result<ExecStatus> {
    let name = CString::new(name.as_ref()).unwrap();
    let value = CString::new(value.as_ref()).unwrap();
    let val = value.as_ptr() as *mut _;
    let flags = flags.unwrap_or(Assign::NONE).bits() as i32;
    let var = unsafe { bash::bind_global_variable(name.as_ptr(), val, flags).as_mut() };
    if let Some(var) = var {
        if let Some(attrs) = attrs {
            var.attributes |= attrs.bits() as i32;
        }
    }
    ok_or_error()
}

#[derive(Debug, Clone)]
pub struct Variable {
    name: String,
}

impl Variable {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Variable { name: name.into() }
    }
}

pub trait Variables {
    fn name(&self) -> &str;

    #[inline]
    fn string_value(&self) -> Option<String> {
        string_value(self.name())
    }

    #[inline]
    fn expand(&self) -> Option<String> {
        self.string_value().and_then(expand)
    }

    #[inline]
    fn bind<S: AsRef<str>>(
        &mut self,
        value: S,
        flags: Option<Assign>,
        attrs: Option<Attr>,
    ) -> Result<ExecStatus> {
        bind(self.name(), value.as_ref(), flags, attrs)
    }

    #[inline]
    fn bind_global<S: AsRef<str>>(
        &mut self,
        value: S,
        flags: Option<Assign>,
        attrs: Option<Attr>,
    ) -> Result<ExecStatus> {
        bind_global(self.name(), value.as_ref(), flags, attrs)
    }

    #[inline]
    fn unbind(&mut self) -> Result<ExecStatus> {
        unbind(self.name())
    }

    #[inline]
    fn append(&mut self, s: &str) -> Result<ExecStatus> {
        self.bind(s, Some(Assign::APPEND), None)
    }

    #[inline]
    fn shell_var(&self) -> Option<&mut bash::ShellVar> {
        let var_name = CString::new(self.name()).unwrap();
        unsafe { bash::find_variable(var_name.as_ptr()).as_mut() }
    }

    #[inline]
    fn is_array(&self) -> bool {
        match self.shell_var() {
            None => false,
            Some(v) => v.attributes as u32 & Attr::ARRAY.bits() != 0,
        }
    }

    #[inline]
    fn is_readonly(&self) -> bool {
        match self.shell_var() {
            None => false,
            Some(v) => v.attributes as u32 & Attr::READONLY.bits() != 0,
        }
    }
}

impl Variables for Variable {
    #[inline]
    fn name(&self) -> &str {
        self.name.as_str()
    }
}

#[derive(Debug, Clone)]
pub struct ScopedVariable {
    var: Variable,
    orig: Option<String>,
}

/// Variable that will reset itself to its original value when it leaves scope.
impl ScopedVariable {
    pub fn new<S: Into<String>>(name: S) -> Self {
        let var = Variable::new(name);
        let orig = string_value(&var.name);
        ScopedVariable { var, orig }
    }
}

impl Variables for ScopedVariable {
    #[inline]
    fn name(&self) -> &str {
        self.var.name.as_str()
    }
}

impl Drop for ScopedVariable {
    #[inline]
    fn drop(&mut self) {
        if string_value(&self.var.name) != self.orig {
            let mut reset = || -> Result<ExecStatus> {
                if let Some(val) = &self.orig {
                    self.var.bind(val, None, None)
                } else {
                    self.var.unbind()
                }
            };
            reset().unwrap_or_else(|_| panic!("failed resetting variable: {}", self.var.name));
        }
    }
}

/// Get the raw string value of a given variable name.
pub fn string_value<S: AsRef<str>>(name: S) -> Option<String> {
    let name = CString::new(name.as_ref()).unwrap();
    let ptr = unsafe { bash::get_string_value(name.as_ptr()).as_ref() };
    ptr.map(|s| unsafe { String::from(CStr::from_ptr(s).to_str().unwrap()) })
}

/// Get the expanded value of a given string.
pub fn expand<S: AsRef<str>>(name: S) -> Option<String> {
    let name = CString::new(name.as_ref()).unwrap();
    let ptr = unsafe { bash::expand_string_to_string(name.as_ptr() as *mut _, 0).as_ref() };
    ptr.map(|s| unsafe { String::from(CStr::from_ptr(s).to_str().unwrap()) })
}

/// Get the string value of a given variable name splitting it into Vec<String> based on IFS.
pub fn string_vec<S: AsRef<str>>(name: S) -> Result<Vec<String>> {
    let name = name.as_ref();
    let var_name = CString::new(name).unwrap();
    let ptr = unsafe { bash::get_string_value(var_name.as_ptr()).as_mut() };
    match ptr {
        None => Err(Error::Base(format!("undefined variable: {}", name))),
        Some(s) => {
            let words = unsafe { bash::list_string(s, bash::IFS, 1) };
            // TODO: implement iterators directly for WordList
            let strings = words.into_vec().iter().map(|s| s.to_string()).collect();
            unsafe { bash::dispose_words(words) };
            Ok(strings)
        }
    }
}

/// Get the value of an array for a given variable name.
pub fn array_to_vec<S: AsRef<str>>(name: S) -> Result<Vec<String>> {
    let name = name.as_ref();
    let var_name = CString::new(name).unwrap();
    let var = unsafe { bash::find_variable(var_name.as_ptr()).as_ref() };
    let array_ptr = match var {
        None => return Err(Error::Base(format!("undefined variable: {}", name))),
        Some(v) => match (v.attributes as u32 & Attr::ARRAY.bits()) != 0 {
            true => v.value as *mut bash::Array,
            false => return Err(Error::Base(format!("variable is not an array: {}", name))),
        },
    };

    let mut count: i32 = 0;
    let strings: Vec<String>;

    unsafe {
        let str_array = bash::array_to_argv(array_ptr, &mut count);
        strings = slice::from_raw_parts(str_array, count as usize)
            .iter()
            .map(|s| String::from(CStr::from_ptr(*s).to_str().unwrap()))
            .collect();
        bash::strvec_dispose(str_array);
    }

    Ok(strings)
}

/// Get the value of a given variable as Vec<String>.
pub fn var_to_vec<S: AsRef<str>>(name: S) -> Result<Vec<String>> {
    let name = name.as_ref();
    let var = Variable::new(name);
    match var.is_array() {
        false => string_vec(name),
        true => array_to_vec(name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Shell;

    use rusty_fork::rusty_fork_test;

    rusty_fork_test! {
        #[test]
        fn test_string_vec() {
            let _sh = Shell::new("sh", None);
            assert!(string_vec("VAR").is_err());
            bind("VAR", "", None, None).unwrap();
            assert_eq!(string_vec("VAR").unwrap(), vec![""; 0]);
            bind("VAR", "a", None, None).unwrap();
            assert_eq!(string_vec("VAR").unwrap(), vec!["a"]);
            bind("VAR", "1 2 3", None, None).unwrap();
            assert_eq!(string_vec("VAR").unwrap(), vec!["1", "2", "3"]);
            unbind("VAR").unwrap();
            assert!(string_vec("VAR").is_err());
        }

        #[test]
        fn test_readonly_var() {
            let _sh = Shell::new("sh", None);
            bind("VAR", "1", None, Some(Attr::READONLY)).unwrap();
            assert_eq!(string_value("VAR").unwrap(), "1");
            let err = bind("VAR", "1", None, None).unwrap_err();
            assert_eq!(err.to_string(), "sh: VAR: readonly variable");
            let err = unbind("VAR").unwrap_err();
            assert_eq!(err.to_string(), "sh: VAR: cannot unset: readonly variable");
        }

        #[test]
        fn test_variable() {
            let _sh = Shell::new("sh", None);
            let mut var = Variable::new("VAR");
            assert_eq!(var.string_value(), None);
            var.bind("", None, None).unwrap();
            assert_eq!(var.string_value().unwrap(), "");
            var.bind("1", None, None).unwrap();
            assert_eq!(var.string_value().unwrap(), "1");
            var.append("2").unwrap();
            assert_eq!(var.string_value().unwrap(), "12");
            var.append(" 3").unwrap();
            assert_eq!(var.string_value().unwrap(), "12 3");
            var.unbind().unwrap();
            assert_eq!(var.string_value(), None);
        }

        #[test]
        fn test_expand() {
            let _sh = Shell::new("sh", None);
            let mut var1 = Variable::new("VAR1");
            let mut var2 = Variable::new("VAR2");
            var1.bind("1", None, None).unwrap();
            var2.bind("${VAR1}", None, None).unwrap();
            assert_eq!(var2.expand().unwrap(), "1");
            assert_eq!(expand("$VAR1").unwrap(), "1");
        }

        #[test]
        fn test_scoped_variable() {
            let _sh = Shell::new("sh", None);
            bind("VAR", "outer", None, None).unwrap();
            assert_eq!(string_value("VAR").unwrap(), "outer");
            {
                let mut var = ScopedVariable::new("VAR");
                var.bind("inner", None, None).unwrap();
                assert_eq!(var.string_value().unwrap(), "inner");
            }
            assert_eq!(string_value("VAR").unwrap(), "outer");
        }
    }
}
