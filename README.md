# scallop

Scallop is a rust-based library that wraps bash. It supports writing bash
builtins in rust and interacting with various bash data structures including
variables, arrays, and functions.

## Development

Developing scallop requires recent versions of cargo and rust are installed
along with a standard C compiler. Additionally, meson and ninja are required
for shared library support.

To build scallop, run the following commands:

```bash
git clone --recurse-submodules https://github.com/pkgcraft/scallop.git
cd scallop
cargo build
```
