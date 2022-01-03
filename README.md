# scallop

Scallop is a wrapper library for bash.

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
