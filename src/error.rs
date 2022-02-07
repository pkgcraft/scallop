use std::cell::RefCell;
use std::ffi::CStr;
use std::os::raw::c_char;

use tracing::warn;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Base(String),
    #[error("{0}")]
    Builtin(String),
}

thread_local! {
    static LAST_ERROR: RefCell<Option<Error>> = RefCell::new(None);
}

/// Wrapper to convert internal bash errors into native errors.
#[no_mangle]
pub(crate) extern "C" fn bash_error(msg: *mut c_char) {
    let msg = unsafe { CStr::from_ptr(msg).to_string_lossy() };
    LAST_ERROR.with(|prev| {
        *prev.borrow_mut() = Some(Error::Base(msg.into()));
    });
}

/// Retrieve the most recent internal bash error.
#[inline]
pub fn last_error() -> Option<Error> {
    LAST_ERROR.with(|prev| prev.borrow_mut().take())
}

/// Return the most recent error if one exists, otherwise Ok(()).
#[inline]
pub fn ok_or_error() -> Result<()> {
    match last_error() {
        None => Ok(()),
        Some(e) => Err(e),
    }
}

/// Wrapper to support outputting log messages for bash warnings.
#[no_mangle]
pub(crate) extern "C" fn bash_warning(msg: *mut c_char) {
    if let Ok(msg) = unsafe { CStr::from_ptr(msg).to_str() } {
        warn!(msg);
    }
}
