use core::ops;

use const_default::ConstDefault;

#[cfg(doc)]
use crate::video::{
    mode::{Affine, Mixed, Mode, Text},
    tile::sbb,
};

// TODO: const-generic it by putting the background size as
// const LARGE_WIDTH: bool and const LARGE_HEIGHT: bool type parameters to mode::Text.
/// The tile map (or [SBB](sbb::Handle)) size for [`Text`] and [`Mixed`] [`Mode`]s.
///
/// GBATEK calls this "Screen Size."
///
/// # GBA implementation details
///
/// For [`Text`] [`Mode`] layers, larger (ie: non-`Base`) tile map sizes will
/// split the overall tilemap into multiple SBBs.
#[repr(u16)]
pub enum TextSize {
    /// 32×32 tiles, or 256×256 pixels
    Base = 0,
    /// 64×32 tiles, or 512×256 pixels. Takes 2 SBBs (`sbb` and `sbb + 1`).
    Long = 1,
    /// 32×64 tiles, or 256×512 pixels. Takes 2 SBBs (`sbb` and `sbb + 1`).
    Tall = 2,
    /// 64×64 tiles, or 512×512 pixels. Takes 4 SBBs (`sbb + 0 to 3`).
    Large = 3,
}
impl TextSize {
    /// The tile count of a single layer for this `TextSize`.
    pub const fn region(&self) -> Rect {
        match self {
            TextSize::Base => Rect {
                width: 32,
                height: 32,
            },
            TextSize::Long => Rect {
                width: 64,
                height: 32,
            },
            TextSize::Tall => Rect {
                width: 32,
                height: 64,
            },
            TextSize::Large => Rect {
                width: 64,
                height: 64,
            },
        }
    }
}
/// The tile map (or [SBB](sbb::Handle)) size for [`Mixed`] and [`Affine`] [`Mode`]s.
///
/// GBATEK calls this "Screen Size."
#[repr(u16)]
pub enum AffineSize {
    /// 16×16 tiles, or 128×128 pixels
    Base = 0,
    /// 32×32 tiles, or 256×256 pixels
    Double = 1,
    /// 64×64 tiles, or 512×512 pixels
    Quad = 2,
    /// 128×128 tiles, or 1024×1024 pixels
    Octo = 3,
}

/// A rectangular region of the screen.
#[derive(Clone, Copy)]
pub struct Rect {
    pub width: usize,
    pub height: usize,
}
impl Rect {
    /// Whether provided `pos` is inside `Rect`.
    pub const fn contains(&self, pos: Pos) -> bool {
        pos.x < self.width && pos.y < self.height
    }
}

/// A position in the map.
#[derive(Copy, Clone, ConstDefault)]
pub struct Pos {
    pub x: usize,
    pub y: usize,
}
impl Pos {
    pub const fn x(value: usize) -> Self {
        Pos { x: value, y: 0 }
    }
    pub const fn y(value: usize) -> Self {
        Pos { x: 0, y: value }
    }
}
impl ops::Add<Pos> for Pos {
    type Output = Pos;
    fn add(self, other: Pos) -> Pos {
        Pos {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
impl ops::Sub<Pos> for Pos {
    type Output = Pos;
    fn sub(self, other: Pos) -> Pos {
        Pos {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
