use std::process;

use scallop::bash::shell;
use scallop::builtins;

fn main() {
    builtins::register(vec![builtins::profile::BUILTIN]).expect("failed loading builtins");
    process::exit(shell())
}
