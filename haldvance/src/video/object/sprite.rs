//! Object sprite definition, similar to tileset, but with object specific
//! additional data.

use super::Shape;
use crate::UniqueId;

#[cfg(doc)]
use crate::video::{self, object};

#[doc(hidden)]
pub use include_const_aligned as align;
use volmatrix::VolMemcopy;

// There are 8×8 color per tile, each color is 8 bit aka 0.5 u16
// So total of 32 Entry per Tile
#[doc(hidden)]
pub const ENTRY_PER_TILE: usize = (8 * 8) / 2;

/// A unique ID for sprites.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Id(UniqueId);

#[repr(transparent)]
#[derive(Clone, Copy)]
struct Entry(u16);

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Tile([Entry; ENTRY_PER_TILE]);

/// A loaded sprite identifier.
///
/// You first need to "load" your sprite to video memory using
/// [`video::Control::load_sprite`] to get a `Slot`, then use that
/// `Slot` with [`object::Handle::set_sprite`].
///
/// See [`object`] module doc for how to use objects.
#[derive(Clone, Copy, Debug)]
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

/// A set of tiles that represents an object sprite.
///
/// This is the raw data.
///
/// To create a `Sprite` use the [`crate::sprite!`] macro.
pub struct Sprite {
    data: &'static [Tile],
    shape: Shape,
    id: Id,
}
impl Sprite {
    /// INTERNAL USE ONLY.
    ///
    /// This should only be called inside of the [`crate::sprite!`] macro.
    #[doc(hidden)]
    #[must_use]
    pub const fn new(data: &'static [Tile], id: UniqueId, shape: Shape) -> Self {
        Self { data, id: Id(id), shape }
    }
    #[must_use]
    pub const fn shape(&self) -> &Shape {
        &self.shape
    }
    pub(crate) fn load_at_slot(&self, slot: Slot) {
        super::OBJ_SPRITE.write_slice_at_offset(slot.offset as usize, self.data);
    }
    #[must_use]
    pub const fn tile_count(&self) -> u16 {
        self.shape.tile_count()
    }
    #[must_use]
    pub(crate) const fn id(&self) -> Id {
        self.id
    }
}

pub struct SheetSlot<const I: u16> {
    offset: u16,
}

impl<const I: u16> SheetSlot<I> {
    pub(crate) const fn from_slot(Slot { offset }: Slot) -> Self {
        Self { offset }
    }
    /// Get the sprite [`Slot`] a provided index.
    ///
    /// # Panics
    ///
    /// (const): when `index >= I`.
    #[must_use]
    pub const fn get(&self, index: u16) -> Slot {
        assert!(index < I);
        Slot { offset: self.offset + index }
    }
}

/// A collection of `I` 1×1 sprites.
pub struct Sheet<const I: u16> {
    data: &'static [Tile],
    id: Id,
}
impl<const I: u16> Sheet<I> {
    /// INTERNAL USE ONLY.
    ///
    /// This should only be called inside of the [`crate::sprite!`] macro.
    #[doc(hidden)]
    #[must_use]
    pub const fn new(data: &'static [Tile], id: UniqueId) -> Self {
        assert!(data.len() == I as usize);
        Self { data, id: Id(id) }
    }
    pub(crate) fn load_at_slot(&self, slot: Slot) {
        super::OBJ_SPRITE.write_slice_at_offset(usize::from(slot.offset), self.data);
    }

    #[must_use]
    pub const fn tile_count(&self) -> u16 {
        I
    }
    #[must_use]
    pub(crate) const fn id(&self) -> Id {
        self.id
    }
}

/// Define a [`Sprite`].
///
/// Directly pass the file name, prefixes the path to the resources
/// directory.
#[macro_export]
macro_rules! sprite {
    ($file:literal, $shape:expr) => {{
        // SAFETY: `Tile` allows arbitrary bit patterns.
        let bytes = unsafe {
            $crate::video::object::sprite::align::include_const_transmutted!(
                4,
                $crate::video::object::sprite::Tile,
                concat!("../resources/", $file),
            )
        };
        $crate::video::object::Sprite::new(bytes, $crate::unique_id!(), $shape)
    }};
}

/// Define a [`Sheet`].
///
/// Directly pass the file name, prefixes the path to the resources
/// directory.
// allow: we call this `sprite_sheet` because it is always exported at the root
// of the crate, so it's dissociated from the defining module.
#[allow(clippy::module_name_repetitions)]
#[macro_export]
macro_rules! sprite_sheet {
    ($file:literal) => {{
        // SAFETY: `Tile` allows arbitrary bit patterns.
        const bytes: &[$crate::video::object::sprite::Tile] = unsafe {
            $crate::video::object::sprite::align::include_const_transmutted!(
                4,
                $crate::video::object::sprite::Tile,
                concat!("../resources/", $file),
            )
        };
        $crate::video::object::sprite::Sheet::<{ bytes.len() as u16 }>::new(
            bytes,
            $crate::unique_id!(),
        )
    }};
}
