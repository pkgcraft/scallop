use std::ffi::CString;

use crate::{bash, Result};

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
}

impl Variable {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Variable { name: name.into() }
    }

    pub fn bind<S: AsRef<str>>(&self, value: S, flags: Option<i32>) {
        let name = CString::new(self.name.as_str()).unwrap();
        let value = CString::new(value.as_ref()).unwrap();
        let val = value.as_ptr() as *mut _;
        let flags = flags.unwrap_or(0);
        unsafe { bash::bind_variable(name.as_ptr(), val, flags) };
    }

    pub fn unbind(&self) -> Result<i32> {
        let name = CString::new(self.name.as_str()).unwrap();
        let ret: i32;
        unsafe { ret = bash::unbind_variable(name.as_ptr()) };
        Ok(ret)
    }
}
