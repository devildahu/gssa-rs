//! High level API to safely acces video memory.
//!
//! To use this module, create a singleton [`VideoControl`] and use
//! methods on it.
// TODO: consider replacing SBB by "tile map" in public API.
// TODO: split this in multiple files.
// TODO: consider replacing the enum { _1, _2 ... } by a macro.
// TODO: consider having a const_generic for the textmode tile map width,
//       so that checks and computations are done at compile time.
// TODO: consider using a "video command" buffer, so that methods on
//       VideoControl can be called anytime, but will be submitted guarentee at
//       vblank with minimal memory moving.

use core::mem;
use core::{marker::PhantomData, slice};

use gba::mmio_addresses::{BG0CNT, BG1CNT, BG2CNT, BG3CNT, DISPCNT};
use gba::mmio_types::{BackgroundControl, DisplayControl, TextTile};

use crate::{
    assets::{Palette, Tileset, HARDCODED_TILEMAP_WIDTH},
    text::{self, draw::Pos},
    volmatrix::{VolAddress, VolBlock, VolMatrix, VolMemcopy},
};

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
/// for a different subset of video modes. Some methods return a `***Handle`,
/// which might contain the methods you are looking for. For example, to draw
/// something on-screen in [`Text`] mode, you should:
/// - call [`VideoControl::sbb`] to get a [`SbbHandle`],
/// - then call [`SbbHandle::set_tiles`] with the [`text::Draw`]able you want to display
/// - call [`VideoControl::layer`] to get a [`TextLayerHandle`],
/// - then call [`TextLayerHandle::set_sbb`] to set it to the SBB you just drew
///   your stuff to.
/// - (make sure also to call [`VideoControl::enable_layer`]) with the layer
///   you want to display)
pub(crate) struct VideoControl<M: Mode> {
    _t: PhantomData<fn() -> M>,
}

/// Background-specific types, used mostly with methods on `***Handle` or [`VideoControl`].
pub(crate) mod bg {
    use super::*;

    /// Background layer priority, lower is more in front.
    ///
    /// Used by [`TextLayerHandle`].
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    #[repr(u16)]
    pub(crate) enum Priority {
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

    // TODO: probably should invert the indices here, so that
    // higher allocation "spill down" to tile sprite data memory,
    // rather than starting in the data memory.
    /// A specific SBB slot.
    ///
    /// See [`super::SbbHandle`] for explanations on SBB.
    #[derive(Clone, Copy)]
    pub(crate) struct SbbSlot(usize);
    impl SbbSlot {
        /// Return value.
        pub(super) const fn get(&self) -> u16 {
            self.0 as u16
        }
        /// How many Sbb slot there is.
        pub(crate) const MAX_BLOCKS: usize = super::SBB_COUNT;
        pub(crate) const fn new(inner: usize) -> Self {
            assert!(inner < Self::MAX_BLOCKS);
            Self(inner)
        }
        /// SAFETY: `inner` must be lower than [`Self::MAX_BLOCKS`]
        pub(super) const unsafe fn new_unchecked(inner: usize) -> Self {
            Self(inner)
        }
        pub(super) const fn index_volmatrix<T, const C: usize>(
            self,
            volmatrix: VolMatrix<T, C, { Self::MAX_BLOCKS }>,
        ) -> VolBlock<T, C> {
            // SAFETY: It is impossible to build a SbbSlot of higher value than Self::MAX_BLOCK.
            unsafe { volmatrix.row_unchecked(self.0) }
        }

