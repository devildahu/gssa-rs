pub(crate) mod layout;

use hal::video::{
    tile::{map::Pos, Drawable},
    Tile,
};

/// Draws an empty line of length COLUMNS.
pub(crate) struct EmptyLine<const COLUMNS: usize>;

/// Draws an empty line of given length.
pub(crate) struct DynEmptyLine(pub(crate) usize);

/// A rectangular region of the screen.
pub(crate) struct Rect {
    width: usize,
    height: usize,
}

/// An empty rectangular region of the screen.
pub(crate) struct EmptyRect(pub(crate) Rect);

/// Compile-time empty rectangular region of the screen.
pub(crate) struct ConstEmptyRect<const WIDTH: usize, const HEIGHT: usize>;

impl<const C: usize> Drawable for EmptyLine<C> {
    fn for_each_tile<F: FnMut(Tile, Pos)>(&self, pos: Pos, screen_width: usize, mut f: F) {
        (pos.x..pos.x + C).for_each(|x| f(Tile::EMPTY, Pos { x, ..pos }));
    }
}
impl Drawable for DynEmptyLine {
    fn for_each_tile<F: FnMut(Tile, Pos)>(&self, pos: Pos, screen_width: usize, mut f: F) {
        (pos.x..pos.x + self.0).for_each(|x| f(Tile::EMPTY, Pos { x, ..pos }));
    }
}
impl<const W: usize, const H: usize> Drawable for ConstEmptyRect<W, H> {
    fn for_each_tile<F: FnMut(Tile, Pos)>(&self, pos: Pos, screen_width: usize, mut f: F) {
        todo!()
    }
}
impl Drawable for EmptyRect {
    fn for_each_tile<F: FnMut(Tile, Pos)>(&self, pos: Pos, screen_width: usize, mut f: F) {
        todo!()
    }
}
