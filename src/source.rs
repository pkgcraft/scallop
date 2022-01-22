use std::ffi::CString;
use std::path::Path;

use bitflags::bitflags;
use once_cell::sync::Lazy;

use crate::error::ok_or_error;
use crate::{bash, Result};

bitflags! {
    /// Flag values used with source::string() for altering string evaluation.
    pub struct Eval: u32 {
        const NONE = 0;
        const NON_INTERACTIVE = bash::SEVAL_NONINT;
        const INTERACTIVE = bash::SEVAL_INTERACT;
        const NO_HISTORY = bash::SEVAL_NOHIST;
        const NO_FREE = bash::SEVAL_NOFREE;
        const RESET_LINE = bash::SEVAL_RESETLINE;
        const PARSE_ONLY = bash::SEVAL_PARSEONLY;
        const NO_LONGJMP = bash::SEVAL_NOLONGJMP;
        const FUNCDEF = bash::SEVAL_FUNCDEF;
        const ONE_COMMAND = bash::SEVAL_ONECMD;
        const NO_HISTORY_EXPANSION = bash::SEVAL_NOHISTEXP;
    }
}

static FILE_STR: Lazy<CString> = Lazy::new(|| CString::new("scallop::source::string").unwrap());

pub fn string<S: AsRef<str>>(s: S) -> Result<()> {
    let file_ptr = FILE_STR.as_ptr();
    let s = s.as_ref();
    let c_str = CString::new(s).unwrap();
    let str_ptr = c_str.as_ptr() as *mut _;

    unsafe {
        bash::evalstring(str_ptr, file_ptr, Eval::NO_FREE.bits() as i32);
    }

    ok_or_error()
}

pub fn file<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    let c_str = CString::new(path.to_str().unwrap()).unwrap();
    let str_ptr = c_str.as_ptr();

    unsafe {
        bash::source_file(str_ptr, 0);
    }

    ok_or_error()
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use crate::variables::string_value;
    use crate::{source, Shell};

    use rusty_fork::rusty_fork_test;
    use tempfile::NamedTempFile;

    rusty_fork_test! {
        #[test]
        fn test_source_string() {
            let _sh = Shell::new("sh", None);
            assert_eq!(string_value("VAR"), None);

            source::string("VAR=1").unwrap();
            assert_eq!(string_value("VAR").unwrap(), "1");

            source::string("VAR=").unwrap();
            assert_eq!(string_value("VAR").unwrap(), "");

            source::string("unset -v VAR").unwrap();
            assert_eq!(string_value("VAR"), None);

            // bad bash code raises error
            assert!(source::string("local VAR").is_err());
        }

        #[test]
        fn test_source_file() {
            let _sh = Shell::new("sh", None);
            assert_eq!(string_value("VAR"), None);
            let mut file = NamedTempFile::new().unwrap();

            writeln!(file, "VAR=1").unwrap();
            source::file(file.path()).unwrap();
            assert_eq!(string_value("VAR").unwrap(), "1");

            writeln!(file, "VAR=").unwrap();
            source::file(file.path()).unwrap();
            assert_eq!(string_value("VAR").unwrap(), "");

            writeln!(file, "unset -v VAR").unwrap();
            source::file(file.path()).unwrap();
            assert_eq!(string_value("VAR"), None);

            // bad bash code raises error
            writeln!(file, "local VAR").unwrap();
            assert!(source::file(file.path()).is_err());
        }
    }
}
