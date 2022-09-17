# gssa-rs

The Rust port of Generic Space Shooter Advance (tm)

## Building

### Rust version

This is an old project on lifesupports. I've been developping this with
an antedeluvian version of rust:

- rustc 1.46.0-nightly (2020-06-08)
- cargo-xbuild 0.5.35
- arm-none-eabi-{ld,objdump} (GNU ld) 2.32

### Steps

1. Devkit-pro. See their [getting started page](https://devkitpro.org/wiki/Getting_Started).

2. `cargo-xbuild`

```sh
rustup update
rustup default nightly
rustup component add rust-src
cargo install cargo-xbuild
```

3. `make` and `find` (posix stuff)

4. A symbolic link to the `resources` folder of the gssa C code. (see [the project page](https://gitlab.com/nicopap/gssa/-/tree/master))
\
   It's actually quite easy! In the gssa project root:
\
```sh
cd png2gba
make
cd ..
make make_resources
```

5. a GBA emulator (`mgba` is currently hard-coded in the Makefile, and is
   currently the best GBA emulator there is, so strongly recommended)

6. `make run`

## Current features

Currently, the title screen shows up, and pressing "start" as the game suggests
stops the game execution.

This port aims to accomplish feature-parity and further with the original gssa.
The tricky bit being that the original being written in the dodgiest C possible,
I'm having constant trouble compiling it.
The highest point of functionality I managed to accomplish with the original C
code was:

- [ ] browse game menu, select ship
- [ ] start game with selected ship
- [ ] See game background scroll and palette cycle (with visible tearing)
- [ ] Pause and resume the game (allowing to move the "pause menu" message)
- [ ] spawn multiple waves of enemies on a timer, shooting bullets at player
- [ ] enemy AI capable of moving in set patterns, aiming and dodging the player
- [ ] being hit by enemy bullets and ships, lowering health and power level
      (the two being the same)
- [ ] random drops allowing to restore health
- [ ] random drop allowing to change weapon
- [ ] Game over screen similar to a panic

(checked are features implemented in the Rust version)

After updating my system, the game doesn't compiler anymore! I had to mark
some global variables as static (in the very original version, the global
variables were literally random memory location addresses cast to pointer)
I had to mark the global variables defined in a header file as `static`,
but seemingly it broken OAM, now I only manage to browse the menu,
chose a ship and start the game, get a scrolling space background,
but no space ships shows up on screen.
