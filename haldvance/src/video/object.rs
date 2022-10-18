//! Deal with GBA objects.
//!
//! Objects are entities moving independently from the tileset background grid.
//! Furthermore, they may be accessed in the bitmap [`video::Mode`]s.
//!
//! There is a total of `128` [`Slot`]s for objects, and they may be controlled
//! through their [`Handle`], accessed through the [`video::Control::object`]
//! method.
use core::mem;

use const_default::ConstDefault;
use gba::mmio_types::{ObjAttr0, ObjAttr1, ObjAttr2};
use volmatrix::rw::VolAddress;

use crate::bitset::Bitset128;
use crate::sane_assert;
use crate::video::{self, palette, Priority};

/// A tile ID for object definitions.
///
/// Is in [0..1024], but may be limited following those conditions:
///
/// - If the specific object's [`Handle::set_palette_mode`] is **not**
///   [`palette::Type::Bank`], then odd-numbered tile numbers are invalid.
/// - If in a bitmap [`video::Mode`], then only tiles in [512..1024] are valid.
/// - If both conditions apply, then only even-numbered tiles in [512..1024]
///   are valid.
#[derive(Clone, Copy)]
pub struct Tile(u16);
impl Tile {
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
    pub(crate) const fn new(inner: u16) -> Self {
        Self(inner)
    }
}

/// The layout in memory of tiles used by objects.
/// Set this using [`video::Control::set_object_tile_mapping`].
#[derive(Clone, Copy)]
pub enum TileMapping {
    /// Multi-tiles objects use tiles that are one after the other in memory.
    ///
    /// When managing dynamically object sprites, it's much easier to deal
    /// with a 1D memory layout.
    OneDim,
    /// The object tile layout reflect the tileset layout.
    TwoDim,
}
impl TileMapping {
    pub(crate) const fn is_1d(self) -> bool {
        matches!(self, Self::OneDim)
    }
}

#[derive(Clone, Copy)]
#[repr(u16)]
enum ShapeDir {
    Square,
    Horizontal,
    Vertical,
}
#[derive(Clone, Copy)]
#[repr(u16)]
enum ShapeSize {
    Simple,
    Double,
    Quad,
    Octo,
}

macro_rules! impl_shape_consts {
    (
        dir: [$($_:tt)*],
        $($size:ident : [ $squ_dir:ident, $hor_dir:ident, $ver_dir:ident ],)*
     ) => {
        $(
            impl_shape_consts!(@singular Square, $size, $squ_dir);
            impl_shape_consts!(@singular Horizontal, $size, $hor_dir);
            impl_shape_consts!(@singular Vertical, $size, $ver_dir);
        )*
    };
    (@singular $direction:ident, $size:ident, $const_name:ident) => {
        /// An object of `
        #[doc = stringify!($const_name)]
        /// ` tiles.
        pub const $const_name: Self = Self {
            direction: ShapeDir::$direction,
            size: ShapeSize::$size,
        };
    };
}
/// The shape of an object.
#[derive(Clone, Copy)]
pub struct Shape {
    direction: ShapeDir,
    size: ShapeSize,
}
// allow: for the consts
#[allow(non_upper_case_globals)]
impl Shape {
    fn set_attributes(self, attributes: &mut Attributes) {
        attributes.attr0.set_obj_mode(self.direction as u16);
        attributes.attr1.set_obj_size(self.size as u16);
    }
    impl_shape_consts! {
        dir:    [Square, Horizontal, Vertical],
        Simple: [  _1x1,       _2x1,     _1x2],
        Double: [  _2x2,       _4x1,     _1x4],
        Quad:   [  _4x4,       _4x2,     _2x4],
        Octo:   [  _8x8,       _8x4,     _4x8],
    }
}

/// The object mode of an object.
#[derive(Clone, Copy)]
#[repr(u16)]
pub enum Mode {
    Normal,
    AlphaBlend,
    Window,
}

#[derive(Clone, Copy)]
#[repr(C)]
struct Attributes {
    attr0: ObjAttr0,
    attr1: ObjAttr1,
    attr2: ObjAttr2,
}

const OBJ_COUNT: usize = 128;
const OBJ_ADDR_USIZE: usize = 0x0700_0000;

/// An object slot.
///
/// You must use [`Allocator::reserve`] to get a `Slot`, to pass it to
/// [`video::Control::object`] to get a [`Handle`] to be able to draw objects
/// on screen. (See [`Handle`] for details)
pub struct Slot(u32);
impl Slot {
    // allow: We assume here we will compile for the GBA only, and it's really
    // burdensome to cast it in a "safe" way. (beside OBJ_COUNT = 128)
    /// How many Object slot there is.
    #[allow(clippy::cast_possible_truncation)]
    pub const MAX_BLOCKS: u32 = OBJ_COUNT as u32;

