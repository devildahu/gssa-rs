//! High level API to safely acces video memory.
//!
//! To use this module, create a singleton [`VideoControl`] and use
//! methods on it.
// TODO: consider replacing SBB by "tile map" in public API.
// TODO: consider replacing the enum { _1, _2 ... } by a macro.
// TODO: consider having a const_generic for the textmode tile map width,
//       so that checks and computations are done at compile time.
// TODO: consider using a "video command" buffer, so that methods on
//       VideoControl can be called anytime, but will be submitted guarentee at
//       vblank with minimal memory moving.

pub mod colmod;
pub mod mode;
pub mod palette;
pub mod tile;

use core::hint::unreachable_unchecked;
use core::marker::PhantomData;

use gba::mmio_addresses::DISPCNT;
use gba::mmio_types::DisplayControl;

pub use colmod::ColorMode;
pub use mode::Mode;
pub use tile::set::Tileset;
pub use tile::Tile;
// pub use tile::map::Tilemap;

/// Controls video memory in text mode.
///
/// `VideoControl` is a zero-sized type (meaning it has no runtime representation)
/// parametrized over [`M: Mode`](Mode).
///
/// `M` reflects the current display mode, each display mode has a very different
/// API, yet manipulates the same memory region. This is fundamentally unsafe,
/// but it's yet possible to write a safe API abstraction over it.
///
/// # How to read this doc page
///
/// Methods on `VideoControl` are divided in many different `impl` blocks. Each
/// for a different subset of video modes. Some methods return a `*::Handle`,
/// which might contain the methods you are looking for. For example, to draw
/// something on-screen in [`mode::Text`] mode, you should:
/// - call [`VideoControl::sbb`] to get a [`tile::sbb::Handle`],
/// - then call [`tile::sbb::Handle::set_tiles`] with the [`tile::Drawable`]
///   you want to display
/// - call [`VideoControl::layer`] to get a [`tile::layer::Handle`],
/// - then call [`tile::layer::Handle::set_sbb`] to set it to the SBB you just drew
///   your stuff to.
/// - (make sure also to call [`VideoControl::enable_layer`]) with the layer
///   you want to display)
pub struct VideoControl<M: Mode> {
    _t: PhantomData<fn() -> M>,
    _ref: (),
}

/// General `VideoControl` methods available in all [`Mode`]s.
impl<M: Mode> VideoControl<M> {
    const fn new() -> Self {
        Self {
            _t: PhantomData,
            _ref: (),
        }
    }

    /// Create an instance of `VideoControl`.
    ///
    /// Note that if you are using [`crate::exec::full_game`],
    /// you should not call this method!
    ///
    /// # Safety
    ///
    /// There must be at most one `VideoControl` existing
    /// at the same time during the execution of the game.
    ///
    /// Failure to uphold this safety comment shouldn't result
    /// in undefined behavior, but will violate the basic Rust
    /// reference model.
    pub const unsafe fn init() -> VideoControl<mode::Text> {
        VideoControl::<mode::Text>::new()
    }

    // TODO: Consider doing something similar to TextLayerHandle::commit
    // to minimize memory access when possible.
    /// Enter new video mode.
    ///
    /// WARNING: this doesn't clean up video memory, so you'll probably
    /// see artifacts until you clear it up.
    pub fn enter_mode<N: Mode>(self) -> VideoControl<N> {
        let old_settings = DISPCNT.read();
        DISPCNT.write(old_settings.with_display_mode(N::RAW_REPR));
        VideoControl::new()
    }

    pub fn enable_layer(&mut self, layer: Layer<M>) {
        let old_settings = DISPCNT.read();
        DISPCNT.write(layer.set_display(true, old_settings));
    }

    pub fn reset_display_control(&mut self) {
        DISPCNT.write(DisplayControl::new().with_display_mode(M::RAW_REPR));
    }

    pub fn disable_layer(&mut self, layer: Layer<M>) {
        let old_settings = DISPCNT.read();
        DISPCNT.write(layer.set_display(false, old_settings));
    }

    /// Internal function to erase the type parameter.
    fn erased<'a>(&'a mut self) -> &'a mut () {
        &mut self._ref
    }
}
macro_rules! layer_const {
    ($($name:ident => $value:expr;)* $(;)?) => {
        $( pub const $name : Self = Self {
            value: $value,
            _t: PhantomData,
        };)*
    }
}
#[repr(transparent)]
pub struct Layer<M: Mode> {
    value: u16,
    _t: PhantomData<fn() -> M>,
}
impl<M: Mode> Layer<M> {
    fn set_display(&self, bit: bool, settings: DisplayControl) -> DisplayControl {
        match self.value {
            0 => settings.with_display_bg0(bit),
            1 => settings.with_display_bg1(bit),
            2 => settings.with_display_bg2(bit),
            3 => settings.with_display_bg3(bit),
            // SAFETY: it is impossible to build a `Layer` with a value
            // different than 0, 1, 2 or 3
            _ => unsafe { unreachable_unchecked() },
        }
    }
}
impl Layer<mode::Text> {
    layer_const! {
        _0 => 0;
        _1 => 1;
        _2 => 2;
        _3 => 3;
    }
}
impl Layer<mode::Mixed> {
    layer_const! {
        _0 => 0;
        _1 => 1;
        _2 => 2;
    }
}
impl Layer<mode::Affine> {
    layer_const! {
        _2 => 2;
        _3 => 3;
    }
}
