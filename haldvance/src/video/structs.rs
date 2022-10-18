//! Generic structs used for video.

use core::ops;

use const_default::ConstDefault;

/// A position, depending on context, may be a tile location on a tile map or
/// a coordinate of an object.
#[derive(Copy, Clone, ConstDefault)]
pub struct Pos {
    pub x: u16,
    pub y: u16,
}
impl Pos {
    #[must_use]
    pub const fn x(value: u16) -> Self {
        Self { x: value, y: 0 }
    }
    #[must_use]
    pub const fn y(value: u16) -> Self {
        Self { x: 0, y: value }
    }
}
impl ops::Add<Self> for Pos {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self { x: self.x + other.x, y: self.y + other.y }
    }
}
impl ops::Sub<Self> for Pos {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self { x: self.x - other.x, y: self.y - other.y }
    }
}
