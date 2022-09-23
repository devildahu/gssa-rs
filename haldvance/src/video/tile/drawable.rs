//! Draw images, tiles and text in  [`Text`]/[`Affine`] [`Mode`].
//!
//! [`Text`]: crate::video::mode::Text
//! [`Affine`]: crate::video::mode::Affine
//! [`Mode`]: crate::video::Mode

use super::{map::Pos, Tile};

#[cfg(doc)]
use crate::video::mode::{Affine, Mode, Text};

// TODO: fn size()
/// Something that can be drawn on a tilemap in [`Text`]/[`Affine`] [`Mode`].
///
/// To draw something, call [`super::sbb::Handle::set_tiles`]
/// with a `Drawable`. This trait only defines which tiles to place where on a map
/// for a given instance by calling a function once per relevant tile.
/// The lower level aspect of bit-fiddling with the mmio registers is left
/// to this crate's `set_tiles` implementation.
pub trait Drawable {
    /// Call `f` once per tile of self.
    fn for_each_tile<F: FnMut(Tile, Pos)>(&self, pos: Pos, screen_width: usize, f: F);
}

const ASCII_OFFSET: u8 = 0x20;
impl<'s> Drawable for &'s str {
    fn for_each_tile<F: FnMut(Tile, Pos)>(&self, mut pos: Pos, screen_width: usize, mut f: F) {
        self.bytes().for_each(|byte| {
            let tile = Tile::new(u16::from(byte - ASCII_OFFSET));
            f(tile, pos);
            pos.x += 1;
            if pos.x == screen_width {
                pos.x = 0;
                pos.y += 1;
            }
        });
    }
}
