use crate::video::Pos;

#[cfg(doc)]
use crate::video::{
    mode::{Affine, Mixed, Mode, Text},
    tile::sbb,
};

// TODO: const-generic it by putting the background size as
// const LARGE_WIDTH: bool and const LARGE_HEIGHT: bool type parameters to mode::Text.
/// The tile map (or [SBB](sbb::TextHandle)) size for [`Text`] and [`Mixed`] [`Mode`]s.
///
/// GBATEK calls this "Screen Size."
///
/// # GBA implementation details
///
/// For [`Text`] [`Mode`] layers, larger (ie: non-`Base`) tile map sizes will
/// split the overall tilemap into multiple SBBs.
#[derive(Clone, Copy)]
#[repr(u8)]
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
    #[must_use]
    pub const fn region(&self) -> Rect {
        match self {
            Self::Base => Rect { width: 32, height: 32 },
            Self::Long => Rect { width: 64, height: 32 },
            Self::Tall => Rect { width: 32, height: 64 },
            Self::Large => Rect { width: 64, height: 64 },
        }
    }
    #[must_use]
    pub const fn width(self) -> u16 {
        self.region().width
    }
    #[must_use]
    pub const fn height(self) -> u16 {
        self.region().height
    }
}
/// The tile map (or [SBB](sbb::TextHandle)) size for [`Mixed`] and [`Affine`] [`Mode`]s.
///
/// GBATEK calls this "Screen Size."
#[derive(Clone, Copy)]
#[repr(u8)]
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
impl AffineSize {
    /// The proportions of a single layer for this `TextSize`.
    #[must_use]
    pub const fn region(self) -> Rect {
        match self {
            Self::Base => Rect { width: 16, height: 16 },
            Self::Quad => Rect { width: 64, height: 64 },
            Self::Octo => Rect { width: 128, height: 128 },
            Self::Double => Rect { width: 32, height: 32 },
        }
    }
    /// The number of tiles in a row of a map of this size.
    #[must_use]
    pub const fn width(self) -> u16 {
        self.region().width
    }
    /// The number of tiles in a column of a map of this size.
    #[must_use]
    pub const fn height(self) -> u16 {
        self.region().height
    }
    /// The tile count of a map of this size.
    #[must_use]
    pub const fn surface_size(self) -> u16 {
        self.width() * self.height()
    }
}

/// A rectangular region of the screen.
#[derive(Clone, Copy)]
pub struct Rect {
    /// Number of rows.
    pub width: u16,
    /// Number of columns.
    pub height: u16,
}
impl Rect {
    /// Whether provided `pos` is inside `Rect`.
    #[must_use]
    pub const fn contains(&self, pos: Pos) -> bool {
        pos.x < self.width && pos.y < self.height
    }
}