        // SAFETY: for all the following const definitions: all values are bellow Self::MAX_BLOCKS
        pub(crate) const _0: Self = unsafe { Self::new_unchecked(0) };
        pub(crate) const _1: Self = unsafe { Self::new_unchecked(1) };
        pub(crate) const _2: Self = unsafe { Self::new_unchecked(2) };
        pub(crate) const _3: Self = unsafe { Self::new_unchecked(3) };
        pub(crate) const _4: Self = unsafe { Self::new_unchecked(4) };
        pub(crate) const _5: Self = unsafe { Self::new_unchecked(5) };
        pub(crate) const _6: Self = unsafe { Self::new_unchecked(6) };
        pub(crate) const _7: Self = unsafe { Self::new_unchecked(7) };
        pub(crate) const _8: Self = unsafe { Self::new_unchecked(8) };
        pub(crate) const _9: Self = unsafe { Self::new_unchecked(9) };
        pub(crate) const _10: Self = unsafe { Self::new_unchecked(10) };
        pub(crate) const _11: Self = unsafe { Self::new_unchecked(11) };
        pub(crate) const _12: Self = unsafe { Self::new_unchecked(12) };
        pub(crate) const _13: Self = unsafe { Self::new_unchecked(13) };
        pub(crate) const _14: Self = unsafe { Self::new_unchecked(14) };
        pub(crate) const _15: Self = unsafe { Self::new_unchecked(15) };
        pub(crate) const _16: Self = unsafe { Self::new_unchecked(16) };
        pub(crate) const _17: Self = unsafe { Self::new_unchecked(17) };
        pub(crate) const _18: Self = unsafe { Self::new_unchecked(18) };
        pub(crate) const _19: Self = unsafe { Self::new_unchecked(19) };
        pub(crate) const _20: Self = unsafe { Self::new_unchecked(20) };
    }
    /// A specific CBB slot.
    ///
    /// See [`super::VideoControl::load_tileset`] for explanations on CBB.
    #[derive(Clone, Copy)]
    pub(crate) struct CbbSlot(usize);
    impl CbbSlot {
        /// Return value.
        pub(super) const fn get(&self) -> u16 {
            self.0 as u16
        }
        pub(super) const fn add(&self, offset: usize) -> Option<Self> {
            if self.0 + offset < Self::MAX_BLOCKS {
                // SAFETY: we make sure to not go over MAX_BLOCKS
                Some(unsafe { Self::new_unchecked(self.0 + offset) })
            } else {
                None
            }
        }
        /// How many Cbb slot there is.
        pub(crate) const MAX_BLOCKS: usize = super::CBB_COUNT;
        pub(crate) const fn new(inner: usize) -> Self {
            assert!(inner < Self::MAX_BLOCKS);
            Self(inner)
        }
        /// SAFETY: `inner` must be lower than [`Self::MAX_BLOCKS`]
        pub(super) const unsafe fn new_unchecked(inner: usize) -> Self {
            Self(inner)
        }
        pub(super) const fn index_volmatrix<T, const C: usize>(
            self,
            volmatrix: VolMatrix<T, C, { Self::MAX_BLOCKS }>,
        ) -> VolBlock<T, C> {
            // SAFETY: It is impossible to build a CbbSlot of higher value than Self::MAX_BLOCK.
            unsafe { volmatrix.row_unchecked(self.0) }
        }

        // SAFETY: for all the following const definitions: all values are bellow Self::MAX_BLOCKS
        pub(crate) const _0: Self = unsafe { Self::new_unchecked(0) };
        pub(crate) const _1: Self = unsafe { Self::new_unchecked(1) };
        pub(crate) const _2: Self = unsafe { Self::new_unchecked(2) };
        pub(crate) const _3: Self = unsafe { Self::new_unchecked(3) };
    }

