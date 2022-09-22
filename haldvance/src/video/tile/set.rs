use core::marker::PhantomData;

use crate::video::colmod::ColorMode;

/// A set of tiles for text mode.
///
/// This is the raw data, not the tiles as represented by [`Image`].
///
/// To create a `Tileset` use the [`tileset!`] macro.
pub struct Tileset<M: ColorMode> {
    data: &'static [u8],
    _m: PhantomData<fn() -> M>,
}
impl<M: ColorMode> Tileset<M> {
    /// INTERNAL USE ONLY.
    ///
    /// This should only be called inside of the [`tileset!`] macro.
    #[doc(hidden)]
    pub const fn new(data: &'static [u8]) -> Self {
        Self {
            data,
            _m: PhantomData,
        }
    }
    pub(crate) const fn get(&self) -> &'static [u8] {
        self.data
    }
}

/// Define a [`Tileset`].
///
/// Directly pass the file name, prefixes the path to the resources
/// directory.
#[macro_export]
macro_rules! tileset {
    ($file:literal) => {
        $crate::video::Tileset::new(include_bytes!(concat!("../resources/", $file)))
    };
}
