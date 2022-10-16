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
pub mod object;
pub mod palette;
pub mod tile;

use core::hint::unreachable_unchecked;
use core::marker::PhantomData;
use core::mem;

use gba::mmio_addresses::DISPCNT;
use gba::mmio_types::DisplayControl;
use volmatrix::VolMemcopy;

use tile::{Color, OBJ_PALRAM};

pub use colmod::ColorMode;
pub use mode::Mode;
pub use tile::set::Tileset;
pub use tile::Tile;

// pub use tile::map::Tilemap;

/// Controls video memory in text mode.
///
/// `VideoControl` is parametrized over [`M: Mode`](Mode).
///
/// `M` reflects the current display mode, each display mode has a very different
/// API, yet manipulates the same memory region. This is fundamentally unsafe,
/// but it's yet possible to write a safe API abstraction over it.
///
/// # How to read this doc page
///
/// Methods on `VideoControl` are divided in many different `impl` blocks. Each
/// for a different subset of video modes. You can use the `[+]` at the left
/// of `impl` to hide methods for specific video modes.
///
/// # How to use `VideoControl`
///
/// Some methods return a `*::Handle`,
/// which might contain the methods you are looking for. For example, to draw
/// something on-screen in [`mode::Text`] mode, you should:
/// - Use [`VideoControl::load_palette`] to load a palette
/// - Use [`VideoControl::load_tileset`] to load a tileset
/// - call [`VideoControl::sbb`] to get a [`tile::sbb::TextHandle`],
/// - Use [`tile::sbb::TextHandle::set_tiles`] with the [`tile::Drawable`]
///   you want to display
/// - Use [`VideoControl::layer`] to get a [`tile::layer::Handle`],
/// - Use [`tile::layer::Handle::set_sbb`] to set it to the SBB you just drew
///   your stuff to.
/// - (make sure to use [`VideoControl::enable_layer`]) with the layer
///   you want to display)
pub struct VideoControl<M: Mode> {
    _t: PhantomData<fn() -> M>,
    inner: (),
}

/// General `VideoControl` methods available in all [`Mode`]s.
impl<M: Mode> VideoControl<M> {
    const fn new() -> Self {
        Self { _t: PhantomData, inner: () }
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
    #[must_use]
    pub const unsafe fn init() -> VideoControl<mode::Text> {
        VideoControl::<mode::Text>::new()
    }

    // TODO: Consider doing something similar to TextLayerHandle::commit
    // to minimize memory access when possible.
    /// Enter new video mode.
    ///
    /// WARNING: this doesn't clean up video memory, so you'll probably
    /// see artifacts until you clear it up.
    #[must_use]
    pub fn enter_mode<N: Mode>(self) -> VideoControl<N> {
        let old_settings = DISPCNT.read();
        DISPCNT.write(old_settings.with_display_mode(N::TYPE as u16));
        VideoControl::new()
    }

    pub fn enable_layer(&mut self, layer: Layer<M>) {
        let old_settings = DISPCNT.read();
        DISPCNT.write(layer.set_display(true, old_settings));
    }

    pub fn disable_layer(&mut self, layer: Layer<M>) {
        let old_settings = DISPCNT.read();
        DISPCNT.write(layer.set_display(false, old_settings));
    }

    pub fn reset_display_control(&mut self) {
        DISPCNT.write(DisplayControl::new().with_display_mode(M::TYPE as u16));
    }

    pub fn set_object_tile_mapping(&mut self, mapping: object::TileMapping) {
        let old_settings = DISPCNT.read();
        DISPCNT.write(old_settings.with_obj_vram_1d(mapping.is_1d()));
    }

    pub fn enable_objects(&mut self) {
        let old_settings = DISPCNT.read();
        DISPCNT.write(old_settings.with_display_obj(true));
    }

    pub fn disable_objects(&mut self) {
        let old_settings = DISPCNT.read();
        DISPCNT.write(old_settings.with_display_obj(false));
    }

    /// Internal function to erase the type parameter.
    const fn erased(&mut self) -> &mut () {
        &mut self.inner
    }

    /// Obtain a [`object::Handle`] to manage objects.
    pub fn object<'a>(&'a mut self, slot: &object::Slot) -> object::Handle<'a> {
        object::Handle::new(self, slot)
    }
    // TODO: special method for palette::Bank type
    /// Load a palette to the object palette memory.
    pub fn load_object_palette(&mut self, offset: usize, palette: &[Color]) {
        OBJ_PALRAM.write_slice_at_offset(offset, palette);
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
/// Identify a global-level layer, eg. in [`VideoControl::enable_layer`].
///
/// This struct is only used for enabling/disabling layers.
/// See [`tile::layer::Slot`] and methods accepting `Slot` for more controls.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Layer<M: Mode> {
    value: u16,
    _t: PhantomData<fn() -> M>,
}
impl<M: Mode> Layer<M> {
    const fn set_display(self, bit: bool, settings: DisplayControl) -> DisplayControl {
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

/// Priority, lower is more in front.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u16)]
pub enum Priority {
    _0 = 0,
    _1 = 1,
    _2 = 2,
    _3 = 3,
}
impl Priority {
    /// Construct a priority from dynamic value without bound checks.
    ///
    /// Favor using the enum variants if the priority is known at compile time.
    ///
    /// # SAFETY
    ///
    /// `priority` must be 0, 1, 2 or 3.
    pub(super) const unsafe fn new_unchecked(priority: u16) -> Self {
        // SAFETY: Priority is repr(u16), and less than 4 as upheld by
        // function's SAFETY section.
        mem::transmute(priority)
    }
}
