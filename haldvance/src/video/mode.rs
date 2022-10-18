//! Deal with GBA video modes, see [`Mode`].
use crate::video::tile::layer::{self, affine, text};

#[cfg(doc)]
use crate::video::{
    colmod,
    tile::{self, map, sbb},
    ColorMode,
};

/// Enumerates video [`Mode`]s.
#[repr(u16)]
pub enum Type {
    Text = 0,
    Mixed = 1,
    Affine = 2,
}

/// Video modes for use with [`super::Control`].
///
/// | Mode  | Layers (BG*) | Res | Tiles/double buffering | Colors | Features | Status|
/// |:-----:|:------------:|:---:|:----------------------:|:------:|:--------:|:--------------------:|
/// | [`Text`]        | 0,1,2,3 | 256² to 512²| [`map::TextSize`]  | 4/8bpp | Scroll, Flip | Done
/// | [`Mixed`]       | 0,1,2 | BG0/1 ↑, BG2 ↓| ← ibid           | ← ibid | ← ibid       | TODO
/// | [`Affine`]      | 2,3 | 128² to 1024² | [`map::AffineSize`]| 4/8bpp | Scroll, Affine | TODO
/// | `ColorBitmap`   | 2   | 240×160       | no               | RGB555   | Affine | TODO
/// | `PaletteBitmap` | 2   | 240×160       | double buff      | 4bpp     | Affine | TODO
/// | `LowBitmap`     | 2   | 160×128       | double buff      | RGB555   | Affine | TODO
///
/// See links to `Mode` implementors for fully detailed documentation.
///
/// `ColorBitmap` allows just to set individual pixel color in memory, there
/// is no double-buffering meaning that you will have a hard time avoiding screen
/// tearing. But you could in theory display anything in this mode, theoretically
/// allows you to display at the same time 32768 colors on screen.
///
/// `PaletteBitmap` is like `ColorBitmap`, but has two buffers, which means
/// you can work on one while the other is shown, then flipping between the two
/// buffers at your leisure. allowing you to avoid screen tearing.
/// However, in this mode, each pixel color is palette-indexed, allowing at most
/// 256 colors shown at the same time.
///
/// `LowBitmap` is like `ColorBitmap`, but with limited resolution (160×128 pixels
/// rather than the GBA-native 240×160), this allows double-buffering with full color
/// range available.
///
/// [`map::TextSize`]: crate::video::tile::map::TextSize
/// [`map::AffineSize`]: crate::video::tile::map::AffineSize
pub trait Mode: sealed::Mode {}

/// A background mode. Used to control backgrounds.
pub trait Background: sealed::Background {}

/// Subset of [`Mode`]s that support tile-based access.
pub trait Tile: Mode {}

/// Text mode, tile+map based background mode supporting 4 distinct background layers,
/// both [`colmod::Bit4`] and [`colmod::Bit8`] tile definition
/// and sprite flipping.
///
/// `Text` mode supports up to 1024 unique tiles per layer, also flipping individually
/// each tile, see [`tile::Tile`] methods for details.
///
/// `Text` mode doesn't support rotation and scaling, this is strictly tile-based.
/// See [`Mixed`] for an alternative with rotation and scaling.
pub enum Text {}
impl Mode for Text {}
impl Tile for Text {}
impl sealed::Mode for Text {
    const TYPE: Type = Type::Text;
}
impl Background for Text {}
impl sealed::Background for Text {
    type Slot = text::Slot;
}

/// Mixed mode, tile+map based background mode controlled like [`Text`]
/// for background layers 0 and 1,
/// and [`Affine`] for layer 2.
pub enum Mixed {}
impl Mode for Mixed {}
impl Tile for Mixed {}
impl sealed::Mode for Mixed {
    const TYPE: Type = Type::Mixed;
}

// TODO: implement scaling and rotation
/// Affine mode, tile+map based mode, only supports 2 layers (2 and 3).
///
/// Also often refered as "Affine" mode, or "Rotation/Scaling Modes".
///
/// It allows arbitrary affine transformations on the layer. Such as the ones
/// that would allow _Mario Kart Advance_ courses.
///
/// All layers in this mode support scaling and rotation,
/// but this is at the cost of some flexibility, notably:
///
/// - Only 8 bit color sprites are supported
/// - Only 2 layers are available
/// - Only 256 unique tile per layer (vs 1024 in [`Text`] mode)
/// - Can't individually flip tiles vertically/horizontally
///
/// Affine mode supports larger tile maps than [`Text`] mode,
/// (see [`map::TextSize`] and [`map::AffineSize`] for a discussion).
/// Furthermore, the memory layout of a tile map in `Affine` mode is much
/// simpler. They are byte per tile, and the map layout is one full row after
/// another (instead of the map being split in N regions).
///
/// Beware that the vram memory bus is 2 bytes, meaning that you can't set
/// tiles in `Affine` mode independently, see [`sbb::AffineHandle`] for the
/// performance implications.
pub enum Affine {}
impl Mode for Affine {}
impl Tile for Affine {}
impl sealed::Mode for Affine {
    const TYPE: Type = Type::Affine;
}
impl Background for Affine {}
impl sealed::Background for Affine {
    type Slot = affine::Slot;
}

/// traits to "seal" public traits in this module, to prevent
/// downstream implementation and exposing lower level implementation
/// details such as how memory is access in various video modes.
///
/// It also defines hardware representation of specific types.
pub(super) mod sealed {
    /// Seal the [`super::Mode`] trait.
    pub trait Mode {
        /// The `Type` representation of the display mode, one of 0,1,2,4,5
        const TYPE: super::Type;
    }
    pub trait Background {
        type Slot: super::layer::Slot;
    }
}
