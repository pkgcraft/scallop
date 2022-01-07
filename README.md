# scallop

Scallop is a rust-based library and executable that wrap bash. It enables
writing bash builtins natively in rust and running them in either standard bash
(with loadable builtin support enabled) or via the scallop executable.

## Development

Developing scallop requires recent versions of cargo, rust, meson, and ninja
are installed along with a standard C compiler.

To build scallop, run the following commands:

```bash
git clone --recurse-submodules https://github.com/pkgcraft/scallop.git
cd scallop
cargo build
```
