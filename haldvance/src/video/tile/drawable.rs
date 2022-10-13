//! Draw images, tiles and text in  [`Text`]/[`Affine`] [`Mode`].
//!
//! [`Text`]: crate::video::mode::Text
//! [`Affine`]: crate::video::mode::Affine
//! [`Mode`]: crate::video::Mode

use core::{iter, slice};

use const_default::ConstDefault;

use super::{
    map::{Pos, Rect},
    Tile,
};

#[cfg(doc)]
use crate::video::{
    mode::{Affine, Mode, Text},
    tile::sbb,
};

// TODO: fn size()
/// Something that can be drawn on a tilemap in [`Text`]/[`Affine`] [`Mode`].
///
/// To draw something, call [`sbb::TextHandle::set_tiles`] with a `Drawable`.
/// This trait only defines which tiles to place where on a map
/// for a given instance by calling a function once per relevant tile.
/// The lower level aspect of bit-fiddling with the mmio registers is left
/// to this crate's `set_tiles` implementation.
pub trait Drawable {
    type Iter: Iterator<Item = Tile>;

    /// Call `f` once per line of self.
    fn for_each_line<F: FnMut(Pos, Self::Iter)>(&self, f: F);

    fn all_tiles<F: FnMut(Pos)>(&self, mut f: F) {
        self.for_each_line(|pos, iter| {
            // TODO: use size_hint() here instead of iter.count() somehow
            (0_u16..)
                .take(iter.count())
                .for_each(|x| f(pos + Pos::x(x)));
        });
    }
}

const ASCII_OFFSET: u8 = b' ';
impl<'s> Drawable for &'s str {
    type Iter = iter::Map<slice::Iter<'s, u8>, fn(&u8) -> Tile>;

    fn for_each_line<F: FnMut(Pos, Self::Iter)>(&self, mut f: F) {
        let bytes = self.as_bytes().split(|b| *b == b'\n').zip(0_u16..);
        bytes.for_each(|(bytes, y)| {
            let byte_to_tile: fn(&u8) -> Tile = |byte| Tile::new(u16::from(byte - ASCII_OFFSET));
            let tiles = bytes.iter().map(byte_to_tile);
            f(Pos::y(y), tiles);
        });
    }
}

impl<'a, T: Drawable> Drawable for &'a T {
    type Iter = T::Iter;

    fn for_each_line<F: FnMut(Pos, Self::Iter)>(&self, f: F) {
        T::for_each_line(self, f);
    }
}

/// Same as `T`, but drawing, acts like clearing `T`.
pub struct Clear<T: Drawable>(pub T);
impl<T: Drawable> Drawable for Clear<T> {
    type Iter = iter::Map<T::Iter, fn(Tile) -> Tile>;

    fn for_each_line<F: FnMut(Pos, Self::Iter)>(&self, mut f: F) {
        self.0
            .for_each_line(|pos, iter| f(pos, iter.map(|_| Tile::EMPTY)));
    }
}

/// Draws `T` limiting it only to the specified `window` area.
pub struct Windowed<T: Drawable> {
    pub inner: T,
    pub window: Rect,
}
impl<T: Drawable> Drawable for Windowed<T> {
    type Iter = iter::Take<T::Iter>;

    fn for_each_line<F: FnMut(Pos, Self::Iter)>(&self, mut f: F) {
        self.inner.for_each_line(|pos, iter| {
            let relative_width = self.window.width - pos.x;
            f(pos, iter.take(usize::from(relative_width)));
        });
    }

    fn all_tiles<F: FnMut(Pos)>(&self, f: F) {
        EmptyRect(self.window).all_tiles(f);
    }
}

/// Draws an empty line of length COLUMNS.
pub struct ConstEmptyLine<const COLUMNS: usize>;

/// Draws an empty line of given length.
pub struct EmptyLine(pub usize);

/// An empty rectangular region of the screen.
pub struct EmptyRect(pub Rect);

/// Compile-time empty rectangular region of the screen.
pub struct ConstEmptyRect<const WIDTH: usize, const HEIGHT: u16>;

type EmptyTileLine = iter::Take<iter::Repeat<Tile>>;
fn empty_line(n: usize) -> EmptyTileLine {
    iter::repeat(Tile::EMPTY).take(n)
}

impl<const C: usize> Drawable for ConstEmptyLine<C> {
    type Iter = EmptyTileLine;

    fn for_each_line<F: FnMut(Pos, Self::Iter)>(&self, mut f: F) {
        f(Pos::DEFAULT, empty_line(C));
    }
}
impl Drawable for EmptyLine {
    type Iter = EmptyTileLine;

    fn for_each_line<F: FnMut(Pos, Self::Iter)>(&self, mut f: F) {
        f(Pos::DEFAULT, empty_line(self.0));
    }
}
impl<const W: usize, const H: u16> Drawable for ConstEmptyRect<W, H> {
    type Iter = EmptyTileLine;

    fn for_each_line<F: FnMut(Pos, Self::Iter)>(&self, mut f: F) {
        (0..H).for_each(|y| f(Pos::y(y), empty_line(W)));
    }
}
impl Drawable for EmptyRect {
    type Iter = EmptyTileLine;

    fn for_each_line<F: FnMut(Pos, Self::Iter)>(&self, mut f: F) {
        (0..self.0.height).for_each(|y| f(Pos::y(y), empty_line(self.0.width as usize)));
    }
}
