//! Game loop code execution.
//!
//! Higher level API to deal with game loop,
//! proper handling of draw commands etc.
use core::{marker::PhantomData, mem};

use const_default::ConstDefault;
use gba::mmio_addresses::VCOUNT;

use crate::{
    input::{Input, KEYINPUT},
    video::{self, mode, object},
};

pub use crate::planckrand::{RandBitsIter, Rng};

pub enum EnterMode<T: ?Sized, F, G, H>
where
    F: FnOnce(&mut video::Control<mode::Text>, &T, &mut ConsoleState),
    G: FnOnce(&mut video::Control<mode::Mixed>, &T, &mut ConsoleState),
    H: FnOnce(&mut video::Control<mode::Affine>, &T, &mut ConsoleState),
{
    Text(F),
    Mixed(G),
    Affine(H),
    #[doc(hidden)]
    #[allow(non_camel_case_types)]
    _phantom(PhantomData<T>),
}
impl<T: ?Sized, F, G, H> EnterMode<T, F, G, H>
where
    F: FnOnce(&mut video::Control<mode::Text>, &T, &mut ConsoleState),
    G: FnOnce(&mut video::Control<mode::Mixed>, &T, &mut ConsoleState),
    H: FnOnce(&mut video::Control<mode::Affine>, &T, &mut ConsoleState),
{
    // TODO: inspect asm
    fn enter(self, mode: ControlModes, state: &T, console: &mut ConsoleState) -> ControlModes {
        macro_rules! execute_enter {
            (@branch $variant:ident, $ctrl:expr, $init:expr) => {{
                let mut new_mode = $ctrl.enter_mode::<mode::$variant>();
                $init(&mut new_mode, state, console);
                ControlModes::$variant(new_mode)
            }};
            ($current:ident, $ctrl:expr) => {
                match self {
                    EnterMode::Text(init) => execute_enter!(@branch Text, $ctrl, init),
                    EnterMode::Mixed(init) => execute_enter!(@branch Mixed, $ctrl, init),
                    EnterMode::Affine(init) => execute_enter!(@branch Affine, $ctrl, init),
                    EnterMode::_phantom(_) => ControlModes::$current($ctrl),
                }
            }
        }
        match mode {
            ControlModes::Text(ctrl) => execute_enter!(Text, ctrl),
            ControlModes::Mixed(ctrl) => execute_enter!(Mixed, ctrl),
            ControlModes::Affine(ctrl) => execute_enter!(Affine, ctrl),
        }
    }
}

enum ControlModes {
    Text(video::Control<mode::Text>),
    Mixed(video::Control<mode::Mixed>),
    Affine(video::Control<mode::Affine>),
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

// TODO: input latency is sooooo bad. What's the deal?
/// Global console state.
#[derive(ConstDefault)]
pub struct ConsoleState {
    /// The frame count.
    pub frame: usize,
    /// The button state
    pub input: Input,
    /// The object allocation state.
    pub(crate) objects: object::Allocator,
    /// A random number generator.
    /// Just set this with [`Rng::new`] to seed it.
    pub rng: Rng,
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
    /// Reserve an object slot.
    /// Returns `None` if no more slots are available.
    ///
    /// Make sure to call [`Self::free_object`] before dropping an [`object::Slot`],
    /// otherwise, the object slot will forever be leaked.
    #[must_use]
    pub fn reserve_object(&mut self) -> Option<object::Slot> {
        self.objects.reserve()
    }
    /// Free an object slot, consuming it.
    pub fn free_object(&mut self, slot: object::Slot) {
        self.objects.free(slot);
    }
}

type GsF<T> = fn(&mut video::Control<mode::Text>, &T, &mut ConsoleState);
type GsG<T> = fn(&mut video::Control<mode::Mixed>, &T, &mut ConsoleState);
type GsH<T> = fn(&mut video::Control<mode::Affine>, &T, &mut ConsoleState);
pub type GameStateEnterMode<T> = EnterMode<T, GsF<T>, GsG<T>, GsH<T>>;

/// The game definition.
///
/// All method take a `self`, the game state.
/// The `*_draw` methods are called when the GBA is in the corresponding
/// [`crate::video::Mode`]. The initial mode is [`mode::Text`] and must
/// be handled (if only to enter a different mode).
pub trait GameState {
    /// The game logic, updates the state based on input for current frame.
    fn logic(&mut self, console: &mut ConsoleState) -> Option<GameStateEnterMode<Self>>;

    /// Draw stuff in [`mode::Text`], text mode is the initial video mode.
    ///
    /// You must handle text mode, if only to setup a different mode you'll
    /// use for the rest of your game.
    fn text_draw(&self, console: &mut ConsoleState, video: &mut video::Control<mode::Text>);

    /// Draw stuff in [`mode::Mixed`], by default does nothing.
    fn mixed_draw(&self, console: &mut ConsoleState, video: &mut video::Control<mode::Mixed>) {
        let _ = (video, console);
    }

    /// Draw stuff in [`mode::Affine`], by default does nothing.
    fn affine_draw(&self, console: &mut ConsoleState, video: &mut video::Control<mode::Affine>) {
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
/// You must not have multiple concurrent instances of [`video::Control`]
/// existing at the same time.
///
/// This also access arbitrary memory addresses, so if ran outside of the GBA,
/// it basically is a big ball of segfaults.
///
/// You have been warned.
pub unsafe fn full_game<Stt: GameState>(mut state: Stt) -> ! {
    // SAFETY: upheld by function safety invariants.
    let mut video_control = ControlModes::Text(unsafe { video::Control::<mode::Text>::init() });
    let mut console = ConsoleState::DEFAULT;
    loop {
        console.input.previous = mem::replace(&mut console.input.current, KEYINPUT.read());
        console.frame = console.frame.wrapping_add(1);
        let mut enter_video_mode = state.logic(&mut console);

        spin_until_vblank();
        video_control = match enter_video_mode.take() {
            Some(mode) => mode.enter(video_control, &state, &mut console),
            None => video_control,
        };
        match &mut video_control {
            ControlModes::Text(video_control) => state.text_draw(&mut console, video_control),
            ControlModes::Mixed(video_control) => state.mixed_draw(&mut console, video_control),
            ControlModes::Affine(video_control) => state.affine_draw(&mut console, video_control),
        }
        spin_until_vdraw();
    }
}

#[allow(clippy::missing_const_for_fn, unused_variables, clippy::empty_loop)]
pub fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    #[cfg(feature = "log")]
    gba::fatal!("{}", info);
    // SAFETY: resets the whole console
    #[cfg(not(feature = "log"))]
    unsafe {
        use gba::{bios, mmio_types::ResetFlags};
        bios::RegisterRamReset(
            ResetFlags::new()
                .with_vram(true)
                .with_oam(true)
                .with_sio(true)
                .with_sound(true)
                .with_all_other_io(true),
        );
        bios::SoftReset()
    };
}
