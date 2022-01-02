use crate::bash::builtins::Builtin;
use crate::Result;

static LONG_DOC: &str = "\
Profile a given function's processor time usage.

Options:
    -n loops    execute command for a given number of loops
    -s seconds  execute command for a given number of seconds";

#[doc = stringify!(LONG_DOC)]
pub(crate) fn run(_args: &[&str]) -> Result<i32> {
    Ok(0)
}

pub static BUILTIN: Builtin = Builtin {
    name: "profile",
    func: run,
    help: LONG_DOC,
    usage: "profile [-n loops] [-s seconds] func arg1 arg2",
};
