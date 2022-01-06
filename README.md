# scallop

Scallop is a rust-based library and executable that wrap bash. It enables
writing bash builtins natively in rust and running them in either standard bash
(with loadable builtin support enabled) or via the scallop executable.

## Development

Developing scallop assumes that recent versions of cargo, rust, and meson are
installed along with a standard C compiler.

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

## Usage

See the following example using the **profile** and **ver_test** builtins.

```bash
# if not installed, tell the linker where to find the scallop library
export LD_LIBRARY_PATH=$PWD/build

# use the profile builtin to benchmark the ver_test builtin
bash -c "enable -f target/debug/libscallop_builtins.so profile ver_test && profile ver_test 1.2.3 -lt 1.2.3_p"
```
