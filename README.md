# scallop

Scallop is a rust-based library that wraps bash. It supports writing bash
builtins in rust and interacting with various bash data structures including
variables, arrays, and functions.

## Development

Developing scallop requires recent versions of cargo and rust are installed
along with a standard C compiler.

Note that using `cargo nextest` or another test runner that runs tests in
separate processes is required, using `cargo test` will break as long as it
uses threads since bash isn't thread-friendly in any fashion.

To build scallop, run the following commands:

```bash
git clone --recurse-submodules https://github.com/pkgcraft/scallop.git
cd scallop
cargo build
```
