pub(crate) mod layout;

use hal::video::{
    tile::{map::Pos, Drawable},
    Tile,
};

/// Draws an empty line of length COLUMNS.
pub(crate) struct EmptyLine<const COLUMNS: usize>;

impl<const C: usize> Drawable for EmptyLine<C> {
    fn for_each_tile<F: FnMut(Tile, Pos)>(&self, pos: Pos, screen_width: usize, mut f: F) {
        (pos.x..pos.x + C).for_each(|x| f(Tile::EMPTY, Pos { x, ..pos }));
    }
}