    /// Background layers accessible in [`Text`] [`Mode`].
    ///
    /// To manipulate the background, get a [`TextLayerHandle`] from
    /// [`VideoControl<Text>::layer`] or [`VideoControl<Mixed>::text_layer`]
    /// and use the methods on [`TextLayerHandle`].
    #[derive(Clone, Copy)]
    #[repr(u16)]
    pub(crate) enum TextLayerSlot {
        _0 = 0,
        _1 = 1,
        _2 = 2,
        _3 = 3,
    }
    impl TextLayerSlot {
        pub(super) fn set_display(self, bit: bool, settings: DisplayControl) -> DisplayControl {
            match self {
                Self::_0 => settings.with_display_bg0(bit),
                Self::_1 => settings.with_display_bg1(bit),
                Self::_2 => settings.with_display_bg2(bit),
                Self::_3 => settings.with_display_bg3(bit),
            }
        }
        pub(super) const fn register(self) -> VolAddress<BackgroundControl> {
            match self {
                Self::_0 => BG0CNT,
                Self::_1 => BG1CNT,
                Self::_2 => BG2CNT,
                Self::_3 => BG3CNT,
            }
        }
    }

    /// Text background layers accessible in [`Mixed`] [`Mode`].
    ///
    /// To manipulate the background, get a [`TextLayerHandle`] from
    /// [`VideoControl<Mixed>::text_layer`]
    /// and use the methods on [`TextLayerHandle`].
    #[derive(Clone, Copy)]
    #[repr(u16)]
    pub(crate) enum MixedTextLayerSlot {
        _0 = 0,
        _1 = 1,
    }
    impl MixedTextLayerSlot {
        pub(super) const fn into_pure_text(self) -> TextLayerSlot {
            match self {
                Self::_0 => TextLayerSlot::_0,
                Self::_1 => TextLayerSlot::_1,
            }
        }
    }
}

/// traits to "seal" public traits in this module, to prevent
/// downstream implementation and exposing lower level implementation
/// details such as how memory is access in various video modes.
///
/// It also defines hardware representation of specific types.
mod sealed {
    /// Seal the [`super::Mode`] trait.
    pub(crate) trait Mode {
        /// The `u16` representation of the display mode, one of 0,1,2,4,5
        const RAW_REPR: u16;
    }
    /// Seal the [`super::ColorMode`] trait.
    pub(crate) trait ColorMode {
        /// The `bool` representation of the color mode. (`true` for [`super::Color8bit`]
        /// and `false` for [`super::Color4bit`])
        const RAW_REPR: bool;
    }
}

/// Video modes for use with [`VideoControl`].
///
/// | Mode  | Affine | Layers (BG*) | Res | Tiles/double buffering | Colors | Features | Implementation status|
/// |:-----:|:------:|:------------:|:---:|:----------------------:|:------:|:--------:|:--------------------:|
/// | [`Text`]          | No  | 0-3 | 256^2 to 512^2  | 1024  | 4bpp/8bpp | Scroll, Flip | Working
/// | [`Mixed`]         | BG2 | 0-2 | BG0/1 ↑, BG2 ↓  | ibid  | ibid      | ibid | Planned
/// | [`Affine`]        | Yes | 2,3 | 128^2 to 1024^2 | 256   | 4bpp/8bpp | Scroll | Planned
/// | `FullColorBitmap` | Yes | 2   | 240x160         | no    | RGB555    | - | Low priority
/// | `PaletteBitmap`   | Yes | 2   | 240x160         | dbuff | 4bpp      | - | Low priority
/// | `LowBitmap`       | Yes | 2   | 160x128         | dbuff | RGB555    | - | Low priority
///
/// See links to `Mode` implementors for fully detailed documentation.
///
/// `FullColorBitmap` allows just to set individual pixel color in memory, there
/// is no double-buffering meaning that you will have a hard time avoiding screen
/// tearing. But you could in theory display anything in this mode, theoretically
/// allows you to display at the same time 32768 colors on screen.
///
/// `PaletteBitmap` is like `FullColorBitmap`, but has two buffers, which means
/// you can work on one while the other is shown, then flipping between the two
/// buffers at your leisure. allowing you to avoid screen tearing.
/// However, in this mode, each pixel color is palette-indexed, allowing at most
/// 256 colors shown at the same time.
///
/// `LowBitmap` is like `FullColorBitmap`, but with limited resolution (160×128 pixels
/// rather than the GBA-native 240×160), this allows double-buffering with full color
/// range available.
pub(crate) trait Mode: sealed::Mode {}

/// Subset of [`Mode`]s that support tile-based access.
pub(crate) trait TileMode: Mode {}

/// Text mode, tile+map based background mode supporting 4 distinct background layers,
/// both [`Color4bit`] and [`Color8bit`] tile definition
/// and sprite flipping.
///
/// `Text` mode doesn't support rotation and scaling, this is strictly tile-based.
/// See [`Mixed`] for an alternative with rotation and scaling.
pub(crate) enum Text {}
impl Mode for Text {}
impl TileMode for Text {}
impl sealed::Mode for Text {
    const RAW_REPR: u16 = 0;
}

/// Mixed mode, tile+map based background mode controlled like [`Text`]
/// for background layers 0 and 1,
/// and [`Affine`] for layer 2.
pub(crate) enum Mixed {}
impl Mode for Mixed {}
impl TileMode for Mixed {}
impl sealed::Mode for Mixed {
    const RAW_REPR: u16 = 1;
}

// TODO: implement scaling and rotation
/// Affine mode, tile+map based mode, only supports 2 layers (2 and 3).
///
/// Also often refered as "Affine" mode, or "Rotation/Scaling Modes".
///
/// All layers in this mode support scaling and rotation,
/// but this is at the cost of some flexibility, notably:
/// - Only 8 bit color sprites are supported
/// - Only 2 layers are available
///
/// Affine mode supports larger tile maps than [`Text`] mode,
/// this is how a game like _Mario Kart Advance_ would be implemented.
pub(crate) enum Affine {}
impl Mode for Affine {}
impl TileMode for Affine {}
impl sealed::Mode for Affine {
    const RAW_REPR: u16 = 2;
}

/// The color mode of tile bitmap information.
///
/// Also commonly refered as `8bpp` or `bpp8`.
///
/// GBATEK calls the memory block that holds the tile bitmap information
/// `CHAR_BASE_BLOCK`.
///
/// The GBA allows specifying tile texture data in two ways
/// when in [`Text`] or [`Mixed`] mode:
///
/// * **[`Color8bit`]:** Each color index within a background or object is 8 bits. The 8
///   bits index into the full block of colors as if it was a single set of 256
///   elements. Index 0 is counted as the "transparency" index.
///   * `base_addr + size_of::<Color>() * index`
/// * **[`Color4bit`]:** Each color index within a background or object is 4 bits. The 4
///   bits within the image data is an entry into a "palette bank" of 16
///   entries. The palette bank used for the BG or OBJ overall is determined by
///   their control bits. Index 0 within a palbank is still considered the
///   "transparency" index.
///   * `base_addr + size_of::<Color>() * (pal_bank << 4 | entry)`
///
/// A series of 32×32 colors represent a [`crate::text::Tile`], individual tiles are
/// addressed by index in the [SBB](SbbHandle).
pub(crate) trait ColorMode: sealed::ColorMode {}

/// 8 bit color tiles video mode.
pub(crate) enum Color8bit {}
impl ColorMode for Color8bit {}
impl sealed::ColorMode for Color8bit {
    const RAW_REPR: bool = true;
}

/// 4 bit color tiles video mode.
///
/// Also commonly refered as `4bpp` or `bpp4`.
///
/// See [`ColorMode`] for details.
pub(crate) enum Color4bit {}
impl ColorMode for Color4bit {}
impl sealed::ColorMode for Color4bit {
    const RAW_REPR: bool = false;
}

/// TODO: define rotation/scaling manipulation.
pub(crate) type AffineLayerHandle<'a, M> = TextLayerHandle<'a, M>;

/// General `VideoControl` methods available in all [`Mode`]s.
impl<M: Mode> VideoControl<M> {
    // TODO: Consider doing something similar to TextLayerHandle::commit
    // to minimize memory access when possible.
    /// Enter new video mode.
    ///
    /// WARNING: this doesn't clean up video memory, so you'll probably
    /// see artifacts until you clear it up.
    pub(crate) fn enter_mode<N: Mode>(self) -> VideoControl<N> {
        let old_settings = DISPCNT.read();
        DISPCNT.write(old_settings.with_display_mode(N::RAW_REPR));
        VideoControl { _t: PhantomData }
    }
    pub(crate) fn enable_layer(&mut self, layer: bg::TextLayerSlot) {
        let old_settings = DISPCNT.read();
        DISPCNT.write(layer.set_display(true, old_settings));
    }
    pub(crate) fn reset_display_control(&mut self) {
        DISPCNT.write(DisplayControl::new().with_display_mode(M::RAW_REPR));
    }
    pub(crate) fn disable_layer(&mut self, layer: bg::TextLayerSlot) {
        let old_settings = DISPCNT.read();
        DISPCNT.write(layer.set_display(false, old_settings));
    }
}
/// SAFETY: the slice is intended to be converted to u16 with regard to endianness.
unsafe fn u8_to_u16_slice(u8s: &[u8]) -> &[u16] {
    let byte_len = u8s.len() >> 1;
    // SAFETY: byte_len is always less or equal to half the u8s size,
    // so the covered memory block is always within bounds of u8s
    unsafe { slice::from_raw_parts(u8s.as_ptr().cast(), byte_len) }
}
/// `VideoControl` methods for [tile](TileMode) [`Mode`] ([`Mixed`], [`Text`] and [`Affine`]).
impl<M: TileMode> VideoControl<M> {
    /// Load a [`Tileset`] into video memory.
    ///
    /// Each [layer](TextLayerHandle) may select one of four character base block (CBB),
    /// the CBB is the "tileset" or tile bitmap data. While the [SBB](SbbHandle) is
    /// the map, each entry an index into the CBB.
    pub(crate) fn load_tileset(&mut self, slot: bg::CbbSlot, tileset: &Tileset) {
        // SAFETY: Tileset can only be constructed in assets.rs and
        // data is special-prepared for this to work (ok, likely to fuck
        // up in tooling at one point, which means I should have runtime
        // check flags (TODO), but lol)
        let data = unsafe { u8_to_u16_slice(tileset.get()) };
        for (i, data) in data.chunks(CBB_SIZE).enumerate() {
            if let Some(cbb) = slot.add(i) {
                let cbb = cbb.index_volmatrix(TILE_IMG_DATA);
                cbb.write_slice(data);
            }
        }
    }
    // TODO: 16 colors version
    /// Load a 256 colors palette to the palette memory.
    pub(crate) fn load_palette(&mut self, palette: &Palette) {
        // SAFETY: Tileset can only be constructed in assets.rs and
        // data is special-prepared for this to work (ok, likely to fuck
        // up in tooling at one point, which means I should have runtime
        // check flags (TODO), but lol)
        let data = unsafe { u8_to_u16_slice(palette.get()) };
        PALRAM.write_slice(data);
    }
}

/// `VideoControl` methods exclusive to [`Mixed`] [`Mode`].
impl VideoControl<Mixed> {
    /// Obtain a [`SbbHandle`] to write tiles into a tile map.
    pub(crate) fn sbb(&mut self, slot: bg::SbbSlot) -> SbbHandle<Mixed> {
        SbbHandle::from_sbb_and_width(slot, HARDCODED_TILEMAP_WIDTH as usize, self)
    }

