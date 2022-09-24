//! Structs related to the Tile Map, aka Screen Base Block.
use gba::mmio_types::TextTile;
use volmatrix::rw::{VolBlock, VolMatrix};

use crate::video::{
    mode::{Mode, TileMode},
    tile::{map, Drawable, Tile, SBB, SBB_SIZE},
    VideoControl,
};

#[cfg(doc)]
use crate::video::{colmod, tile::layer};

// TODO: probably should invert the indices here, so that
// higher allocation "spill down" to tile sprite data memory,
// rather than starting in the data memory.
/// A specific SBB slot.
///
/// See [`Handle`] for explanations on SBB.
#[derive(Clone, Copy)]
pub struct Slot(usize);
impl Slot {
    // TODO: handle different screen sizes based on tile mode
    /// Handle for a given sbb and screen size.
    pub(super) const fn handle<M: TileMode>(
        self,
        size: map::TextSize,
        ctrl: &mut VideoControl<M>,
    ) -> Handle<M> {
        Handle {
            _ctrl: ctrl,
            region: size.region(),
            sbb: self.index_volmatrix(SBB),
        }
    }
    /// Return value.
    pub(super) const fn get(&self) -> u16 {
        self.0 as u16
    }
    /// How many Sbb slot there is.
    pub const MAX_BLOCKS: usize = super::SBB_COUNT;
    /// The sbb slot of index `inner`.
    ///
    /// # Panics
    ///
    /// When `inner >= Self::MAX_BLOCKS`
    pub const fn new(inner: usize) -> Self {
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
    pub const _0: Self = unsafe { Self::new_unchecked(0) };
    pub const _1: Self = unsafe { Self::new_unchecked(1) };
    pub const _2: Self = unsafe { Self::new_unchecked(2) };
    pub const _3: Self = unsafe { Self::new_unchecked(3) };
    pub const _4: Self = unsafe { Self::new_unchecked(4) };
    pub const _5: Self = unsafe { Self::new_unchecked(5) };
    pub const _6: Self = unsafe { Self::new_unchecked(6) };
    pub const _7: Self = unsafe { Self::new_unchecked(7) };
    pub const _8: Self = unsafe { Self::new_unchecked(8) };
    pub const _9: Self = unsafe { Self::new_unchecked(9) };
    pub const _10: Self = unsafe { Self::new_unchecked(10) };
    pub const _11: Self = unsafe { Self::new_unchecked(11) };
    pub const _12: Self = unsafe { Self::new_unchecked(12) };
    pub const _13: Self = unsafe { Self::new_unchecked(13) };
    pub const _14: Self = unsafe { Self::new_unchecked(14) };
    pub const _15: Self = unsafe { Self::new_unchecked(15) };
    pub const _16: Self = unsafe { Self::new_unchecked(16) };
    pub const _17: Self = unsafe { Self::new_unchecked(17) };
    pub const _18: Self = unsafe { Self::new_unchecked(18) };
    pub const _19: Self = unsafe { Self::new_unchecked(19) };
    pub const _20: Self = unsafe { Self::new_unchecked(20) };
}

/// Write tiles to video memory at specific SBB offsets.
///
/// Called "Text BG Screen" or "BG Map" or "SC0, SC1 etc." in GBATEK.
///
/// The upper part of video memory holds tile map layout information.
/// An SBB (Screen Base Block) is a region of memory that
/// represents a map of tiles to be displayed.
///
/// There is normally only 6 SBBs in [`colmod::Bit8`], but seemingly, the GBA allows
/// the SBB memory to "spill down" the to the tile pixel data.
/// As long as you are not referencing higher id tiles, it should be fine.
///
/// Generally [`colmod::Bit4`] should be favored,
/// but the same tile can use different palettes, and you have much more SBB space.
///
/// You should use [`layer::Handle::set_sbb`] to set the SBB.
///
/// # Character Base Block
///
/// Character Base Block (or CBB) is similar to SBB, but controls the tile bitmap
/// information, the thing that is either encoded as .
pub struct Handle<'a, M: Mode> {
    _ctrl: &'a mut VideoControl<M>,
    sbb: VolBlock<TextTile, SBB_SIZE>,
    region: map::Rect,
}
impl<'a, M: TileMode> Handle<'a, M> {
    pub fn set_tile(&mut self, tile: Tile, pos: map::Pos) {
        // TODO: very poor perf, probably can make Pos const generic
        // over maximum sizes, so that access is compile-time checked.
        let to_set = self.sbb.index(pos.x + pos.y * self.region.width);
        to_set.write(tile.get());
    }
    pub fn clear_tiles(&mut self, offset: map::Pos, drawable: &impl Drawable) {
        drawable.for_each_clear_tile(|pos| {
            let pos = pos + offset;
            if self.region.contains(pos) {
                self.set_tile(Tile::EMPTY, pos);
            }
        });
    }
    pub fn set_tiles(&mut self, offset: map::Pos, drawable: &impl Drawable) {
        drawable.for_each_tile(|tile, pos| {
            let pos = pos + offset;
            if self.region.contains(pos) {
                self.set_tile(tile, pos);
            }
        });
    }
}
