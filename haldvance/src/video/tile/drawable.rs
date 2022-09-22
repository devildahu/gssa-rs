//! Draw images, tiles and text in  [`Text`]/[`Affine`] [`Mode`].
//!
//! [`Text`]: crate::video::mode::Text
//! [`Affine`]: crate::video::mode::Affine
//! [`Mode`]: crate::video::Mode

use super::{map::Pos, Tile};

/// Something that can be drawn on a tilemap in [`Text`]/[`Affine`] [`Mode`].
///
/// To draw something, call [`super::sbb::Handle::set_tiles`]
/// with a `Drawable`. This trait only defines which tiles to place where on a map
/// for a given instance by calling a function once per relevant tile.
/// The lower level aspect of bit-fiddling with the mmio registers is left
/// to this crate's `set_tiles` implementation.
///
/// [`Text`]: crate::video::mode::Text
/// [`Affine`]: crate::video::mode::Affine
/// [`Mode`]: crate::video::Mode
pub trait Drawable {
    /// Call `f` once per tile of self.
    fn for_each_tile<F: FnMut(Tile, Pos)>(&self, pos: Pos, width: usize, f: F);
}

const ASCII_OFFSET: u8 = 0x20;
impl<'s> Drawable for &'s str {
    fn for_each_tile<F: FnMut(Tile, Pos)>(&self, mut pos: Pos, width: usize, mut f: F) {
        self.bytes().for_each(|byte| {
            let tile = Tile::new((byte - ASCII_OFFSET) as u16);
            f(tile, pos);
            pos.x += 1;
            if pos.x == width {
                pos.x = 0;
                pos.y += 1;
            }
        });
    }
}
