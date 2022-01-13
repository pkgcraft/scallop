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

pub fn unbind<S: AsRef<str>>(name: S) -> Result<i32> {
    let name = name.as_ref();
    let cstr = CString::new(name).unwrap();
    let var = unsafe { bash::find_variable(cstr.as_ptr()).as_ref() };
    if var.is_some() {
        let ret = unsafe { bash::check_unbind_variable(cstr.as_ptr()) };
        if ret != 0 {
            return Err(Error::new(format!("failed unbinding variable: {}", name)));
        }
    }
    Ok(0)
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
}

impl Variable {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Variable { name: name.into() }
    }

    pub fn bind<S: AsRef<str>>(&self, value: S, flags: Option<Attr>) {
        let name = CString::new(self.name.as_str()).unwrap();
        let value = CString::new(value.as_ref()).unwrap();
        let val = value.as_ptr() as *mut _;
        let flags = flags.unwrap_or(Attr::NONE).bits() as i32;
        unsafe { bash::bind_variable(name.as_ptr(), val, flags) };
    }

    pub fn bind_global<S: AsRef<str>>(&self, value: S, flags: Option<Attr>) {
        let name = CString::new(self.name.as_str()).unwrap();
        let value = CString::new(value.as_ref()).unwrap();
        let val = value.as_ptr() as *mut _;
        let flags = flags.unwrap_or(Attr::NONE).bits() as i32;
        unsafe { bash::bind_global_variable(name.as_ptr(), val, flags) };
    }

    #[inline]
    pub fn unbind(&self) -> Result<i32> {
        unbind(self.name.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct ScopedVariable {
    var: Variable,
    orig: Option<String>,
}

/// Variable that will reset itself to its original value when it leaves scope.
impl ScopedVariable {
    #[inline]
    pub fn new<S: Into<String>>(name: S) -> Self {
        let var = Variable::new(name);
        let orig = string_value(&var.name);
        ScopedVariable { var, orig }
    }

    #[inline]
    pub fn bind<S: AsRef<str>>(&self, value: S, flags: Option<Attr>) {
        self.var.bind(value, flags)
    }

    #[inline]
    pub fn bind_global<S: AsRef<str>>(&self, value: S, flags: Option<Attr>) {
        self.var.bind_global(value, flags)
    }
}

impl Drop for ScopedVariable {
    #[inline]
    fn drop(&mut self) {
        let current = string_value(&self.var.name);
        if current != self.orig {
            if let Some(val) = &self.orig {
                self.var.bind(val, None);
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
