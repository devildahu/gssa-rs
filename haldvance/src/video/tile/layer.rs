//! Deal with tilemap-based backgrounds, or "layers."
use core::marker::PhantomData;

use gba::mmio_addresses::{BG0CNT, BG1CNT, BG2CNT, BG3CNT};
use gba::mmio_types::BackgroundControl;
use volmatrix::rw::VolAddress;

use crate::video::{mode::TileMode, tile::sbb, ColorMode, Mode, Priority, VideoControl};

#[cfg(doc)]
use crate::video::mode::{Mixed, Text};

/// Background layers accessible in [`Text`] [`Mode`].
///
/// To manipulate the background, get a [`Handle`] from
/// [`VideoControl<Text>::layer`] or [`VideoControl<Mixed>::text_layer`]
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
/// [`VideoControl<Mixed>::text_layer`]
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

/// Background layer operations in [`Text`] or [`Mixed`] [`Mode`]s.
///
/// Note that the changes are only effective when the handle is dropped,
/// to avoid extraneous memory reads/writes.
pub struct Handle<'a, M: TileMode> {
    _ctrl: &'a mut (),
    value: BackgroundControl,
    register: VolAddress<BackgroundControl>,
    _t: PhantomData<fn() -> M>,
}
impl<'a, M: TileMode> Handle<'a, M> {
    pub(super) fn new<N: Mode>(ctrl: &'a mut VideoControl<N>, bg: Slot) -> Self {
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
impl<'a, M: TileMode> Drop for Handle<'a, M> {
    /// Commit all changes to video memory.
    fn drop(&mut self) {
        self.commit();
    }
}
