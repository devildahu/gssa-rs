//! Deal with tilemap-based backgrounds, or "layers."
use core::marker::PhantomData;

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

pub mod affine;
pub mod text;

type WoVolAddress<T> = volmatrix::VolAddress<T, (), volmatrix::Safe>; // owo

#[doc(hidden)]
pub trait Slot: Copy {
    fn register(self) -> VolAddress<BackgroundControl>;
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
    pub const fn into_pure_text(self) -> text::Slot {
        match self {
            Self::_0 => text::Slot::_0,
            Self::_1 => text::Slot::_1,
        }
    }
}

/// Background layer operations in [`Text`] or [`Mixed`] [`Mode`]s.
///
/// Note that the changes are only effective when the handle is dropped,
/// to avoid extraneous memory reads/writes.
pub struct Handle<'a, M: mode::Background> {
    _ctrl: &'a mut (),
    value: BackgroundControl,
    bg: M::Slot,
    _t: PhantomData<fn() -> M>,
}
impl<'a, M: mode::Background> Handle<'a, M> {
    pub(super) fn new<N: Mode>(ctrl: &'a mut video::Control<N>, bg: M::Slot) -> Self {
        Self {
            _ctrl: ctrl.erased(),
            value: bg.register().read(),
            bg,
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
    /// video_control.layer(layer::text::Slot::_0).set_color_mode::<colmod::Bit8>();
    /// ```
    pub fn set_color_mode<CM: ColorMode>(&mut self) {
        self.value = self.value.with_is_8bpp(CM::RAW_REPR);
    }
    fn commit(&mut self) {
        let register = self.bg.register();
        register.write(self.value);
    }
}
impl<'a, M: mode::Background> Drop for Handle<'a, M> {
    /// Commit all changes to video memory.
    fn drop(&mut self) {
        self.commit();
    }
}
