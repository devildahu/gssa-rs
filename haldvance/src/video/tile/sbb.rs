//! Structs related to the Tile Map, aka Screen Base Block.
use gba::prelude::TextEntry;
use volmatrix::rw::{VolAddress, VolBlock, VolMatrix};

use crate::video::{
    self, mode,
    tile::{self, drawable, map, AffineEntry, Drawable, Tile, AFFINE_SBB, SBB_SIZE, TEXT_SBB},
};

#[cfg(doc)]
use crate::video::{
    colmod,
    tile::{cbb, layer},
};

// TODO: probably should invert the indices here, so that
// higher allocation "spill down" to tile sprite data memory,
// rather than starting in the data memory.
/// A specific SBB slot.
///
/// See [`TextHandle`] for explanations on SBB.
#[derive(Clone, Copy)]
pub struct Slot(usize);
impl Slot {
    /// [`TextHandle`] for a given sbb and screen size.
    pub(super) const fn text_handle<M: mode::Tile>(
        self,
        size: map::TextSize,
        ctrl: &mut video::Control<M>,
    ) -> TextHandle {
        TextHandle {
            _ctrl: ctrl.erased(),
            size,
            sbb: self.index_volmatrix(TEXT_SBB),
        }
    }
    /// [`TextHandle`] for a given sbb and screen size.
    pub(super) const fn affine_handle<M: mode::Tile>(
        self,
        size: map::AffineSize,
        ctrl: &mut video::Control<M>,
    ) -> AffineHandle {
        AffineHandle { _ctrl: ctrl.erased(), size, sbb: self.0 }
    }
    /// Return value.
    ///
    /// By definition, the return value is smaller than `Self::MAX_BLOCKS`.
    #[allow(clippy::cast_possible_truncation)]
    pub(super) const fn get(self) -> u16 {
        self.0 as u16
    }

    /// How many Sbb slot there is.
    pub const MAX_BLOCKS: usize = super::SBB_COUNT;

    /// The sbb slot of index `inner`.
    ///
    /// # Panics
    ///
    /// (const time) When `inner >= Self::MAX_BLOCKS`
    #[must_use]
    pub const fn new(inner: usize) -> Self {
        assert!(inner < Self::MAX_BLOCKS);
        Self(inner)
    }
    /// # Safety
    /// `inner` must be lower than [`Self::MAX_BLOCKS`]
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
    pub const _21: Self = unsafe { Self::new_unchecked(21) };
    pub const _22: Self = unsafe { Self::new_unchecked(22) };
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
/// Character Base Block (or [cbb]) is similar to SBB, but controls the tile bitmap
/// information.
pub struct TextHandle<'a> {
    _ctrl: &'a mut (),
    size: map::TextSize,
    sbb: VolBlock<TextEntry, SBB_SIZE>,
}
impl<'a> TextHandle<'a> {
    pub fn set_tile(&mut self, tile: Tile, pos: map::Pos) {
        // TODO: very poor perf, probably can make Pos const generic
        // over maximum sizes, so that access is compile-time checked.
        let voladdress_index = pos.x + pos.y * self.size.width();
        let to_set = self.sbb.index(voladdress_index as usize);
        to_set.write(tile.get());
    }
    pub fn clear_tiles(&mut self, offset: map::Pos, drawable: &impl Drawable) {
        drawable.all_tiles(|pos| {
            self.set_tile(Tile::EMPTY, pos + offset);
        });
    }
    pub fn set_tiles(&mut self, offset: map::Pos, drawable: &impl Drawable) {
        drawable.for_each_line(|pos, iter| {
            for (tile, x) in iter.zip(0_u16..) {
                let pos = map::Pos::x(x) + pos + offset;
                if self.size.region().contains(pos) {
                    self.set_tile(tile, pos);
                }
            }
        });
    }
}

