//! Embedded game asset definitions.
#![no_std]
#![warn(clippy::pedantic, clippy::nursery)]
#![feature(const_mut_refs)]

use core::{iter, ops::Range, slice};

use haldvance::video::{palette, tile, Pos, Tile};

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

// TODO: affine alternative
pub struct DynamicImage<const COUNT: usize> {
    pub tiles: [u16; COUNT],
    pub width: u16,
}
impl<const COUNT: usize> DynamicImage<COUNT> {
    #[must_use]
    pub fn new(offset: u16, tileset_width: u16, width: u16) -> Self {
        let mut tiles = [0; COUNT];
        Image::set_tiles(offset, width, tileset_width, &mut tiles);
        Self { tiles, width }
    }
}
impl<'a, const COUNT: usize> tile::Drawable for &'a DynamicImage<COUNT> {
    type Iter = iter::Map<slice::Iter<'a, u16>, fn(&u16) -> Tile>;

    fn for_each_line<F: FnMut(Pos, Self::Iter)>(&self, mut f: F) {
        let DynamicImage { tiles, width } = self;
        let to_tile = |tile: &u16| Tile::new(*tile);
        tiles
            .chunks_exact(*width as usize)
            .zip(0_u16..)
            .for_each(|(line, y)| f(Pos::y(y), line.iter().map(to_tile)));
    }
}

// TODO: type-safe `Tileset` to make it impossible to missuse
// with regard to Color4bit and Color8bit.
// # Alternative implementations.
//
// I need to benchmark this, because Image might be perf-critical, but there is
// a few alternative on how to define an image, and all have different performance
// and ergonomic implications.
//
// ## Const everything
//
// ```
// pub struct Image<
//     const tileset_width: u16,
//     const offset: u16,
//     const width: u16,
//     const height: u16,
// >;
// ```
//
// Since all images are known at compile, we _could_ just make all fields into
// const type parameters. This requires implementing all APIs in term of and
// might result in code bloat, as monomorphization creates an instance of every
// methods on `Image` per instance. But it might not, as const
// propagation might be smart enough to replace basically all logic with simple
// `mov r10 #5` etc.
//
// ## A simple struct
//
// ```
// pub struct Image {
//     pub tileset_width: u16,
//     pub offset: u16,
//     pub width: u16,
//     pub height: u16,
// }
// ```
//
// Unlike the `const` solution, we do not have a monomorphization per instance,
// but this becomes relatively math heavy whenever we want to load an image,
// There is the multiplication with the `Pos`, `tileset_width`, out-of-bound
// checks etc. Obviously, this needs more fine-grained assembly inspection,
// but I'm worried we are hitting something heavy.
//
// ## Precomputed tiles
//
// ```
// pub struct Image {
//     pub tiles: &'static [u8],
//     pub width: u16,
// }
// ```
//
// But we can get rid of `offset`, `tileset_width`, `height` and move the
// computation of individual tile index to compile-time, if instead
// of storing them in. This `struct` is of size 10, previous was 8 (reference
// to slice is 2×usize)
//
/// An image in a tileset.
///
/// It can be drawn and stuff, while [`Tileset`] is the raw data to load in VRAM.
///
/// [`Tileset`]: haldvance::video::Tileset
pub struct Image {
    // TODO: consider using u8 to avoid code bloat here + Affine mode
    pub tiles: &'static [u16],
    pub width: u16,
}
impl tile::Drawable for Image {
    type Iter = iter::Map<slice::Iter<'static, u16>, fn(&u16) -> Tile>;

    fn for_each_line<F: FnMut(Pos, Self::Iter)>(&self, mut f: F) {
        let Self { tiles, width } = *self;
        let to_tile = |tile: &u16| Tile::new(*tile);
        tiles
            .chunks_exact(width as usize)
            .zip(0_u16..)
            .for_each(|(line, y)| f(Pos::y(y), line.iter().map(to_tile)));
    }
}
impl Image {
    /// Height of the image.
    #[must_use]
    pub const fn height(&self) -> u16 {
        self.tiles.len() as u16 / self.width
    }
    /// Width of the image.
    #[must_use]
    pub const fn width(&self) -> u16 {
        self.width
    }
    /// Set values of a slice to a tilemap image, for use in the [`image!`]
    /// macro in combination with the hidden `Image` constructor.
    pub const fn set_tiles(offset: u16, image_width: u16, tileset_width: u16, tiles: &mut [u16]) {
        let mut x = 0;
        let mut y = 0;
        let mut i = 0;
        loop {
            let oob_slice = i >= tiles.len();
            let oob_image = y >= image_width;
            if oob_image || oob_slice {
                return;
            }
            tiles[i] = offset + y * tileset_width + x;
            i += 1;
            x += 1;
            if x >= image_width {
                y += 1;
                x = 0;
            }
        }
    }

    #[doc(hidden)]
    #[must_use]
    pub const fn new(tiles: &'static [u16], width: u16) -> Self {
        Self { tiles, width }
    }
}
/// Define an [`Image`].
///
/// # Syntax
///
/// ```text
/// image!($offset, $image_width, $image_height, $tileset_width $(,)?)
/// ```
///
/// An [`Image`] is not the raw bytes of sprite, it is the offset
/// and position in the tile buffer of a specific image.
#[macro_export]
macro_rules! image {
    ($offset:expr, $image_width:expr, $image_height:expr, $tileset_width:expr $(,)?) => {{
        const slice: &'static [u16] = &{
            let mut tiles = [0; ($image_width as usize) * ($image_height as usize)];
            $crate::Image::set_tiles($offset, $image_width, $tileset_width, &mut tiles);
            tiles
        };
        $crate::Image::new(slice, $image_width)
    }};
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
        Self { range, frames_per_step }
    }
}
