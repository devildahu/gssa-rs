//! Game loop code execution.
//!
//! Higher level API to deal with game loop,
//! proper handling of draw commands etc.

use gba::mmio_addresses::{KEYINPUT, VCOUNT};

use crate::video::{mode, VideoControl};
use crate::Input;

enum ControlModes {
    Text(VideoControl<mode::Text>),
    Mixed(VideoControl<mode::Mixed>),
    Affine(VideoControl<mode::Affine>),
}

/// Performs a busy loop until VBlank starts.
///
/// This is very inefficient, and please keep following the lessons until we
/// cover how interrupts work!
fn spin_until_vblank() {
    while VCOUNT.read() < 160 {}
}

/// Performs a busy loop until VDraw starts.
///
/// This is very inefficient, and please keep following the lessons until we
/// cover how interrupts work!
fn spin_until_vdraw() {
    while VCOUNT.read() >= 160 {}
}

/// The game definition.
///
/// All method take a `self`, the game state.
/// The `*_draw` methods are called when the GBA is in the corresponding
/// [`crate::video::Mode`]. The initial mode is [`mode::Text`] and must
/// be handled (if only to enter a different mode).
pub trait GameState {
    /// The initial state to start the game with.
    const INITIAL: Self;

    /// The game logic, updates the state based on input for current frame.
    fn logic(&mut self, input: Input);

    /// Draw stuff in [`mode::Text`].
    fn text_draw(&self, video: &mut VideoControl<mode::Text>);

    /// Draw stuff in [`mode::Mixed`], by default does nothing.
    fn mixed_draw(&self, video: &mut VideoControl<mode::Mixed>) {
        let _ = video;
    }

    /// Draw stuff in [`mode::Affine`], by default does nothing.
    fn affine_draw(&self, video: &mut VideoControl<mode::Affine>) {
        let _ = video;
    }
}

/// Run [`GameState::logic`] and one of `GameState::*_draw` once
/// per frame.
///
/// For the `*_draw` family of functions, they will only be called
/// if the video mode is the one provided in argument.
///
/// # Safety
///
/// You must not have multiple concurrent instances of [`VideoControl`]
/// existing at the same time.
///
/// You have been warned.
pub unsafe fn full_game<Stt: GameState>() -> ! {
    // SAFETY: upheld by function safety invariants.
    let mut video_control = ControlModes::Text(unsafe { VideoControl::<mode::Text>::init() });
    let mut state = Stt::INITIAL;
    loop {
        state.logic(Input {
            keypad: KEYINPUT.read().into(),
        });

        spin_until_vblank();
        match &mut video_control {
            ControlModes::Text(video_control) => state.text_draw(video_control),
            ControlModes::Mixed(video_control) => state.mixed_draw(video_control),
            ControlModes::Affine(video_control) => state.affine_draw(video_control),
        }
        spin_until_vdraw();
    }
}

pub fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    gba::fatal!("{}", info)
}
