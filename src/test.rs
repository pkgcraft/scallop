#![cfg(test)]

use ctor::ctor;

use crate::shell::Shell;

/// Initialize bash for all test executables.
#[ctor]
fn initialize() {
    Shell::init();
}
