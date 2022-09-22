pub mod cbb;
mod drawable;
pub mod layer;
pub mod map;
pub mod sbb;
pub mod set;

use core::{mem, slice};

use gba::mmio_types::TextTile;
use volmatrix::{
    rw::{VolBlock, VolMatrix},
    VolMemcopy,
};

use crate::video::{
    colmod,
    mode::{Affine, Mixed, Text, TileMode},
    palette, VideoControl,
};

pub use drawable::Drawable;
pub use gba::mmio_types::Color;
pub use set::Tileset;

/// A tile for [`sbb::Handle::set_tile`].
#[derive(Clone, Copy)]
pub struct Tile(TextTile);
impl Tile {
    pub const fn new(tile_id: u16) -> Self {
        Self(TextTile::from_tile_id(tile_id))
    }
    pub const fn flip_hori(self) -> Self {
        Self(self.0.with_hflip(!self.0.hflip()))
    }
    pub const fn flip_vert(self) -> Self {
        Self(self.0.with_vflip(!self.0.vflip()))
    }
    /// In [`crate::video::colmod::Bit4`] mode, each individual `Tile`
    /// has at most 16 colors, but the palette for each tile can be
    /// specified in the tilemap `Tile` data.
    ///
    /// This has no effect if the color mode of the background is `Bit8`.
    pub const fn with_palette(self, palette: palette::BankHandle) -> Self {
        Self(self.0.with_palbank(palette.id))
    }
    pub(crate) const fn get(self) -> TextTile {
        self.0
    }
}

/// `VideoControl` methods exclusive to [`Text`] [`Mode`].
impl VideoControl<Text> {
    /// Get the requested [`layer::Handle`].
    pub fn layer(&mut self, slot: layer::Slot) -> layer::Handle<Text> {
        layer::Handle::new(self, slot)
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
    /// Each [layer](layer::Handle) may select one of four character base block (CBB),
    /// the CBB is the "tileset" or tile bitmap data. While the [SBB](sbb::Handle) is
    /// the map, each entry an index into the CBB.
    pub fn load_tileset(&mut self, slot: cbb::Slot, tileset: &Tileset<colmod::Bit8>) {
        // SAFETY: TODO
        let data = unsafe { u8_to_u16_slice(tileset.get()) };
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
    /// Obtain a [`sbb::Handle`] to write tiles into a tile map.
    pub fn sbb(&mut self, slot: sbb::Slot) -> sbb::Handle<M> {
        slot.handle(map::HARDCODED_TILEMAP_WIDTH as usize, self)
    }
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
// - the stack doesn't expand to VRAM, and we do not use an allocator
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
// - repr(u16) Color & BG_PALRAM bus size is 16
// - BG_PALRAM size is 1Kb == 4 * 256
const BG_PALRAM: VolBlock<Color, PALRAM_SIZE> = unsafe { VolBlock::new(PALRAM_ADDR_USIZE) };
const _OBJ_PALRAM: VolBlock<Color, PALRAM_SIZE> =
    unsafe { VolBlock::new(PALRAM_ADDR_USIZE + PALRAM_SIZE * mem::size_of::<Color>()) };
