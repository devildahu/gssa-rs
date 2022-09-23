use gba::mmio_types::Color;

#[cfg(doc)]
use crate::video::{colmod, ColorMode, Tile, Tileset};

macro_rules! impl_palette {
    (
        $( [
            impl $([$( $generic:tt )*])? $name:ident $(<$( $generic_arg:ident ),*>)?
            $(, size: $size:tt )? $(,)?
        ] ,)* $(,)?
    ) => {
        $(
            impl $(<$($generic)*>)? $name $(<$($generic_arg),*>)? {
                /// INTERNAL USE ONLY.
                ///
                /// This should only be called inside of the [`palette!`] macro.
                #[doc(hidden)]
                pub const fn new(data: &'static [Color $(; $size)?]) -> Self {
                    Self { data }
                }

                pub const fn get(&self) -> &[Color] {
                    self.data
                }
            }
        )*
    }
}

/// A palette [`Bank`] handle to refer to individual palette banks in [`Tile`].
pub struct BankHandle {
    pub(super) id: u16,
}

// TODO: implement palette manager
/// A partial color palette, for use with a palette manager.
pub struct Dynamic {
    data: &'static [Color],
}
/// A full color palette for [`colmod::Bit8`] [`ColorMode`].
pub struct Full {
    data: &'static [Color; 256],
}
/// An individual palette for [`colmod::Bit4`] [`ColorMode`].
///
/// In this mode, the GBA can hold 16 different "palette banks."
/// A "palette bank" is a 16 colors palette.
///
/// Furthermore, each individual tiles of a [`Tileset`] may refer to
/// a single "palette bank," but each tile can be assigned
/// a different "palette bank" in the `Tilemap`.
pub struct Bank {
    data: &'static [Color; 16],
}
impl_palette! {
    [impl Dynamic],
    [impl Bank, size: 16],
    [impl Full, size: 256],
}

/// Define a [`palette`](self).
///
/// Directly pass the file name, prefixes the path to the resources
/// directory.
#[macro_export]
macro_rules! palette {
    ($file:literal) => {
        Palette::new(include_bytes!(concat!("../resources/", $file)))
    };
}
