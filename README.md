# scallop

Rust-based wrapper library and executable for bash. It enables writing bash
builtins natively in rust and running them in either standard bash (with
loadable builtin support enabled) or via the scallop executable.

## Development

To build scallop, run the following commands:

```bash
# clone repos
git clone --recurse-submodules https://github.com/pkgcraft/scallop.git
cd scallop

# build libbash
./scripts/build-libbash -j16

# build scallop
meson setup build && meson compile -C build -v

# build rust support
cargo build
```