/// Get an arbitrary affine SBB adress.
///
/// # Safety
///
/// Adress must be within vram.
const unsafe fn affine_sbb_at(sbb: usize, offset: usize) -> VolAddress<AffineEntry> {
    // SAFETY: function safety requirements
    unsafe { AFFINE_SBB.get_unchecked(offset, sbb) }
}

/// Same as [`TextHandle`], but for [`mode::Affine`] layers.
///
/// The two layer types differs in that a tile is encoded as a 8 bits value
/// in [`mode::Affine`] , while in [`mode::Text`], it is held on 16 bits,
/// including some meta data like horizontal/vertical flip.
///
/// This doesn't seem like a big deal, but the GBA `vram` has a 16 bits bus
/// size, meaning that you **cannot** independently set individual bytes (8 bits
/// values). Hence warranting a completely different API.
///
/// Thankfully, just using [`AffineHandle::set_tiles`] should let you use
/// an API similar to [`TextHandle`] without thinking much about it.
pub struct AffineHandle<'a> {
    _ctrl: &'a mut (),
    size: map::AffineSize,
    sbb: usize,
}
impl<'a> AffineHandle<'a> {
    /// Get the declared map size of this SBB.
    #[must_use]
    #[inline]
    pub const fn size(&self) -> map::AffineSize {
        self.size
    }
    /// # Safety
    ///
    /// `offset` must be within bounds of the sbb
    unsafe fn set_couple_unchecked(&mut self, to_set: AffineEntry, offset: u16) {
        // SAFETY: method safety requirements
        let address = unsafe { affine_sbb_at(self.sbb, (offset / 2) as usize) };
        address.write(to_set);
    }
    /// # Safety
    ///
    /// `offset` must be within bounds of the sbb
    unsafe fn set_unique_unchecked(&mut self, to_set: u8, offset: u16) {
        // SAFETY: method safety requirements
        let address = unsafe { affine_sbb_at(self.sbb, (offset / 2) as usize) };
        let mut previous = address.read();
        if offset % 2 == 0 {
            previous.set_left(to_set);
        } else {
            previous.set_right(to_set);
        }
        address.write(previous);
    }
    /// Draws a line.
    ///
    /// When going out of bound of [`Self::size`], crop.
    pub fn set_line(&mut self, pos: map::Pos, iter: impl Iterator<Item = u8>) {
        let y_oob = pos.y >= self.size.region().height;
        let x_oob = pos.x >= self.size.region().width;
        if y_oob || x_oob {
            return;
        }
        let max_line_len = (self.size.region().width - pos.x) as usize;

        let mut iter = iter.take(max_line_len);
        let mut offset = pos.x + pos.y * self.size.width();

        let is_even = offset % 2 == 0;
        if !is_even {
            let right = match iter.next() {
                Some(right) => right,
                None => return, // Nothing to draw this line.
            };
            // SAFETY: both x and y are within the `self.size.region()`
            unsafe { self.set_unique_unchecked(right, offset) };
            offset += 1;
        }
        for entry in tile::entrify(iter) {
            // SAFETY: both x and y are within the `self.size.region()`
            // AND the line is cropped to not exceed `self.size.region().width`
            match entry {
                Ok(entry) => unsafe { self.set_couple_unchecked(entry, offset) },
                Err(left) => unsafe { self.set_unique_unchecked(left, offset) },
            }
            offset += 2;
        }
    }
    /// Draw the [`Drawable`] where top-left tile is `offset`.
    ///
    /// Note that if the drawable spills out of this [`AffineHandle::size`],
    /// then it will be cropped to the bounds of this SBB's region.
    pub fn set_tiles(&mut self, offset: map::Pos, drawable: &impl Drawable) {
        drawable.for_each_line(|pos, iter| {
            // TODO: usage of .get().tile_index(), consider a different
            // trait for affine tilemaps.
            let iter = iter.map(|t| t.get().tile_index() as u8);
            self.set_line(pos + offset, iter);
        });
    }
    pub fn clear_tiles(&mut self, offset: map::Pos, drawable: &impl Drawable) {
        self.set_tiles(offset, &drawable::Clear(drawable));
    }
}
