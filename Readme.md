# gssa-rs

The Rust port of Generic Space Shooter Advance (tm)

## Building

### Rust version

I've updated the project to use a more recent version of rust
(previous version was 1.46 for reference).

- rustc 1.66.0-nightly (nightly-2022-10-12-x86_64-unknown-linux-gnu)
- arm-none-eabi-{ld,objdump} (GNU ld) 2.32

### Nightly

Nightly is required for the following:

The `haldvance` and `gbassets` crate requires:

- `const_mut_refs` as it heavily depends on `&mut` in `const fn`s.
- `const_type_id` for `Tileset` unique identifier generation, for the sprite
  manager (see [docs/Unique-tileset-ids.md] for reasoning)

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
the name of the function you want to inspect, use `cargo asm --target thumbv4t-none-eabi`
without specifying a function to inspect to see a list of inspectable functions):

```sh
cargo asm --att --no-default-features \
  --target thumbv4t-none-eabi \
  --rust "gssa_rust::game::mainmenu::MainmenuData::draw_title_screen"
```

## Architecture

The codebase is split in two (actually three) major components/crates:

- `haldvance`: a HAL (hardware abstraction layer) over the GBA hardware, to
  expose a rust-friendly API over the GBA crate. It is structured like a
  framework, in that it enforces a structure to the end-user code. This is
  because video operations **must** be ran during the HBLANK in order to avoid
  screen-tearing.
- `gbassets`: (currently very minor) an abstraction tile maps, images and
  palettes. This is meant to expose an API to a potential tile editor/converter
  to make it possible to author assets for your GBA games, export them and then
  inlcude them in your game in a type-safe maner.
- `src`: The actual game implementation and logic.

`include_const_aligned` and `volmatrix` are just helper crates.

### Documentation

Run `cargo doc --open` to access documentation. The `doc` directory contains
a copy of the [GBATEK] Lokathor-edited version, for offline usage. It also
contains reasoning and discussion on major architecture choice of the game. 

### Linting

All crates in this repository use the `#![warn(clippy::pendantic, clippy::nursery)]`
global attribute, the general goal is to have 0 warnings, but in this period
of rapid iteration, it is only 

### Testing

LOL 🤣.
You have to understand that interacting with the GBA means literally twiddling
bits in hardcoded addresses.
Short of implementing a full emulator, testing seems difficult.
Furthermore, games being by definition a constantly moving target
with an end product that do generally not require decades-long maintenance,
tests are much less "value add" than in other contexts.

Therefore, we rely heavily on the type system for corectness.
This might be unwise, given the amount of unsafe required to get things running
on the GBA, but there is not much choice in the matter.

### Debugging

The current intended way of debugging the game is using the
`haldvance::{warn,info,error,debug}` macros in combination with `mgba`, this
will print the messages in the terminal if `mgba` is launched from a terminal.

It is probably possible to setup a GDB connection with mgba, but I've never
managed to get it working, and I'm generally not comfortable with GDB.
If you are, it would be amazing if you shared resources on how to use GDB, and
provide hints on how to set it up 🙂

### Panicking

Another goal of gssa-rs is to:

- Take advantage of rust's compile time guarentees to avoid runtime bound checks
- **Never panic**. It's possible to tell where panics are possible by running
  `strings` on the build artifact ROM. There shouldn't be any panic anywhere.

A good embodiement of this philosophy is the `Drawable` trait, it exposes a
callback and iterator based API, give control to the implementor on how to
iterate over tiles yet still allowing the drawing code to basically be a tight
loop.

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
   - [X] Add OBJ handling
   - [X] Implement a sprite allocator for managing object sprite memory
   - [ ] Palette manager
   - [ ] Use interrupts over busy-looping for waiting VBLANK
- [X] See game background scroll (with visible tearing)
- [X] See player character (change depending on which selected)
- [X] Move player character
- [ ] Player can shoot bullets
- [ ] random drop allowing to change weapon
- [ ] random drops allowing to restore health
- [ ] Spawn Random enemies
- [ ] Spawn enemies by wave
- [ ] Cycle palette to give a shimering effect to bullets and background stars
- [ ] Pause and resume the game (allowing to move the "pause menu" message)
- [ ] spawn multiple waves of enemies on a timer, shooting bullets at player
- [ ] enemy AI capable of moving in set patterns, aiming and dodging the player
- [ ] being hit by enemy bullets and ships, lowering health and power level
      (the two being the same)
- [ ] Complete console lockup with a blue screen saying "you ded" when game over

Following are probably never going to happen, but are long term potential
developpment for the game.

- [ ] Consider using `static` instead of `const` for assets.
- [ ] Consider switching back to a flow-driven architecture, this avoids the
      constant diffing, since we do not "erase" the execution state.
- [ ] Conditionally compile the rust-console/gba debugging facilities to elide
      them from the final binary when not using them.
- [ ] Proper game over screen with restart option and score.
- [ ] Game over screen similar to a panic
- [ ] Setup <https://github.com/Technolution/rustig> or <https://github.com/viperproject/prusti-dev>
      for panic checking and other static analysis functionalities
- [ ] Improve the HAL so that it's possible to make a fully-featured game
   - [ ] Split implementation of `Image` between `Affine`/`Object` and `Text`,
     so that the image has a `const` representation equal to the value that will
     be uploaded to VRAM.
   - [X] Allow switching video mode in exec::GameState
   - [ ] proc/simple macro for generating _1, _2 etc. for typesafe registry values.
   - [ ] Tooling to generate 4bpp and 8bpp tilemaps and declare rust structs
         for image layouts, integrate palette management into asset pipeline
   - [ ] Including color cylcing (+blending) as described in the [effectgames article]
   - [ ] Pos with screen size encoded in the type as const generic, to avoid bound
         checking overhead
   - [ ] Potentially redesign the video HAL to minimize memory access, and add
         visibility into usage overhead.
   - [ ] Implement an audio layer HAL <https://maxmod.devkitpro.org/> <https://rentry.org/beepbox-gba-music>
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
[GBATEK]: https://rust-console.github.io/gbatek-gbaonly/
