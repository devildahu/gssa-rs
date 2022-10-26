//! Object sprite definition, similar to tileset, but with object specific
//! additional data.
use core::mem;

use super::Shape;
use crate::UniqueId;

#[cfg(doc)]
use crate::video::{self, object};

#[doc(hidden)]
pub use include_const_aligned as align;

/// A unique ID for sprites.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Id(UniqueId);

#[repr(transparent)]
#[derive(Clone, Copy)]
pub(crate) struct Entry(u16);

/// A loaded sprite identifier.
///
/// You first need to "load" your sprite to video memory using
/// [`video::Control::load_object_spirte`] to get a `Slot`, then use that
/// `Slot` with [`object::Handle::set_sprite`].
///
/// See [`object`] module doc for how to use objects.
#[derive(Clone, Copy)]
pub struct Slot {
    pub(super) offset: u16,
}
impl Slot {
    #[must_use]
    pub const fn get(self) -> u16 {
        self.offset
    }
    /// Define a sprite position.
    ///
    /// # Safety
    ///
    /// `inner < 1024`.
    ///
    /// Also note limitations:
    ///
    /// - If the specific object's [`Handle::set_palette_mode`] is **not**
    ///   [`palette::Type::Bank`], then odd-numbered tile numbers are invalid.
    /// - If in a bitmap [`video::Mode`], then only tiles in [512..1024] are valid.
    /// - If both conditions apply, then only even-numbered tiles in [512..1024]
    ///   are valid.
    #[must_use]
    pub(super) const unsafe fn new_unchecked(offset: u16) -> Self {
        Self { offset }
    }
}

/// A set of tiles for text mode.
///
/// This is the raw data, not the tiles as represented by `Image`.
///
/// To create a `Sprite` use the [`crate::tileset!`] macro.
pub struct Sprite {
    data: &'static [Entry],
    pub(super) shape: Shape,
    pub(crate) id: Id,
}

impl Sprite {
    /// INTERNAL USE ONLY.
    ///
    /// This should only be called inside of the [`crate::tileset!`] macro.
    #[doc(hidden)]
    #[must_use]
    pub const fn new(data: &'static [u16], id: UniqueId, shape: Shape) -> Self {
        Self {
            // SAFETY: technically, I should be using ptr::cast, but holy crap,
            // this would require so much boilerplate. As it is, it's perfectly
            // safe to transmute a &'static [u16] to a &'static [Entry],
            // because #[repr(transparent)] on Entry(u16).
            data: unsafe { mem::transmute(data) },
            id: Id(id),
            shape,
        }
    }
    pub(crate) const fn get(&self) -> &'static [Entry] {
        self.data
    }
    #[must_use]
    pub const fn shape(&self) -> &Shape {
        &self.shape
    }
}

/// Define a [`Sprite`].
///
/// Directly pass the file name, prefixes the path to the resources
/// directory.
#[macro_export]
macro_rules! sprite {
    ($file:literal, $shape:expr) => {{
        // SAFETY: `u16` allows arbitrary bit patterns.
        let bytes = unsafe {
            $crate::video::object::sprite::align::include_const_transmutted!(
                u16,
                concat!("../resources/", $file),
            )
        };
        $crate::video::object::Sprite::new(bytes, $crate::unique_id!(), $shape)
    }};
}
