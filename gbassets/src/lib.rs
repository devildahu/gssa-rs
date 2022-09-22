//! Embedded game asset definitions.
#![no_std]
#![warn(clippy::pedantic, clippy::nursery)]

use core::ops::Range;

use haldvance::video::{
    palette,
    tile::{self, map::Pos},
    Tile,
};

#[doc(hidden)]
pub use haldvance::video::tile::Color;

#[macro_export]
macro_rules! palette {
    ($file:literal $(, cycle ($range:expr, $rate:expr) )* $(,)?) => {{
// https://users.rust-lang.org/t/can-i-conveniently-compile-bytes-into-a-rust-program-with-a-specific-alignment/24049
// This struct is generic in Bytes to admit unsizing coercions.
#[repr(C)] // guarantee 'bytes' comes after '_align'
struct AlignedTo<Align, Bytes: ?Sized> {
    _align: [Align; 0],
    bytes: Bytes,
}

// dummy static used to create aligned data
const ALIGNED: &'static AlignedTo<$crate::Color, [u8]> = &AlignedTo {
    _align: [],
    bytes: *include_bytes!(concat!("../resources/", $file)),
};

const ALIGNED_BYTES: &'static [u8] = &ALIGNED.bytes;


        let u8s = &ALIGNED.bytes;
        let byte_len = u8s.len();
        // SAFETY: byte_len >> 1 is half the size of u8s, Color is repr(u16)
        let colors: &[$crate::Color] = unsafe {
            core::slice::from_raw_parts(u8s.as_ptr().cast(), byte_len >> 1)
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
    pub const fn new(colors: &'static [Color], cycles: &'static [Cycle]) -> Self {
        Self {
            data: palette::Dynamic::new(colors),
            cycles,
        }
    }
    pub const fn get(&self) -> &[Color] {
        self.data.get()
    }
}

// TODO: type-safe [`Tileset`] to make it impossible to missuse
// with regard to Color4bit and Color8bit.
// TODO: probably requires distinguishing "dynamic" images from
// fixed position images.
/// An image in a tileset.
///
/// It can be drawn and stuff, while [`Tileset`] is the raw data to load in VRAM.
pub struct Image {
    /// The **tileset**'s width.
    pub tileset_width: u16,
    pub offset: u16,
    pub width: usize,
    pub height: usize,
}
impl tile::Drawable for Image {
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
    pub const fn new(range: Range<usize>, frames_per_step: usize) -> Self {
        Self {
            range,
            frames_per_step,
        }
    }
}
