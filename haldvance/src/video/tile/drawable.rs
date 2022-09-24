//! Draw images, tiles and text in  [`Text`]/[`Affine`] [`Mode`].
//!
//! [`Text`]: crate::video::mode::Text
//! [`Affine`]: crate::video::mode::Affine
//! [`Mode`]: crate::video::Mode

use const_default::ConstDefault;

use super::{
    map::{Pos, Rect},
    Tile,
};

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
    fn for_each_tile<F: FnMut(Tile, Pos)>(&self, f: F);
    fn for_each_clear_tile<F: FnMut(Pos)>(&self, mut f: F) {
        self.for_each_tile(|_tile, pos| f(pos));
    }
}

const ASCII_OFFSET: u8 = 0x20;
impl<'s> Drawable for &'s str {
    fn for_each_tile<F: FnMut(Tile, Pos)>(&self, mut f: F) {
        let mut pos = Pos::DEFAULT;
        let line_ending = b'\n';
        self.bytes().for_each(|byte| {
            if byte == line_ending {
                pos.x = 0;
                pos.y += 1;
            } else {
                let tile = Tile::new(u16::from(byte - ASCII_OFFSET));
                f(tile, pos);
                pos.x += 1;
            }
        });
    }
}

/// Draws `T` limiting it only to the specified `window` area.
pub struct Windowed<T: Drawable> {
    pub inner: T,
    pub window: Rect,
}
impl<T: Drawable> Drawable for Windowed<T> {
    fn for_each_tile<F: FnMut(Tile, Pos)>(&self, mut f: F) {
        self.inner.for_each_tile(|tile, pos| {
            if self.window.contains(pos) {
                f(tile, pos);
            }
        });
    }

    fn for_each_clear_tile<F: FnMut(Pos)>(&self, f: F) {
        EmptyRect(self.window).for_each_clear_tile(f);
    }
}

/// Draws an empty line of length COLUMNS.
pub struct ConstEmptyLine<const COLUMNS: usize>;

/// Draws an empty line of given length.
pub struct EmptyLine(pub usize);

/// An empty rectangular region of the screen.
pub struct EmptyRect(pub Rect);

/// Compile-time empty rectangular region of the screen.
pub struct ConstEmptyRect<const WIDTH: usize, const HEIGHT: usize>;

impl<const C: usize> Drawable for ConstEmptyLine<C> {
    fn for_each_tile<F: FnMut(Tile, Pos)>(&self, mut f: F) {
        (0..C).for_each(|x| f(Tile::EMPTY, Pos { x, y: 0 }));
    }
}
impl Drawable for EmptyLine {
    fn for_each_tile<F: FnMut(Tile, Pos)>(&self, mut f: F) {
        (0..self.0).for_each(|x| f(Tile::EMPTY, Pos { x, y: 0 }));
    }
}
impl<const W: usize, const H: usize> Drawable for ConstEmptyRect<W, H> {
    fn for_each_tile<F: FnMut(Tile, Pos)>(&self, mut f: F) {
        (0..H).for_each(|y| (0..W).for_each(|x| f(Tile::EMPTY, Pos { x, y })));
    }
}
impl Drawable for EmptyRect {
    fn for_each_tile<F: FnMut(Tile, Pos)>(&self, mut f: F) {
        let h = self.0.height;
        let w = self.0.width;
        (0..h).for_each(|y| (0..w).for_each(|x| f(Tile::EMPTY, Pos { x, y })));
    }
}