    /// # Safety
    /// `inner` must be lower than [`Self::MAX_BLOCKS`]
    #[must_use]
    pub(crate) const unsafe fn new_unchecked(inner: u32) -> Self {
        Self(inner)
    }

    const fn register(&self) -> VolAddress<Attributes> {
        // SAFETY: `self.0` is by definition lower than Self::MAX_BLOCKS,
        // which is the size of OBJ_ARRAY, meaning that `.get` returns always a `Some`.
        let offset = mem::size_of::<[u16; 4]>() * self.0 as usize;
        unsafe { VolAddress::new(OBJ_ADDR_USIZE + offset) }
    }
}

// TODO: reduce memory operations. (probably impossible to outperform
// memory load/store, unless I manage a compression scheme)
/// Game object video operations.
///
/// An "object" is a sprite on screen that can move independently from the
/// background. Such as Yoshi in Yoshi's Island. There can be up to 128
/// objects on screen at the same time.
///
/// To get an `object::Handle`, use [`video::Control::object`].
/// Note that the changes are only effective when the handle is dropped,
/// to avoid extraneous memory reads/writes.
pub struct Handle<'a> {
    value: Attributes,
    register: VolAddress<Attributes>,
    _ctrl: &'a mut (),
}
impl<'a> Handle<'a> {
    pub(super) fn new<N: video::Mode>(ctrl: &'a mut video::Control<N>, bg: &Slot) -> Self {
        let register = bg.register();
        Self {
            _ctrl: ctrl.erased(),
            value: register.read(),
            register,
        }
    }
    /// Set `x` coordinate of object.
    pub fn set_x(&mut self, x: u16) {
        self.value.attr1.set_x_pos(x);
    }
    /// Set `y` coordinate of object.
    pub fn set_y(&mut self, y: u16) {
        self.value.attr0.set_y_pos(y);
    }
    /// Set the object size.
    pub fn set_shape(&mut self, shape: Shape) {
        shape.set_attributes(&mut self.value);
    }
    pub fn set_priority(&mut self, priority: Priority) {
        self.value.attr2.set_priority(priority as u16);
    }
    pub fn set_mode(&mut self, mode: Mode) {
        self.value.attr0.set_obj_mode(mode as u16);
    }
    pub fn set_mosaic(&mut self, is_mosaic: bool) {
        self.value.attr0.set_mosaic(is_mosaic);
    }
    /// Set tile used by object.
    ///
    /// # Panics
    ///
    /// (`"sane_assert"` only)
    /// If `self.palette_mode() == palette::Type::Full && tile.get() % 2 == 1`
    ///
    /// Without `sane_assert`, odd tiles won't have effect with a full palette mode.
    pub fn set_tile(&mut self, tile: Tile) {
        sane_assert!(!self.value.attr0.use_palbank() && tile.0 % 2 == 0);
        self.value.attr2.set_tile_index(tile.0);
    }
    /// Set palette mode used by object.
    ///
    /// Note that if not set to [`palette::Type::Bank`],
    /// [`Self::set_tile`] will only accept even numbered tiles.
    pub fn set_palette_mode(&mut self, kind: palette::Type) {
        let use_palbank = matches!(kind, palette::Type::Bank);
        self.value.attr0.set_use_palbank(use_palbank);
    }
    /// Execute changes specified in this handle.
    pub fn commit(&mut self) {
        self.register.write(self.value);
    }
}
impl<'a> Drop for Handle<'a> {
    /// Commit all changes to video memory.
    fn drop(&mut self) {
        self.commit();
    }
}

// TODO: drop impl on Slot that updates this probably.
/// A generic allocator of exactly 128 items.
///
/// This is a 0-d allocator, ie: each allocated item are atoms of identical size.
#[derive(ConstDefault)]
pub struct Allocator(Bitset128);
impl Allocator {
    // allow: the `assert!(free<128)` should ALWAYS be true, due to a check in
    // `self.0.first_free`.
    /// Reserve an object slot.
    /// Returns `None` if no more slots are available.
    ///
    /// Make sure to call [`Allocator::free`] before dropping a [`Slot`],
    /// otherwise, the object slot will forever be leaked.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn reserve(&mut self) -> Option<Slot> {
        let free = self.0.first_free()?;
        self.0.reserve(free);
        assert!(free < 128);
        // SAFETY: `free` is always in 0..128.
        Some(unsafe { Slot::new_unchecked(free) })
    }
    // allow: `Slot` is meant to not be Copy or Clone, the goal of this method
    // is to provide an API where you can't have multiple handles to the same slot.
    /// Free an object slot, consuming it.
    #[allow(clippy::needless_pass_by_value)]
    pub fn free(&mut self, slot: Slot) {
        self.0.free(slot.0);
    }
}
