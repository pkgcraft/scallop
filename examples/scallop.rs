use scallop::builtins;
use scallop::shell::Shell;

fn main() {
    let internal_builtins =
        vec![&builtins::profile::BUILTIN, &builtins::command_not_found_handle::BUILTIN];

    let sh = Shell::new("scallop", Some(internal_builtins));
    sh.interactive()
}
