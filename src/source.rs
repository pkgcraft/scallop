use std::ffi::CString;
use std::path::Path;

use bitflags::bitflags;
use once_cell::sync::Lazy;

use crate::builtins::ExecStatus;
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

pub fn string<S: AsRef<str>>(s: S) -> Result<ExecStatus> {
    let file_ptr = FILE_STR.as_ptr();
    let s = s.as_ref();
    let c_str = CString::new(s).unwrap();
    let str_ptr = c_str.as_ptr() as *mut _;

    unsafe {
        bash::evalstring(str_ptr, file_ptr, Eval::NO_FREE.bits() as i32);
    }

    ok_or_error()
}

pub fn file<P: AsRef<Path>>(path: P) -> Result<ExecStatus> {
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

    use tempfile::NamedTempFile;

    use crate::variables::string_value;
    use crate::source;

    #[test]
    fn test_source_string() {
        assert_eq!(string_value("VAR"), None);

        source::string("VAR=1").unwrap();
        assert_eq!(string_value("VAR").unwrap(), "1");

        source::string("VAR=").unwrap();
        assert_eq!(string_value("VAR").unwrap(), "");

        source::string("unset -v VAR").unwrap();
        assert_eq!(string_value("VAR"), None);
    }

    #[test]
    fn test_source_string_error() {
        // bad bash code raises error
        let err = source::string("local VAR").unwrap_err();
        assert_eq!(err.to_string(), "scallop: local: can only be used in a function");

        // Sourcing still continues even when an error is returned
        // because the analog to `set -e` isn't enabled.
        assert!(source::string("local VAR\nVAR=1").is_err());
        assert_eq!(string_value("VAR").unwrap(), "1");
    }

    #[test]
    fn test_source_file() {
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
    }

    #[test]
    fn test_source_file_error() {
        assert_eq!(string_value("VAR"), None);
        let mut file = NamedTempFile::new().unwrap();

        // bad bash code raises error
        writeln!(file, "local VAR").unwrap();
        let err = source::file(file.path()).unwrap_err();
        assert!(err
            .to_string()
            .ends_with("line 1: local: can only be used in a function"));

        // Sourcing still continues even when an error is returned
        // because the analog to `set -e` isn't enabled.
        writeln!(file, "VAR=1").unwrap();
        assert!(source::file(file.path()).is_err());
        assert_eq!(string_value("VAR").unwrap(), "1");
    }
}