    /// Get handle to one of the two [`TextLayerHandle`] to manage it.
    pub(crate) fn text_layer(&mut self, slot: bg::MixedTextLayerSlot) -> TextLayerHandle<Mixed> {
        TextLayerHandle::new(self, slot.into_pure_text())
    }

    /// Get handle of the affine layer.
    pub(crate) fn affine_layer(&mut self) -> AffineLayerHandle<Mixed> {
        AffineLayerHandle::new(self, bg::TextLayerSlot::_2)
    }
}
/// `VideoControl` methods exclusive to [`Text`] [`Mode`].
impl VideoControl<Text> {
    /// Create an instance of `VideoControl`.
    ///
    /// # Safety
    ///
    /// There must be at most one `VideoControl` existing
    /// at the same time during the execution of the game.
    ///
    /// Failure to uphold this safety comment shouldn't result
    /// in undefined behavior, but will violate the basic Rust
    /// reference model.
    pub(crate) const unsafe fn init() -> Self {
        VideoControl { _t: PhantomData }
    }
    /// Obtain a [`SbbHandle`] to write tiles into a tile map.
    pub(crate) fn sbb(&mut self, slot: bg::SbbSlot) -> SbbHandle<Text> {
        SbbHandle::from_sbb_and_width(slot, HARDCODED_TILEMAP_WIDTH as usize, self)
    }

