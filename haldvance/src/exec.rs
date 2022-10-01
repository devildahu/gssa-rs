//! Game loop code execution.
//!
//! Higher level API to deal with game loop,
//! proper handling of draw commands etc.
use core::mem;

use const_default::ConstDefault;
use gba::mmio_addresses::VCOUNT;

use crate::{
    input::{Input, KEYINPUT},
    video::{mode, object, VideoControl},
};

enum ControlModes {
    Text(VideoControl<mode::Text>),
    Mixed(VideoControl<mode::Mixed>),
    Affine(VideoControl<mode::Affine>),
}

/// Performs a busy loop until vertical blank starts.
///
/// This is very inefficient, and please keep following the lessons until we
/// cover how interrupts work!
fn spin_until_vblank() {
    while VCOUNT.read() < 160 {}
}

/// Performs a busy loop until vertical draw starts.
///
/// This is very inefficient, and please keep following the lessons until we
/// cover how interrupts work!
fn spin_until_vdraw() {
    while VCOUNT.read() >= 160 {}
}

/// Global console state.
#[derive(ConstDefault)]
pub struct ConsoleState {
    /// The frame count.
    pub frame: usize,
    /// The button state
    pub input: Input,
    /// The object allocation state.
    pub object_allocator: object::Allocator,
}
impl ConsoleState {
    /// Run `f` once every `frequency` frame, with given `offset`.
    ///
    /// # Performance
    ///
    /// This performs much better if `frequency` is a power of two.
    pub fn every<F: FnOnce(&Self)>(&self, offset: isize, frequency: usize, f: F) {
        let offset = self.frame.wrapping_add_signed(offset);
        (offset % frequency == 0).then(|| f(self));
    }
}

/// The game definition.
///
/// All method take a `self`, the game state.
/// The `*_draw` methods are called when the GBA is in the corresponding
/// [`crate::video::Mode`]. The initial mode is [`mode::Text`] and must
/// be handled (if only to enter a different mode).
pub trait GameState {
    /// The game logic, updates the state based on input for current frame.
    fn logic(&mut self, console: &mut ConsoleState);

    /// Draw stuff in [`mode::Text`], text mode is the initial video mode.
    ///
    /// You must handle text mode, if only to setup a different mode you'll
    /// use for the rest of your game.
    fn text_draw(&self, console: &mut ConsoleState, video: &mut VideoControl<mode::Text>);

    /// Draw stuff in [`mode::Mixed`], by default does nothing.
    fn mixed_draw(&self, console: &mut ConsoleState, video: &mut VideoControl<mode::Mixed>) {
        let _ = (video, console);
    }

    /// Draw stuff in [`mode::Affine`], by default does nothing.
    fn affine_draw(&self, console: &mut ConsoleState, video: &mut VideoControl<mode::Affine>) {
        let _ = (video, console);
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
pub unsafe fn full_game<Stt: GameState>(mut state: Stt) -> ! {
    // SAFETY: upheld by function safety invariants.
    let mut video_control = ControlModes::Text(unsafe { VideoControl::<mode::Text>::init() });
    let mut console = ConsoleState::DEFAULT;
    loop {
        console.input.previous = mem::replace(&mut console.input.current, KEYINPUT.read());

        state.logic(&mut console);
        console.frame = console.frame.wrapping_add(1);

        spin_until_vblank();
        match &mut video_control {
            ControlModes::Text(video_control) => state.text_draw(&mut console, video_control),
            ControlModes::Mixed(video_control) => state.mixed_draw(&mut console, video_control),
            ControlModes::Affine(video_control) => state.affine_draw(&mut console, video_control),
        }
        spin_until_vdraw();
    }
}

pub fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    gba::fatal!("{}", info)
}
