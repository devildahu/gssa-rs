//! Draw images, tiles and text in text-mode
use core::ops::Add;

use const_default::ConstDefault;

use super::Tile;
use crate::assets::Image;

/// Something that can be drawn on screen in text mode.
///
/// To draw something in [`crate::vidmod::Text`], call [`SbbHandle::set_tiles`]
/// with a `text::Draw`able. This trait only defines which tiles to place where on a map
/// for a given instance.
///
/// [`SbbHandle::set_tiles`]: crate::video_control::SbbHandle::set_tiles
pub(crate) trait Draw {
    /// Call `f` once per tile of self.
    fn for_each_tile<F: FnMut(Tile, Pos)>(&self, pos: Pos, width: usize, f: F);
}

/// A screen tile position in text mode.
#[derive(Copy, Clone, ConstDefault)]
pub(crate) struct Pos {
    pub(crate) x: usize,
    pub(crate) y: usize,
}
#[cfg(feature = "runtime_asserts")]
impl core::fmt::Display for Pos {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
impl Pos {
    pub(crate) const DEFAULT: Self = Pos { x: 0, y: 0 };
}
impl Add<Pos> for Pos {
    type Output = Pos;
    fn add(self, other: Pos) -> Pos {
        Pos {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Draw for Image {
    fn for_each_tile<F: FnMut(Tile, Pos)>(&self, pos: Pos, screen_width: usize, mut f: F) {
        let Self {
            height,
            offset,
            tileset_width,
            width,
        } = *self;

        for y in 0..height as u16 {
            for x in 0..width as u16 {
                let sprite_pos = offset + tileset_width * y + x;
                let tile = Tile::new(sprite_pos);
                let offset = Pos {
                    x: x as usize,
                    y: y as usize,
                };
                // TODO: do not go beyond screen_width
                f(tile, pos + offset)
            }
        }
    }
}
const ASCII_OFFSET: u8 = 0x20;
impl<'s> Draw for &'s str {
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
