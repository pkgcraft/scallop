use std::ffi::CString;

use crate::builtins::Builtin;
use crate::{bash, string_value, Error, Result};

static LONG_DOC: &str = "\
Export stub functions that call the eclass's functions, thereby making them default.
For example, if ECLASS=base and `EXPORT_FUNCTIONS src_unpack` is called the following
function is defined:

src_unpack() { base_src_unpack; }";

#[doc = stringify!(LONG_DOC)]
pub(crate) fn run(args: &[&str]) -> Result<i32> {
    let eclass = match string_value("ECLASS") {
        Some(val) => val,
        None => return Err(Error::new("no ECLASS defined")),
    };

    let file_ptr = CString::new("EXPORT_FUNCTIONS").unwrap().into_raw();
    for func in args {
        let func_str = format!(
            "{func}() {{ {eclass}_{func} \"$@\"; }}",
            func = func,
            eclass = eclass
        );
        unsafe {
            let func_ptr = CString::new(func_str).unwrap().into_raw();
            bash::evalstring(func_ptr, file_ptr, bash::SEVAL_NOFREE as i32);
            drop(CString::from_raw(func_ptr));
        }
    }
    unsafe { drop(CString::from_raw(file_ptr)) };

    Ok(0)
}

pub static BUILTIN: Builtin = Builtin {
    name: "EXPORT_FUNCTIONS",
    func: run,
    help: LONG_DOC,
    usage: "EXPORT_FUNCTIONS src_configure src_compile",
    exit_on_error: false,
};
