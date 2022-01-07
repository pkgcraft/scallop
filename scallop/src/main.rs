use std::process;

use scallop::bash::shell;

fn main() {
    process::exit(shell())
}
