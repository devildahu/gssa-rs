//! Deal with moving sprites.
//!
//! Objects are entities moving independently from the tileset background grid.
//! Furthermore, they may be accessed in the bitmap [`video::Mode`]s.
//!
//! There is a total of `128` [`Slot`]s for objects, and they may be controlled
//! through their [`Handle`], accessed through the [`video::Control::object`]
//! method.
//!
//! Getting an object is a bit involved, so here is a step-by step API usage
//! guide:
//!
//! - Define a sprite with the [`crate::sprite!`] macro.
//! - Load your sprite to video memory using
//!   [`video::Control::load_spirte`] to get a [`Sprite`].
//! - Allocate an object [`Slot`] with [`ConsoleState::reserve_object`].
//! - Get a [`Handle`] for the [`Slot`] using [`video::Control::object`].
//! - Use methods on [`Handle`] to manipulate object on screen.
//! - For the life time of the object, keep the [`Slot`] in your game state.
//! - Once the object dies, you should free the slot using [`ConsoleState::free_object`].
//!
//! You should store the [`Slot`] in some runtime struct for the lifetime of the object,
//! **and make sure to free it** with [`ConsoleState::free_object`].
//!
//! [`Sprite`] can be handled differently from [`Slot`]. When you define a `Sprite`
//! using the [`crate::sprite!`] macro, a unique identifier is assigned to it, you
//! should be able to chose when to unload it to leave room for other [`Sprite`]
//! with [`video::Control::unload_sprite`].
//!
//! Note that the max number of tiles (multiply the two values of the [`Shape`]
//! constants) loadable at the same time is `1024`, but may be limited in
//! those conditions:
//!
//! - If the specific object's [`Handle::set_palette_mode`] is **not**
//!   [`palette::Type::Bank`], then odd-numbered tile numbers are invalid.
//! - If in a bitmap [`video::Mode`], then only tiles in [512..1024] are valid.
//! - If both conditions apply, then only even-numbered tiles in [512..1024]
//!   are valid.

pub mod sprite;

use core::mem;

use const_default::ConstDefault;
use gba::mmio_types::{ObjAttr0, ObjAttr1, ObjAttr2};
use volmatrix::rw::{VolAddress, VolBlock};

use crate::bitset::Bitset128;
use crate::block::Blocks;
use crate::sane_assert;
use crate::video::{self, palette, Pos, Priority};

#[cfg(doc)]
use crate::exec::ConsoleState;

pub use sprite::Sprite;

const OBJ_COUNT: usize = 128;
const OBJ_ADDR_USIZE: usize = 0x0700_0000;
const OBJ_SPRITE_ADDR_USIZE: usize = 0x0601_0000;
const SPRITE_FULL_SIZE: u16 = 1024;
const SPRITE_MAX_BLOCKS: usize = SPRITE_FULL_SIZE as usize / 2;

// TODO: bump by 512 in bitmap modes

// SAFETY: this OBJ_SPRITE is indeed inside VRAM.
pub(super) const OBJ_SPRITE: VolBlock<sprite::Entry, { 0x8000 / mem::size_of::<sprite::Entry>() }> =
    unsafe { VolBlock::new(OBJ_SPRITE_ADDR_USIZE) };

/// The layout in memory of tiles used by objects.
///
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