    /// Get handle to the provided layer.
    pub(crate) fn layer(&mut self, slot: bg::TextLayerSlot) -> TextLayerHandle<Text> {
        TextLayerHandle::new(self, slot)
    }
}

/// Background layer operations in [`Text`] or [`Mixed`] [`Mode`]s.
///
/// Note that the changes are only effective when the handle is dropped,
/// to avoid extraneous memory reads/writes.
pub(crate) struct TextLayerHandle<'a, M: Mode> {
    ctrl: &'a mut VideoControl<M>,
    value: BackgroundControl,
    register: VolAddress<BackgroundControl>,
}
impl<'a, M: TileMode> TextLayerHandle<'a, M> {
    fn new(ctrl: &'a mut VideoControl<M>, bg: bg::TextLayerSlot) -> Self {
        let register = bg.register();
        Self {
            ctrl,
            value: register.read(),
            register,
        }
    }
    /// Set priority of this layer, returning the previous priority.
    pub(crate) fn set_priority(&mut self, priority: bg::Priority) -> bg::Priority {
        let old_priority = unsafe {
            // SAFETY: return value of `bg_priority` is always `ret & 0b11`.
            bg::Priority::new_unchecked(self.value.priority() as u16)
        };
        self.value = self.value.with_priority(priority as u8);
        old_priority
    }
    /// Set SBB of this layer, returning the previous SBB.
    pub(crate) fn set_sbb(&mut self, sbb: bg::SbbSlot) -> bg::SbbSlot {
        let old_sbb = bg::SbbSlot::new(self.value.screen_base_block() as usize);
        self.value = self.value.with_screen_base_block(sbb.get() as u8);
        old_sbb
    }
    /// Set color mode of this layer.
    pub(crate) fn set_color_mode<CM: ColorMode>(&mut self) {
        self.value = self.value.with_is_8bpp(CM::RAW_REPR);
    }
}
impl<'a, M: Mode> TextLayerHandle<'a, M> {
    fn commit(&mut self) {
        self.register.write(self.value);
    }
}
impl<'a, M: Mode> Drop for TextLayerHandle<'a, M> {
    /// Commit all changes to video memory.
    fn drop(&mut self) {
        self.commit()
    }
}

