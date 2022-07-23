use scallop::{builtins, Shell};

fn main() {
    Shell::init();
    Shell::builtins(&[builtins::profile::BUILTIN, builtins::command_not_found_handle::BUILTIN]);
    Shell::interactive()
}
