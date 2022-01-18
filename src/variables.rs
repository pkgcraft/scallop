use std::ffi::{CStr, CString};
use std::slice;

use bitflags::bitflags;

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

pub fn unbind<S: AsRef<str>>(name: S) -> Result<()> {
    let name = name.as_ref();
    let cstr = CString::new(name).unwrap();
    let var = unsafe { bash::find_variable(cstr.as_ptr()).as_ref() };
    if var.is_some() {
        let ret = unsafe { bash::check_unbind_variable(cstr.as_ptr()) };
        if ret != 0 {
            return Err(Error::new(format!("failed unbinding variable: {}", name)));
        }
    }
    Ok(())
}

pub fn bind<S: AsRef<str>>(name: S, value: S, flags: Option<Assign>, attrs: Option<Attr>) {
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
}

pub fn bind_global<S: AsRef<str>>(name: S, value: S, flags: Option<Assign>, attrs: Option<Attr>) {
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
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
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
    fn bind<S: AsRef<str>>(&mut self, value: S, flags: Option<Assign>, attrs: Option<Attr>) {
        bind(self.name(), value.as_ref(), flags, attrs)
    }

    #[inline]
    fn bind_global<S: AsRef<str>>(&mut self, value: S, flags: Option<Assign>, attrs: Option<Attr>) {
        bind_global(self.name(), value.as_ref(), flags, attrs)
    }

    #[inline]
    fn unbind(&mut self) -> Result<()> {
        unbind(self.name())
    }

    #[inline]
    fn append(&mut self, s: &str) {
        self.bind(s, Some(Assign::APPEND), None)
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
        let current = string_value(&self.var.name);
        if current != self.orig {
            if let Some(val) = &self.orig {
                self.var.bind(val, None, None);
            } else {
                self.var.unbind().unwrap();
            }
        }
    }
}

/// Get the raw string value of a given variable name.
pub fn string_value(name: &str) -> Option<String> {
    let name = CString::new(name).unwrap();
    let ptr = unsafe { bash::get_string_value(name.as_ptr()).as_ref() };
    ptr.map(|s| unsafe { String::from(CStr::from_ptr(s).to_str().unwrap()) })
}

/// Get the string value of a given variable name splitting it into Vec<String> based on IFS.
pub fn string_vec(name: &str) -> Option<Vec<String>> {
    let name = CString::new(name).unwrap();
    let ptr = unsafe { bash::get_string_value(name.as_ptr()).as_mut() };
    ptr.map(|s| {
        let words = unsafe { bash::list_string(s, bash::IFS, 1) };
        // TODO: implement iterators directly for WordList
        let strings = words.into_vec().iter().map(|s| s.to_string()).collect();
        unsafe { bash::dispose_words(words) };
        strings
    })
}

/// Get the value of an array for a given variable name.
pub fn array_to_vec(name: &str) -> Result<Vec<String>> {
    let var_name = CString::new(name).unwrap();
    let var = unsafe { bash::find_variable(var_name.as_ptr()).as_ref() };
    let array_ptr = match var {
        None => return Err(Error::new(format!("undefined variable: {}", name))),
        Some(v) => match (v.attributes as u32 & Attr::ARRAY.bits()) != 0 {
            true => v.value as *mut bash::Array,
            false => return Err(Error::new(format!("variable is not an array: {}", name))),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init;

    use rusty_fork::rusty_fork_test;

    rusty_fork_test! {
        #[test]
        fn test_string_vec() {
            init("sh");
            assert_eq!(string_vec("VAR"), None);
            bind("VAR", "", None, None);
            assert_eq!(string_vec("VAR").unwrap(), vec![""; 0]);
            bind("VAR", "a", None, None);
            assert_eq!(string_vec("VAR").unwrap(), vec!["a"]);
            bind("VAR", "1 2 3", None, None);
            assert_eq!(string_vec("VAR").unwrap(), vec!["1", "2", "3"]);
            unbind("VAR").unwrap();
            assert_eq!(string_vec("VAR"), None);
        }

        #[test]
        fn test_variable() {
            init("sh");
            let mut var = Variable::new("VAR");
            assert_eq!(var.string_value(), None);
            var.bind("", None, None);
            assert_eq!(var.string_value().unwrap(), "");
            var.bind("1", None, None);
            assert_eq!(var.string_value().unwrap(), "1");
            var.append("2");
            assert_eq!(var.string_value().unwrap(), "12");
            var.append(" 3");
            assert_eq!(var.string_value().unwrap(), "12 3");
            var.unbind().unwrap();
            assert_eq!(var.string_value(), None);
        }

        #[test]
        fn test_scoped_variable() {
            init("sh");
            bind("VAR", "outer", None, None);
            assert_eq!(string_value("VAR").unwrap(), "outer");
            {
                let mut var = ScopedVariable::new("VAR");
                var.bind("inner", None, None);
                assert_eq!(var.string_value().unwrap(), "inner");
            }
            assert_eq!(string_value("VAR").unwrap(), "outer");
        }
    }
}