// TODO: enum this, or even const-generic it.
pub(crate) type ScreenSize = usize;

/// The tile map (or [SBB](SbbHandle)) size for [`Text`] and [`Mixed`] [`Mode`]s.
///
/// GBATEK calls this "Screen Size."
#[repr(u16)]
pub(crate) enum TextMapSize {
    /// 32×32 tiles, or 256×256 pixels
    Base = 0,
    /// 64×32 tiles, or 512×256 pixels
    Long = 1,
    /// 32×64 tiles, or 256×512 pixels
    Tall = 2,
    /// 64×64 tiles, or 512×512 pixels
    Large = 3,
}
/// The tile map (or [SBB](SbbHandle)) size for [`Mixed`] and [`Affine`] [`Mode`]s.
///
/// GBATEK calls this "Screen Size."
#[repr(u16)]
pub(crate) enum AffineMapSize {
    /// 16×16 tiles, or 128×128 pixels
    Base = 0,
    /// 32×32 tiles, or 256×256 pixels
    Double = 1,
    /// 64×64 tiles, or 512×512 pixels
    Quad = 2,
    /// 128×128 tiles, or 1024×1024 pixels
    Octo = 3,
}

const SBB_SIZE: usize = 0x400;
const SBB_COUNT: usize = 32;
const CBB_SIZE: usize = 0x2000;
const CBB_COUNT: usize = 4;
const PALRAM_ADDR_USIZE: usize = 0x500_0000;
const VRAM_ADDR_USIZE: usize = 0x600_0000;
const PALRAM_SIZE: usize = 256;
// SAFETY:
// - VRAM_BASE_USIZE is non-zero
// - GBA VRAM bus size is 16 bits
// - TextTile is repr(transparent) on u16
// - the stack doesn't expand to VRAM, and we aren't used an allocator
// - GBA VRAM size is 0x10000 (2**16)
//   == 0x400 * size_of(Entry) * 32
//   == 0x2000 * size_of(u16) * 4
const SBB: VolMatrix<TextTile, SBB_SIZE, SBB_COUNT> = unsafe { VolMatrix::new(VRAM_ADDR_USIZE) };
// TODO: a type-safe struct for tile info
const TILE_IMG_DATA: VolMatrix<u16, CBB_SIZE, CBB_COUNT> =
    unsafe { VolMatrix::new(VRAM_ADDR_USIZE) };
