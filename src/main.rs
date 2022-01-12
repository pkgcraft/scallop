use scallop::{builtins, Shell};

fn main() {
    let internal_builtins = vec![
        &builtins::profile::BUILTIN,
        &builtins::command_not_found_handle::BUILTIN,
    ];

    let sh = Shell::new("scallop", internal_builtins).expect("failed initializing shell");
    sh.interactive()
}