/// The shape of an object.
#[derive(Clone, Copy)]
pub enum Shape {
    _1x1,
    _2x2,
    _4x4,
    _8x8,
    _2x1,
    _4x1,
    _4x2,
    _8x4,
    _1x2,
    _1x4,
    _2x4,
    _4x8,
}
impl Shape {
    const fn components(self) -> (ShapeDir, ShapeSize) {
        use ShapeDir::{Horizontal, Square, Vertical};
        use ShapeSize::{Double, Octo, Quad, Simple};
        match self {
            Self::_1x1 => (Square, Simple),
            Self::_2x2 => (Square, Double),
            Self::_4x4 => (Square, Quad),
            Self::_8x8 => (Square, Octo),
            Self::_2x1 => (Horizontal, Simple),
            Self::_4x1 => (Horizontal, Double),
            Self::_4x2 => (Horizontal, Quad),
            Self::_8x4 => (Horizontal, Octo),
            Self::_1x2 => (Vertical, Simple),
            Self::_1x4 => (Vertical, Double),
            Self::_2x4 => (Vertical, Quad),
            Self::_4x8 => (Vertical, Octo),
        }
    }
    const fn tile_count(self) -> u16 {
        #[allow(clippy::match_same_arms, clippy::identity_op)]
        match self {
            Self::_1x1 => 1 * 1,
            Self::_2x2 => 2 * 2,
            Self::_4x4 => 4 * 4,
            Self::_8x8 => 8 * 8,
            Self::_2x1 => 2 * 1,
            Self::_4x1 => 4 * 1,
            Self::_4x2 => 4 * 2,
            Self::_8x4 => 8 * 4,
            Self::_1x2 => 1 * 2,
            Self::_1x4 => 1 * 4,
            Self::_2x4 => 2 * 4,
            Self::_4x8 => 4 * 8,
        }
    }
    fn set_attributes(self, attributes: &mut Attributes) {
        let (direction, size) = self.components();
        attributes.attr0.set_obj_mode(direction as u16);
        attributes.attr1.set_obj_size(size as u16);
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

/// An object slot.
///
/// You must use [`ConsoleState::reserve_object`] to get a `Slot`, to pass it to
/// [`video::Control::object`] to get a [`Handle`] to be able to draw objects
/// on screen. (See [`Handle`] for details)
///
/// See [`self`] module doc for how to use objects.
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
        // SAFETY: `self.objects` is by definition lower than Self::MAX_BLOCKS,
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
///
/// See [`self`] module doc for how to use objects.
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
    /// Set `x` and `y` coordinate of object.
    pub fn set_pos(&mut self, pos: Pos) {
        self.set_x(pos.x);
        self.set_y(pos.y);
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
    pub fn set_visible(&mut self, visible: bool) {
        self.value.attr0.set_double_disabled(!visible);
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
    /// Set sprite used by object.
    ///
    /// # Panics
    ///
    /// (`"sane_assert"` only)
    /// If `self.palette_mode() == palette::Type::Full && tile.get() % 2 == 1`
    ///
    /// Without `sane_assert`, odd tiles won't have effect with a full palette mode.
    pub fn set_sprite(&mut self, sprite: sprite::Slot) {
        sane_assert!(!self.value.attr0.use_palbank() && tile.0 % 2 == 0);
        self.value.attr0.set_use_palbank(true);
        self.value.attr2.set_tile_index(sprite.offset);
    }
    /// Set palette mode used by object.
    ///
    /// Note that if not set to [`palette::Type::Bank`],
    /// [`Self::set_sprite`] will only accept even numbered tiles.
    pub fn set_palette_mode(&mut self, kind: palette::Type) {
        let use_palbank = matches!(kind, palette::Type::Bank);
        // TODO: the method in rust-console/gba is just wrongly named
        self.value.attr0.set_use_palbank(!use_palbank);
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
///
/// See [`self`] module doc for how to use objects.
pub struct Allocator {
    objects: Bitset128,
    sprites: Blocks<sprite::Id, SPRITE_MAX_BLOCKS>,
}
impl ConstDefault for Allocator {
    const DEFAULT: Self = Self {
        objects: Bitset128::DEFAULT,
        sprites: Blocks::new(SPRITE_FULL_SIZE),
    };
}
impl Allocator {
    // allow: the `assert!(free<128)` should ALWAYS be true, due to a check in
    // `self.objects.first_free`.
    /// Reserve an object slot.
    /// Returns `None` if no more slots are available.
    ///
    /// Make sure to call [`Allocator::free`] before dropping a [`Slot`],
    /// otherwise, the object slot will forever be leaked.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn reserve(&mut self) -> Option<Slot> {
        let free = self.objects.first_free()?;
        self.objects.reserve(free);
        assert!(free < 128);
        // SAFETY: `free` is always in 0..128.
        Some(unsafe { Slot::new_unchecked(free) })
    }
    // allow: `Slot` is meant to not be Copy or Clone, the goal of this method
    // is to provide an API where you can't have multiple handles to the same slot.
    /// Free an object slot, consuming it.
    #[allow(clippy::needless_pass_by_value)]
    pub fn free(&mut self, slot: Slot) {
        self.objects.free(slot.0);
    }

    /// Reserve a sprite.
    /// Returns `None` if all sprite tiles are allocated.
    /// Returns existing index if `id` is already allocated.
    ///
    /// This should be used in combination with [`video::Control::load_sprite`]
    /// in order to make sense. [`video::Control::unload_sprite`] should be used
    /// with [`Self::free_sprite`]
    #[must_use]
    pub(crate) fn reserve_sprite(&mut self, sprite: &Sprite) -> Option<sprite::Slot> {
        let shape = sprite.shape;
        let id = sprite.id;
        let free = self.sprites.insert_sized(id, shape.tile_count())?;
        // SAFETY: We assume that `Blocks::insert_size` implementation is correct,
        // and therefore will never allocate something outside of the provided
        // SPRITE_FULL_SIZE, which is 1024.
        Some(unsafe { sprite::Slot::new_unchecked(free) })
    }
    /// Remove sprite.
    #[allow(clippy::needless_pass_by_value)]
    pub(crate) fn free_sprite(&mut self, id: sprite::Id) -> bool {
        self.sprites.remove(id)
    }
    /// Replace sprite.
    pub(crate) fn replace_sprite(&mut self, old: sprite::Id, new: &Sprite) -> Option<sprite::Slot> {
        let Sprite { shape, id, .. } = new;
        self.sprites
            .replace_id(old, *id, shape.tile_count())
            // SAFETY: We assume that `Blocks::replace_id` implementation is correct,
            // and therefore an existing offset will always be bellow SPRITE_FULL_SIZE.
            .map(|offset| unsafe { sprite::Slot::new_unchecked(offset) })
    }
}