// TODO: 4bpp mode palram
// SAFETY:
// - PALRAM_ADDR_USIZE is non-zero
// - u16 & PALRAM bus size is 16
// - PALRAM size is 1Kb < 2 * 256
const PALRAM: VolBlock<u16, PALRAM_SIZE> = unsafe { VolBlock::new(PALRAM_ADDR_USIZE) };

/// Write tiles to video memory at specific SBB offsets.
///
/// Called "Text BG Screen" or "BG Map" or "SC0, SC1 etc." in GBATEK.
///
/// The upper part of video memory holds tile map layout information.
/// An SBB (Screen Base Block) is a region of memory that
/// represents a map of tiles to be displayed.
///
/// There is normally only 6 SBBs in [`Color8bit`], but seemingly, the GBA allows
/// the SBB memory to "spill down" the to the tile pixel data.
/// As long as you are not referencing higher id tiles, it should be fine.
///
/// Generally [`Color4bit`] should be favored,
/// but the same tile can use different palettes, and you have much more SBB space.
///
/// You should use [`TextLayerHandle::set_sbb`] to set the SBB.
///
/// # Character Base Block
///
/// Character Base Block (or CBB) is similar to SBB, but controls the tile bitmap
/// information, the thing that is either encoded as .
pub(crate) struct SbbHandle<'a, M: Mode> {
    ctrl: &'a mut VideoControl<M>,
    sbb: VolBlock<TextTile, SBB_SIZE>,
    width: ScreenSize,
}
impl<'a, M: TileMode> SbbHandle<'a, M> {
    /// Handle for a given sbb and screen size,
    const fn from_sbb_and_width(
        sbb: bg::SbbSlot,
        width: ScreenSize,
        ctrl: &'a mut VideoControl<M>,
    ) -> Self {
        Self {
            ctrl,
            width,
            sbb: sbb.index_volmatrix(SBB),
        }
    }
    #[inline]
    pub(crate) fn set_tile(&mut self, pos: Pos, tile: text::Tile) {
        // TODO: very poor perf, probably can make Pos const generic
        // over maximum sizes, so that access is compile-time checked.
        let to_set = self.sbb.index(pos.x + pos.y * self.width);
        to_set.write(tile.get());
    }
    pub(crate) fn set_tiles(&mut self, pos: Pos, drawable: &impl text::Draw) {
        drawable.for_each_tile(pos, self.width, |tile, pos| {
            self.set_tile(pos, tile);
        });
    }
}
