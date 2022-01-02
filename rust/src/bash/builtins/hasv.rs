use super::has::has;
use crate::Result;

pub(crate) static LONG_DOC: &str = "The same as has, but also prints the first argument if found.";
pub(crate) static SHORT_DOC: &str = "hasv needle ${haystack}";

#[doc = stringify!(LONG_DOC)]
pub fn hasv(args: &[&str]) -> Result<i32> {
    let ret = has(args)?;
    if ret == 0 {
        println!("{}", args[0]);
    }

    Ok(ret)
}
