//! Deal with tile-based GBA video modes, see [`Mode`].
pub mod cbb;
pub mod drawable;
pub mod layer;
pub mod map;
pub mod sbb;
pub mod set;

use core::mem;

use gba::mmio_types::TextEntry;
use volmatrix::{
    rw::{VolBlock, VolMatrix},
    VolMemcopy,
};

use crate::{
    sane_assert,
    video::{
        colmod,
        mode::{self, Affine, Mixed, Text},
        object, palette, VideoControl,
    },
};

#[cfg(doc)]
use crate::video::Mode;

pub use drawable::Drawable;
pub use gba::mmio_types::Color;
pub use set::Tileset;

#[repr(C)]
#[repr(align(2))]
#[derive(Clone, Copy)]
struct AffineEntry {
    left: u8,
    right: u8,
}
fn align_2<'a, I: Iterator + 'a>(
    mut iter: I,
) -> impl Iterator<Item = Result<(I::Item, I::Item), I::Item>> + 'a {
    core::iter::from_fn(move || {
        let current = iter.next();
        let next = iter.next();
        match (current, next) {
            (Some(current), Some(next)) => Some(Ok((current, next))),
            (Some(current), None) => Some(Err(current)),
            (None, _) => None,
        }
    })
}
fn entrify<'a, I: Iterator<Item = u8> + 'a>(
    iter: I,
) -> impl Iterator<Item = Result<AffineEntry, u8>> + 'a {
    align_2(iter).map(|r| r.map(|(left, right)| AffineEntry { left, right }))
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
// - TextEntry is repr(transparent) on u16
// - the stack doesn't expand to VRAM, and we do not use an allocator
// - GBA VRAM size is 0x10000 (2**16)
//   == 0x400 * size_of(Entry) * 32
//   == 0x2000 * size_of(u16) * 4
const TEXT_SBB: VolMatrix<TextEntry, SBB_SIZE, SBB_COUNT> =
    unsafe { VolMatrix::new(VRAM_ADDR_USIZE) };
const AFFINE_SBB: VolMatrix<AffineEntry, SBB_SIZE, SBB_COUNT> =
    unsafe { VolMatrix::new(VRAM_ADDR_USIZE) };
// TODO: a type-safe struct for tile info
const TILE_IMG_DATA: VolMatrix<u16, CBB_SIZE, CBB_COUNT> =
    unsafe { VolMatrix::new(VRAM_ADDR_USIZE) };
// TODO: 4bpp mode palram
// SAFETY:
// - PALRAM_ADDR_USIZE is non-zero
// - repr(u16) Color & BG_PALRAM bus size is 16
// - BG_PALRAM size is 1Kb == 4 * 256
const BG_PALRAM: VolBlock<Color, PALRAM_SIZE> = unsafe { VolBlock::new(PALRAM_ADDR_USIZE) };
pub(super) const OBJ_PALRAM: VolBlock<Color, PALRAM_SIZE> =
    unsafe { VolBlock::new(PALRAM_ADDR_USIZE + PALRAM_SIZE * mem::size_of::<Color>()) };

/// A tile for [`sbb::TextHandle::set_tile`].
#[derive(Clone, Copy)]
pub struct Tile(TextEntry);
impl Tile {
    pub const EMPTY: Self = Self::new(0);

    #[must_use]
    pub const fn new(tile_id: u16) -> Self {
        Self(TextEntry::new().with_tile_index(tile_id))
    }
    #[must_use]
    pub const fn flip_hori(self) -> Self {
        Self(self.0.with_hflip(!self.0.hflip()))
    }
    #[must_use]
    pub const fn flip_vert(self) -> Self {
        Self(self.0.with_vflip(!self.0.vflip()))
    }
    /// In [`colmod::Bit4`] mode, each individual [`Tile`]
    /// has at most 16 colors, but the palette for each tile can be
    /// specified in the tilemap [`Tile`] data.
    ///
    /// This has no effect if the color mode of the background is [`colmod::Bit8`].
    #[must_use]
    pub const fn with_palette(self, palette: palette::BankHandle) -> Self {
        Self(self.0.with_palbank_index(palette.id))
    }
    pub(crate) const fn get(self) -> TextEntry {
        self.0
    }
}

