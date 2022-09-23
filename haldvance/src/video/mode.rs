/// Video modes for use with [`super::VideoControl`].
///
/// | Mode  | Layers (BG*) | Res | Tiles/double buffering | Colors | Features | Status|
/// |:-----:|:------------:|:---:|:----------------------:|:------:|:--------:|:--------------------:|
/// | [`Text`]        | 0-3 | 256^2 to 512^2  | [`map::TextSize`]  | 4/8bpp | Scroll, Flip | Done
/// | [`Mixed`]       | 0-2 | BG0/1 ↑, BG2 ↓  | ← ibid           | ← ibid | ← ibid       | TODO
/// | [`Affine`]      | 2,3 | 128^2 to 1024^2 | [`map::AffineSize`]| 4/8bpp | Scroll, Affine | TODO
/// | `ColorBitmap`   | 2   | 240x160         | no               | RGB555   | Affine | TODO
/// | `PaletteBitmap` | 2   | 240x160         | double buff      | 4bpp     | Affine | TODO
/// | `LowBitmap`     | 2   | 160x128         | double buff      | RGB555   | Affine | TODO
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

/// Subset of [`Mode`]s that support tile-based access.
pub trait TileMode: Mode {}

/// Text mode, tile+map based background mode supporting 4 distinct background layers,
/// both [`colmod::Bit4`] and [`colmod::Bit8`] tile definition
/// and sprite flipping.
///
/// `Text` mode doesn't support rotation and scaling, this is strictly tile-based.
/// See [`Mixed`] for an alternative with rotation and scaling.
pub enum Text {}
impl Mode for Text {}
impl TileMode for Text {}
impl sealed::Mode for Text {
    const RAW_REPR: u16 = 0;
}

/// Mixed mode, tile+map based background mode controlled like [`Text`]
/// for background layers 0 and 1,
/// and [`Affine`] for layer 2.
pub enum Mixed {}
impl Mode for Mixed {}
impl TileMode for Mixed {}
impl sealed::Mode for Mixed {
    const RAW_REPR: u16 = 1;
}

// TODO: implement scaling and rotation
/// Affine mode, tile+map based mode, only supports 2 layers (2 and 3).
///
/// Also often refered as "Affine" mode, or "Rotation/Scaling Modes".
///
/// All layers in this mode support scaling and rotation,
/// but this is at the cost of some flexibility, notably:
/// - Only 8 bit color sprites are supported
/// - Only 2 layers are available
///
/// Affine mode supports larger tile maps than [`Text`] mode,
/// this is how a game like _Mario Kart Advance_ would be implemented.
pub enum Affine {}
impl Mode for Affine {}
impl TileMode for Affine {}
impl sealed::Mode for Affine {
    const RAW_REPR: u16 = 2;
}

/// traits to "seal" public traits in this module, to prevent
/// downstream implementation and exposing lower level implementation
/// details such as how memory is access in various video modes.
///
/// It also defines hardware representation of specific types.
pub(super) mod sealed {
    /// Seal the [`super::Mode`] trait.
    pub trait Mode {
        /// The `u16` representation of the display mode, one of 0,1,2,4,5
        const RAW_REPR: u16;
    }
}
