use core::marker::PhantomData;

use crate::video::colmod::ColorMode;

#[doc(hidden)]
pub use include_const_aligned as align;

/// A set of tiles for text mode.
///
/// This is the raw data, not the tiles as represented by `Image`.
///
/// To create a `Tileset` use the [`crate::tileset!`] macro.
pub struct Tileset<M: ColorMode> {
    data: &'static [u16],
    _m: PhantomData<fn() -> M>,
}
impl<M: ColorMode> Tileset<M> {
    /// INTERNAL USE ONLY.
    ///
    /// This should only be called inside of the [`crate::tileset!`] macro.
    #[doc(hidden)]
    pub const fn new(data: &'static [u16]) -> Self {
        Self {
            data,
            _m: PhantomData,
        }
    }
    pub(crate) const fn get(&self) -> &'static [u16] {
        self.data
    }
}

/// Define a [`Tileset`].
///
/// Directly pass the file name, prefixes the path to the resources
/// directory.
#[macro_export]
macro_rules! tileset {
    ($file:literal) => {{
        // SAFETY: `u16` allows arbitrary bit patterns.
        let bytes = unsafe {
            $crate::video::tile::set::align::include_const_transmutted!(
                u16,
                concat!("../resources/", $file),
            )
        };
        $crate::video::Tileset::new(bytes)
    }};
}
