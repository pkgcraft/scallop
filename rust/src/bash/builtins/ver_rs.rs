use pkgcraft::bash::{parse, version_split};

use crate::bash::string_value;
use crate::{Error, Result};

pub(crate) static SHORT_DOC: &str = "ver_rs 2 - 1.2.3";
pub(crate) static LONG_DOC: &str = "\
Perform string substitution on package version strings.

Returns -1 on error.";

#[doc = stringify!(LONG_DOC)]
pub fn ver_rs(args: &[&str]) -> Result<i32> {
    let pv = string_value("PV").unwrap_or("");
    let (ver, args) = match args.len() {
        n if n < 2 => return Err(Error::new(format!("requires 2 or more args, got {}", n))),

        // even number of args uses $PV
        n if n % 2 == 0 => (pv, args),

        // odd number of args uses the last arg as the version
        _ => (*args.last().unwrap(), &args[..args.len()-1]),
    };

    // Split version string into separators and components, note that the version string doesn't
    // have to follow the spec since args like ".1.2.3" are allowed.
    let mut version_parts = version_split(ver);

    // iterate over (range, separator) pairs
    let mut args_iter = args.chunks_exact(2);
    while let Some(&[range, sep]) = args_iter.next() {
        let (start, end) = parse::range(range, version_parts.len() / 2)?;
        for n in start..=end {
            let idx = n * 2;
            if idx < version_parts.len() {
                version_parts[idx] = sep;
            }
        }
    }

    println!("{}", version_parts.join(""));

    Ok(0)
}
