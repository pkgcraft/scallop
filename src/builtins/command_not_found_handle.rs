use crate::builtins::{make_builtin, ExecStatus};
use crate::{Error, Result};

static LONG_DOC: &str = "\
Executed when the search for a command is unsuccessful.

This allows PATH search failures to be caught and handled within scallop instead of using the
command_not_found_handle() function method instead.
";

#[doc = stringify!(LONG_DOC)]
pub(crate) fn run(args: &[&str]) -> Result<ExecStatus> {
    let cmd = args[0];
    let full_cmd = args.join(" ");
    Err(Error::Base(format!("unknown command {cmd:?} when executing: {full_cmd}")))
}

make_builtin!(
    "command_not_found_handle",
    command_not_found_handle_builtin,
    run,
    LONG_DOC,
    "for internal use only"
);
