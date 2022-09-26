# gssa-rs

The Rust port of Generic Space Shooter Advance (tm)

## Building

### Rust version

I've updated the project to use a more recent version of rust
(previous version was 1.46 for reference).

- rustc 1.66.0-nightly (nightly-2022-09-20-x86_64-unknown-linux-gnu)
- arm-none-eabi-{ld,objdump} (GNU ld) 2.32

### Steps

1. See the [`gba`] crate README for initial setup details
2. A symbolic link to the `resources` folder of the gssa C code. (see the
   [gssa page])
```sh
cd png2gba
make
cd ..
make make_resources
```

3. A GBA emulator (`mgba` is currently hard-coded in the Makefile, and is
   currently the best GBA emulator there is, so strongly recommended)
4. `cargo run`

You may also use the `Makefile` reciepes by installing a basic POSIX
environment:

- `sed`
- `make`
- Stuff you should have installed following the [`gba`] crate instructions

To inspect generated assembly, you may use [`cargo-show-asm`]. For inspecting
the assembly of a specific function, add the `#[inline(never)]` attribute to
the function and run the following command (with the last bit swapped with
the name of the function you want to inspect, use `cargo asm` without
specifying a function to inspect to see a list of inspectable functions):

```sh
cargo asm --att --no-default-features \
  --target thumbv4t-none-eabi \
  --rust "gssa_rust::game::mainmenu::MainmenuData::draw_title_screen"
```

## Current features

This port aims to accomplish feature-parity and further with the original gssa.
The tricky bit being that the original being written in the dodgiest C possible,
I'm having constant trouble compiling it.

The highest point of functionality I managed to accomplish with the original C
code was:

(checkmarked entries are implemented in Rust, non-checkmarked only exist in the
C version)

- [X] Split src/video_control.rs into an independent HAL (hardware abstraction layer)
   - [X] Update to latest rustc version, hopefully allowing me to use RA.
   - [X] volmatrix.rs as separate individual crate.
   - [X] derive macro for const DEFAULT (or use a 3rd party crate)
   - [X] const assert in various places where relevant.
- [X] Fix the text layouting algorithm
- [X] browse game menu, select ship
- [X] start game with selected ship
- [ ] Improve the HAL so that I can implement what the old version used to have
   - [X] Fix include_bytes! alignment for all resource types similarly to palette!
   - [ ] Add OBJ handling
   - [ ] Palette manager
   - [ ] Use interrupts over busy-looping for waiting VBLANK
- [ ] See game background scroll and palette cycle (with visible tearing)
- [ ] Pause and resume the game (allowing to move the "pause menu" message)
- [ ] spawn multiple waves of enemies on a timer, shooting bullets at player
- [ ] enemy AI capable of moving in set patterns, aiming and dodging the player
- [ ] being hit by enemy bullets and ships, lowering health and power level
      (the two being the same)
- [ ] random drops allowing to restore health
- [ ] random drop allowing to change weapon
- [ ] Complete console lockup with a blue screen saying "you ded" when game over

Following are probably never going to happen, but are long term potential
developpment for the game.

- [ ] Conditionally compile the rust-console/gba debugging facilities to elide
      them from the final binary when not using them.
- [ ] Proper game over screen with restart option and score.
- [ ] Game over screen similar to a panic
- [ ] Setup <https://github.com/est31/warnalyzer> for multi-crate dead code detection
- [ ] Improve the HAL so that it's possible to make a fully-featured game
   - [ ] Allow switching video mode in exec::GameState
   - [ ] proc/simple macro for generating _1, _2 etc. for typesafe registry values.
   - [ ] Tooling to generate 4bpp and 8bpp tilemaps and declare rust structs
         for image layouts, integrate palette management into asset pipeline
   - [ ] Including color cylcing (+blending) as described in the [effectgames article]
   - [ ] Pos with screen size encoded in the type as const generic, to avoid bound
         checking overhead
   - [ ] Potentially redesign the video HAL to minimize memory access, and add
         visibility into usage overhead.
   - [ ] Implement an audio layer HAL
   - [ ] Split GBA structs from other GBA stuff, for tooling development
- [ ] Use optimized memcpy intrisicts <https://hackmd.io/snq80PgDTPGeC4uzFg66Pw?view>
      and see agb impl: <https://github.com/agbrs/agb/tree/master/agb/src/agbabi>

(checked are features implemented in the Rust version)

After updating my system, the game doesn't compiler anymore! I had to mark
some global variables as static (in the very original version, the global
variables were literally random memory location addresses cast to pointer)
I had to mark the global variables defined in a header file as `static`,
but seemingly it broken OAM, now I only manage to browse the menu,
chose a ship and start the game, get a scrolling space background,
but no space ships shows up on screen.

[effectgames article]: http://www.effectgames.com/effect/article-Old_School_Color_Cycling_with_HTML5.html
[gssa page]: https://gitlab.com/nicopap/gssa/-/tree/master
[`gba`]: https://github.com/rust-console/gba
[`cargo-show-asm`]: https://crates.io/crates/cargo-show-asm
