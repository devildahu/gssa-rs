//! Text background layers accessible in [`Affine`] [`Mode`].

use gba::mmio_addresses::{BG2CNT, BG3CNT};
use gba::mmio_types::BackgroundControl;
use volmatrix::rw::VolAddress;

use super::{mode, AffineSize, Handle, WoVolAddress};

#[cfg(doc)]
use super::*;

const REG_AFFINE_BG_PARAMETERS_ADDR_USIZE: usize = 0x400_0020;
const REG_AFFINE_OFFSET_ADDR_USIZE: usize = 0x400_0028;
const REG_AFFINE_TRS_STRIDE: usize = 0x10;

#[repr(C)]
#[derive(Clone, Copy)]
struct RotationScale {
    t_00: i16,
    t_01: i16,
    t_10: i16,
    t_11: i16,
}

/// Text background layers accessible in [`Affine`] [`Mode`].
///
/// To manipulate the background, get a [`Handle`] from
/// [`video::Control<Affine>::text_layer`]
/// and use the methods on [`Handle`].
#[derive(Clone, Copy)]
#[repr(u16)]
pub enum Slot {
    _2 = 2,
    _3 = 3,
}
impl super::Slot for Slot {
    fn register(self) -> VolAddress<BackgroundControl> {
        match self {
            Self::_2 => BG2CNT,
            Self::_3 => BG3CNT,
        }
    }
}
impl Slot {
    const fn offset_register(self) -> (WoVolAddress<i32>, WoVolAddress<i32>) {
        let stride = ((self as usize) - 2) * REG_AFFINE_TRS_STRIDE;
        let address = REG_AFFINE_OFFSET_ADDR_USIZE + stride;
        // SAFETY: within the VRAM
        unsafe { (WoVolAddress::new(address), WoVolAddress::new(address + 4)) }
    }
    const fn rot_scale_register(self) -> WoVolAddress<RotationScale> {
        // SAFETY: within the VRAM
        let stride = ((self as usize) - 2) * REG_AFFINE_TRS_STRIDE;
        let address = REG_AFFINE_BG_PARAMETERS_ADDR_USIZE + stride;
        unsafe { WoVolAddress::new(address) }
    }
}
/// [`mode::Affine`] specific layer controls.
///
/// In this mode, it's possible to [offset], [scale] and [rotate] the background.
///
/// [scale]: Handle::set_transform
/// [offset]: Handle::set_x_offset
/// [rotate]: Handle::set_transform
impl<'a> Handle<'a, mode::Affine> {
    pub fn set_size(&mut self, size: AffineSize) {
        self.value = self.value.with_screen_size(size as u8);
    }
    /// Set whether the map should wrap, only available in [`Affine`] mode.
    pub fn set_overflow(&mut self, overflows: bool) {
        self.value = self.value.with_affine_overflow_wrapped(overflows);
    }
    /// Set the x offset of the background, useful for scrolling.
    ///
    /// Note that the `i32` represents a fixed point fractional integer of size
    /// 20.8.
    ///
    /// See the [Tonc article] for details on how to use affine mode
    /// transform/rotate/scale registers.
    ///
    /// [Tonc article]: https://www.coranac.com/tonc/text/affbg.htm
    pub fn set_x_offset(&mut self, offset: i32) {
        let register = self.bg.offset_register().0;
        register.write(offset);
    }
    /// Set the y offset of the background, useful for scrolling.
    ///
    /// See [`Handle::set_x_offset`] for details.
    pub fn set_y_offset(&mut self, offset: i32) {
        let register = self.bg.offset_register().1;
        register.write(offset);
    }
    // TODO: implement a proper API.
    pub fn set_transform(&mut self, t_00: i16, t_01: i16, t_10: i16, t_11: i16) {
        let register = self.bg.rot_scale_register();
        register.write(RotationScale { t_00, t_01, t_10, t_11 });
    }
}
