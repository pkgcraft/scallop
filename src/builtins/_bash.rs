use crate::bash;
use crate::builtins::ExecStatus;
use crate::command::cmd_scope;
use crate::error::ok_or_error;
use crate::traits::*;

/// Run the `declare` builtin with the given arguments.
pub fn declare(args: &[&str]) -> crate::Result<ExecStatus> {
    let args = Words::from_iter(args.iter().copied());
    cmd_scope("declare", || unsafe {
        bash::declare_builtin((&args).into());
    });

    ok_or_error()
}

/// Run the `local` builtin with the given arguments.
pub fn local(args: &[&str]) -> crate::Result<ExecStatus> {
    let args = Words::from_iter(args.iter().copied());
    cmd_scope("local", || unsafe {
        bash::local_builtin((&args).into());
    });

    ok_or_error()
}

/// Run the `set` builtin with the given arguments.
pub fn set(args: &[&str]) -> crate::Result<ExecStatus> {
    let args = Words::from_iter(args.iter().copied());
    cmd_scope("set", || unsafe {
        bash::set_builtin((&args).into());
    });

    ok_or_error()
}

/// Run the `shopt` builtin with the given arguments.
pub fn shopt(args: &[&str]) -> crate::Result<ExecStatus> {
    let args = Words::from_iter(args.iter().copied());
    cmd_scope("shopt", || unsafe {
        bash::shopt_builtin((&args).into());
    });

    ok_or_error()
}

#[cfg(test)]
mod tests {
    use crate::functions::bash_func;
    use crate::variables::{bind, string_value};

    use super::*;

    #[test]
    fn test_local() {
        bind("VAR", "outer", None, None).unwrap();
        bash_func("func_name", || {
            local(&["VAR=inner"]).unwrap();
            assert_eq!(string_value("VAR").unwrap(), "inner");
        });
        assert_eq!(string_value("VAR").unwrap(), "outer");
    }
}
