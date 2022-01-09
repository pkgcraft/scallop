use crate::builtins::Builtin;
use crate::{Error, Result};

static LONG_DOC: &str = "\
Executed when the search for a command is unsuccessful.

This allows PATH search failures to be caught and handled within scallop instead of using the
command_not_found_handle() function method instead.
";

#[doc = stringify!(LONG_DOC)]
pub(crate) fn run(args: &[&str]) -> Result<i32> {
    let cmd = args[0];
    let full_cmd = args.join(" ");
    Err(Error::new(format!(
        "unknown command {:?} when executing: {}",
        cmd, full_cmd
    )))
}

pub static BUILTIN: Builtin = Builtin {
    name: "command_not_found_handle",
    func: run,
    help: LONG_DOC,
    usage: "for internal use only",
    exit_on_error: false,
};
