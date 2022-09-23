use core::ops::Add;

use const_default::ConstDefault;

#[cfg(doc)]
use crate::video::mode::{Affine, Mixed, Mode, Text};

// TODO: enum this, or even const-generic it.
pub(crate) type ScreenSize = usize;

pub const HARDCODED_TILEMAP_WIDTH: u16 = 32;

/// The tile map (or [SBB](SbbHandle)) size for [`Text`] and [`Mixed`] [`Mode`]s.
///
/// GBATEK calls this "Screen Size."
#[repr(u16)]
pub enum TextSize {
    /// 32×32 tiles, or 256×256 pixels
    Base = 0,
    /// 64×32 tiles, or 512×256 pixels
    Long = 1,
    /// 32×64 tiles, or 256×512 pixels
    Tall = 2,
    /// 64×64 tiles, or 512×512 pixels
    Large = 3,
}
/// The tile map (or [SBB](SbbHandle)) size for [`Mixed`] and [`Affine`] [`Mode`]s.
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

/// A position in the map.
#[derive(Copy, Clone, ConstDefault)]
pub struct Pos {
    pub x: usize,
    pub y: usize,
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
