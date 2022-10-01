//! Embedded game asset definitions.
#![no_std]
#![warn(clippy::pedantic, clippy::nursery)]

use core::ops::Range;

use haldvance::video::{
    palette,
    tile::{self, map::Pos},
    Tile,
};

// For usage in the macros defined here.
#[doc(hidden)]
pub use haldvance::video::tile::Color;
#[doc(hidden)]
pub use include_const_aligned as include_macros;

#[macro_export]
macro_rules! palette {
    ($file:literal $(, cycle ($range:expr, $rate:expr) )* $(,)?) => {{
        // SAFETY: `Color` (from gba crate) here is repr(transparent) u16,
        // which allows arbitrary bit patterns.
        let colors = unsafe {
            $crate::include_macros::include_const_transmutted!(
                $crate::Color,
                concat!("../resources/", $file),
            )
        };
        let cycles = &[ $( Cycle::new($range, $rate), )* ];
        $crate::Palette::new(colors, cycles)
    }};
}

/// A palette with color cycling information.
pub struct Palette {
    data: palette::Dynamic,
    cycles: &'static [Cycle],
}

impl Palette {
    #[must_use]
    pub const fn new(colors: &'static [Color], cycles: &'static [Cycle]) -> Self {
        Self {
            data: palette::Dynamic::new(colors),
            cycles,
        }
    }
    #[must_use]
    pub const fn get(&self) -> &[Color] {
        self.data.get()
    }
}

// TODO: type-safe `Tileset` to make it impossible to missuse
// with regard to Color4bit and Color8bit.
// TODO: probably requires distinguishing "dynamic" images from
// fixed position images.
/// An image in a tileset.
///
/// It can be drawn and stuff, while [`Tileset`] is the raw data to load in VRAM.
///
/// [`Tileset`]: haldvance::video::Tileset
pub struct Image {
    /// The **tileset**'s width.
    pub tileset_width: u16,
    pub offset: u16,
    pub width: u16,
    pub height: u16,
}
impl tile::Drawable for Image {
    fn for_each_tile<F: FnMut(Tile, Pos)>(&self, mut f: F) {
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
                f(tile, Pos { x, y });
            }
        }
    }
}
/// Define an [`Image`].
///
/// An [`Image`] is not the raw bytes of sprite, it is the offset
/// and position in the tile buffer of a specific image.
#[macro_export]
macro_rules! image {
    ($file:literal) => {
        Image {
            data: include_bytes!(concat!("../resources/", $file)),
        }
    };
}

/// A palette cycle.
///
/// This control palette cycling, for nice graphical effects.
pub struct Cycle {
    pub range: Range<usize>,
    pub frames_per_step: usize,
}
impl Cycle {
    #[must_use]
    pub const fn new(range: Range<usize>, frames_per_step: usize) -> Self {
        Self {
            range,
            frames_per_step,
        }
    }
}
