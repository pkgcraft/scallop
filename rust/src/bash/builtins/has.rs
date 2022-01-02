use crate::{Error, Result};

pub(crate) static SHORT_DOC: &str = "has needle ${haystack}";
pub(crate) static LONG_DOC: &str = "\
Returns 0 if the first argument is found in the list of subsequent arguments, 1 otherwise.

Returns -1 on error.";

#[doc = stringify!(LONG_DOC)]
pub fn has(args: &[&str]) -> Result<i32> {
    let needle = match args.first() {
        Some(s) => s,
        None => return Err(Error::new("requires 1 or more args")),
    };

    let haystack = &args[1..];
    Ok(!haystack.contains(needle) as i32)
}