/// `VideoControl` methods exclusive to [`Text`] [`Mode`].
impl VideoControl<Text> {
    /// Get the requested [`layer::Handle`].
    pub fn layer(&mut self, slot: layer::Slot) -> layer::Handle<Text> {
        layer::Handle::new(self, slot)
    }
    /// Obtain a [`sbb::TextHandle`] to write tiles into a tile map.
    pub const fn sbb(&mut self, slot: sbb::Slot, map_size: map::TextSize) -> sbb::TextHandle {
        slot.text_handle(map_size, self)
    }
    /// Equivalent to `self.sbb(map::TextSize::Base, slot)`, see [`Self::sbb`].
    pub const fn basic_sbb(&mut self, slot: sbb::Slot) -> sbb::TextHandle {
        slot.text_handle(map::TextSize::Base, self)
    }
}

/// `VideoControl` methods exclusive to [`Mixed`] [`Mode`].
impl VideoControl<Mixed> {
    /// Get handle to one of the two [`layer::Handle`] to manage it.
    pub fn text_layer(&mut self, slot: layer::MixedSlot) -> layer::Handle<Text> {
        layer::Handle::new(self, slot.into_pure_text())
    }

    /// Get handle of the affine layer.
    pub fn affine_layer(&mut self) -> layer::Handle<Affine> {
        layer::Handle::new(self, layer::Slot::_2)
    }
    /// Obtain a [`sbb::AffineHandle`] to write tiles into a tile map.
    pub const fn affine_sbb(
        &mut self,
        slot: sbb::Slot,
        map_size: map::AffineSize,
    ) -> sbb::AffineHandle {
        slot.affine_handle(map_size, self)
    }
    /// Equivalent to `self.affine_sbb(map::AffineSize::Base, slot)`, see [`Self::sbb`].
    pub const fn basic_affine_sbb(&mut self, slot: sbb::Slot) -> sbb::AffineHandle {
        slot.affine_handle(map::AffineSize::Base, self)
    }
    /// Obtain a [`sbb::TextHandle`] to write tiles into a tile map.
    pub const fn text_sbb(&mut self, slot: sbb::Slot, map_size: map::TextSize) -> sbb::TextHandle {
        slot.text_handle(map_size, self)
    }
    /// Equivalent to `self.text_sbb(map::TextSize::Base, slot)`, see [`Self::sbb`].
    pub const fn basic_text_sbb(&mut self, slot: sbb::Slot) -> sbb::TextHandle {
        slot.text_handle(map::TextSize::Base, self)
    }
}

/// `VideoControl` methods exclusive to [`Affine`] [`Mode`].
impl VideoControl<Affine> {
    /// Get handle of the affine layer.
    pub fn layer(&mut self, slot: layer::AffineSlot) -> layer::Handle<Affine> {
        layer::Handle::new(self, slot.into_pure_text())
    }
    /// Obtain a [`sbb::AffineHandle`] to write tiles into a tile map.
    pub const fn sbb(&mut self, slot: sbb::Slot, map_size: map::AffineSize) -> sbb::AffineHandle {
        slot.affine_handle(map_size, self)
    }
    /// Equivalent to `self.sbb(map::AffineSize::Base, slot)`, see [`Self::sbb`].
    pub const fn basic_sbb(&mut self, slot: sbb::Slot) -> sbb::AffineHandle {
        slot.affine_handle(map::AffineSize::Base, self)
    }
}

/// `VideoControl` methods for [tile](mode::Tile) [`Mode`] ([`Mixed`], [`Text`] and [`Affine`]).
impl<M: mode::Tile> VideoControl<M> {
    /// Load a [`Tileset`] into video memory.
    ///
    /// Each [layer](layer::Handle) may select one of four character base block (CBB),
    /// the CBB is the "tileset" or tile bitmap data. While the [SBB](sbb::Handle) is
    /// the map, each entry an index into the CBB.
    pub fn load_tileset(&mut self, slot: cbb::Slot, tileset: &Tileset<colmod::Bit8>) {
        let data = tileset.get();
        for (i, data) in data.chunks(CBB_SIZE).enumerate() {
            if let Some(cbb) = slot.add(i) {
                let cbb = cbb.index_volmatrix(TILE_IMG_DATA);
                cbb.write_slice(data);
            }
        }
    }
    // TODO: Type safety with the various types in palette module
    /// Load a palette to the background palette memory.
    pub fn load_palette(&mut self, palette: &[Color]) {
        BG_PALRAM.write_slice(palette);
    }
    /// Return a [`object::Tile`] to use with [`object::Handle::set_tile`].
    ///
    /// # Panics (`sane_assert`)
    ///
    /// If `index >= 1024`.
    #[must_use]
    pub const fn object_tile(&self, index: u16) -> object::Tile {
        sane_assert!(index < 1024);
        object::Tile::new(index)
    }
}
