use std::cmp;

use pkgcraft::bash::{parse, version_split};

use crate::bash::string_value;
use crate::{Error, Result};

pub(crate) static SHORT_DOC: &str = "ver_cut 1-2 - 1.2.3";
pub(crate) static LONG_DOC: &str = "\
Output substring from package version string and range arguments.

Returns -1 on error.";

#[doc = stringify!(LONG_DOC)]
pub fn ver_cut(args: &[&str]) -> Result<i32> {
    let pv = string_value("PV").unwrap_or("");
    let (range, ver) = match args.len() {
        1 => (args[0], pv),
        2 => (args[0], args[1]),
        n => return Err(Error::new(format!("requires 1 or 2 args, got {}", n))),
    };

    let version_parts = version_split(ver);
    let max_idx = version_parts.len();
    let (start, end) = parse::range(range, version_parts.len() / 2)?;
    let start_idx = match start {
        0 => 0,
        n => cmp::min(n * 2 - 1, max_idx),
    };
    let end_idx = cmp::min(end * 2, max_idx);
    println!("{}", &version_parts[start_idx..end_idx].join(""));

    Ok(0)
}
