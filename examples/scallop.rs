use scallop::{builtins, Shell};

fn main() {
    // initialize shell
    Shell::init();

    // load and enable builtins
    let builtins = vec![builtins::profile::BUILTIN, builtins::command_not_found_handle::BUILTIN];
    builtins::register(&builtins);
    builtins::enable(&builtins).expect("failed enabling builtins");

    // run shell
    Shell::interactive()
}
