//! Deal with tilemap-based backgrounds, or "layers."
use core::marker::PhantomData;

use gba::mmio_addresses::{BG0CNT, BG1CNT, BG2CNT, BG3CNT};
use gba::mmio_types::BackgroundControl;
use volmatrix::rw::VolAddress;

use crate::video::{
    self, mode,
    tile::{
        cbb,
        map::{AffineSize, TextSize},
        sbb,
    },
    ColorMode, Mode, Priority,
};

#[cfg(doc)]
use crate::video::mode::{Affine, Mixed, Text};

/// Background layers accessible in [`Text`] [`Mode`].
///
/// To manipulate the background, get a [`Handle`] from
/// [`video::Control<Text>::layer`] or [`video::Control<Mixed>::text_layer`]
/// and use the methods on [`Handle`].
#[derive(Clone, Copy)]
#[repr(u16)]
pub enum Slot {
    _0 = 0,
    _1 = 1,
    _2 = 2,
    _3 = 3,
}
impl Slot {
    pub(super) const fn register(self) -> VolAddress<BackgroundControl> {
        match self {
            Self::_0 => BG0CNT,
            Self::_1 => BG1CNT,
            Self::_2 => BG2CNT,
            Self::_3 => BG3CNT,
        }
    }
}

/// Text background layers accessible in [`Mixed`] [`Mode`].
///
/// To manipulate the background, get a [`Handle`] from
/// [`video::Control<Mixed>::text_layer`]
/// and use the methods on [`Handle`].
#[derive(Clone, Copy)]
#[repr(u16)]
pub enum MixedSlot {
    _0 = 0,
    _1 = 1,
}
impl MixedSlot {
    #[must_use]
    pub const fn into_pure_text(self) -> Slot {
        match self {
            Self::_0 => Slot::_0,
            Self::_1 => Slot::_1,
        }
    }
}

/// Text background layers accessible in [`Affine`] [`Mode`].
///
/// To manipulate the background, get a [`Handle`] from
/// [`video::Control<Affine>::text_layer`]
/// and use the methods on [`Handle`].
#[derive(Clone, Copy)]
#[repr(u16)]
pub enum AffineSlot {
    _2 = 2,
    _3 = 3,
}
impl AffineSlot {
    #[must_use]
    pub const fn into_pure_text(self) -> Slot {
        match self {
            Self::_2 => Slot::_2,
            Self::_3 => Slot::_3,
        }
    }
}

/// Background layer operations in [`Text`] or [`Mixed`] [`Mode`]s.
///
/// Note that the changes are only effective when the handle is dropped,
/// to avoid extraneous memory reads/writes.
pub struct Handle<'a, M: mode::Tile> {
    _ctrl: &'a mut (),
    value: BackgroundControl,
    register: VolAddress<BackgroundControl>,
    _t: PhantomData<fn() -> M>,
}
impl<'a, M: mode::Tile> Handle<'a, M> {
    pub(super) fn new<N: Mode>(ctrl: &'a mut video::Control<N>, bg: Slot) -> Self {
        let register = bg.register();
        Self {
            _ctrl: ctrl.erased(),
            value: register.read(),
            register,
            _t: PhantomData,
        }
    }
    /// Set priority of this layer, returning the previous priority.
    pub fn set_priority(&mut self, priority: Priority) -> Priority {
        let old_priority = unsafe {
            // SAFETY: return value of `bg_priority` is always `ret & 0b11`.
            Priority::new_unchecked(u16::from(self.value.priority()))
        };
        self.value = self.value.with_priority(priority as u8);
        old_priority
    }

    /// Set SBB of this layer, returning the previous SBB.
    #[allow(clippy::cast_possible_truncation)]
    pub fn set_sbb(&mut self, sbb: sbb::Slot) -> sbb::Slot {
        let old_sbb = sbb::Slot::new(self.value.screen_base_block() as usize);
        self.value = self.value.with_screen_base_block(sbb.get() as u8);
        old_sbb
    }

    /// Set CBB of this layer, returning the previous CBB.
    #[allow(clippy::cast_possible_truncation)]
    pub fn set_cbb(&mut self, cbb: cbb::Slot) -> cbb::Slot {
        let old_cbb = cbb::Slot::new(self.value.char_base_block() as usize);
        self.value = self.value.with_char_base_block(cbb.get() as u8);
        old_cbb
    }

    /// Set color mode of this layer.
    ///
    /// # Usage
    ///
    /// ```
    /// use haldvance::video::{colmod, tile::layer};
    /// video_control.layer(layer::Slot::_0).set_color_mode::<colmod::Bit8>();
    /// ```
    pub fn set_color_mode<CM: ColorMode>(&mut self) {
        self.value = self.value.with_is_8bpp(CM::RAW_REPR);
    }
    fn commit(&mut self) {
        self.register.write(self.value);
    }
}
impl<'a> Handle<'a, mode::Text> {
    pub fn set_size(&mut self, size: TextSize) {
        self.value = self.value.with_screen_size(size as u8);
    }
}
impl<'a> Handle<'a, mode::Affine> {
    pub fn set_size(&mut self, size: AffineSize) {
        self.value = self.value.with_screen_size(size as u8);
    }
    /// Set whether the map should wrap, only available in [`Affine`] mode.
    pub fn set_overflow(&mut self, overflows: bool) {
        self.value = self.value.with_affine_overflow_wrapped(overflows);
    }
}

impl<'a, M: mode::Tile> Drop for Handle<'a, M> {
    /// Commit all changes to video memory.
    fn drop(&mut self) {
        self.commit();
    }
}
