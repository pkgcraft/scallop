use scallop::{builtins, shell};

fn main() {
    let internal_builtins = vec![
        &builtins::profile::BUILTIN,
        &builtins::command_not_found_handle::BUILTIN,
    ];
    builtins::register(internal_builtins).expect("failed loading builtins");

    shell::interactive()
}
