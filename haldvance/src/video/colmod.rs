#[cfg(doc)]
use crate::video::{
    mode::{Mixed, Text},
    tile::sbb,
    Tile,
};

/// The color mode of tile bitmap information.
///
/// Also commonly refered as `8bpp` or `bpp8`.
///
/// GBATEK calls the memory block that holds the tile bitmap information
/// `CHAR_BASE_BLOCK`.
///
/// The GBA allows specifying tile texture data in two ways
/// when in [`Text`] or [`Mixed`] mode:
///
/// * **[`Bit8`]:** Each color index within a background or object is 8 bits. The 8
///   bits index into the full block of colors as if it was a single set of 256
///   elements. Index 0 is counted as the "transparency" index.
///   * `base_addr + size_of::<Color>() * index`
/// * **[`Bit4`]:** Each color index within a background or object is 4 bits. The 4
///   bits within the image data is an entry into a "palette bank" of 16
///   entries. The palette bank used for the BG or OBJ overall is determined by
///   their control bits. Index 0 within a palbank is still considered the
///   "transparency" index.
///   * `base_addr + size_of::<Color>() * (pal_bank << 4 | entry)`
///
/// A series of 32×32 colors represent a [`Tile`], individual tiles are
/// addressed by index in the [SBB](sbb::Handle).
pub trait ColorMode: sealed::ColorMode {}

/// 8 bit color tiles video mode.
pub enum Bit8 {}
impl ColorMode for Bit8 {}
impl sealed::ColorMode for Bit8 {
    const RAW_REPR: bool = true;
}

/// 4 bit color tiles video mode.
///
/// Also commonly refered as `4bpp` or `bpp4`.
///
/// See [`ColorMode`] for details.
pub enum Bit4 {}
impl ColorMode for Bit4 {}
impl sealed::ColorMode for Bit4 {
    const RAW_REPR: bool = false;
}

/// traits to "seal" public traits in this module, to prevent
/// downstream implementation and exposing lower level implementation
/// details such as how memory is access in various video modes.
///
/// It also defines hardware representation of specific types.
pub(super) mod sealed {
    /// Seal the [`super::ColorMode`] trait.
    pub trait ColorMode {
        /// The `bool` representation of the color mode. (`true` for [`super::Bit8`]
        /// and `false` for [`super::Bit4`])
        const RAW_REPR: bool;
    }
}
