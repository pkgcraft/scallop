use scallop::builtins;
use scallop::shell::Shell;

fn main() {
    let sh = Shell::new("scallop");
    sh.builtins([&builtins::profile::BUILTIN, &builtins::command_not_found_handle::BUILTIN]);
    sh.interactive()
}
