# gssa-rs

The Rust port of Generic Space Shooter Advance (tm)

## Building

1. Devkit-pro. See their [getting started page](https://devkitpro.org/wiki/Getting_Started).

2. `cargo-xbuild`

3. `make` and `find` (posix stuff)

4. A symbolic link to the `resources` folder of the gssa C code. (see [the project page](https://gitlab.com/nicopap/gssa/-/tree/master))
  (I'm sorry, it's an utter mess to generate those resource files)

5. a GBA emulator (`gba-qt` is currently hard-coded in the Makefile)

6. `make run`
