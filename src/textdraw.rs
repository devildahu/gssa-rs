//! Draw images, tiles and text in text-mode

use crate::assets::Image;

/// Something that can be drawn on screen in text mode.
pub(crate) trait Draw {
    fn draw(&self, pos: Pos);
}

/// A screen tile position in text mode.
#[derive(Copy, Clone)]
pub(crate) struct Pos {
    pub(crate) x: usize,
    pub(crate) y: usize,
}
impl Pos {
    pub(crate) const DEFAULT: Self = Pos { x: 0, y: 0 };
}

impl Draw for Image {
    fn draw(&self, pos: Pos) {
        todo!()
    }
}
impl Draw for &'static str {
    fn draw(&self, pos: Pos) {
        todo!()
    }
}
