# scallop

Scallop is a C library for bash.

## Development

To build scallop, run the following commands:

```bash
# clone repos
git clone https://github.com/pkgcraft/scallop.git
cd scallop
git clone https://github.com/pkgcraft/bash.git

# build libbash
./scripts/build-libbash -j16

# build scallop
meson setup build && meson compile -C build -v
```
