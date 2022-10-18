use gba::mmio_addresses::{BG0CNT, BG1CNT, BG2CNT, BG3CNT};
use gba::mmio_types::BackgroundControl;
use volmatrix::rw::VolAddress;

use super::{mode, Handle, TextSize, WoVolAddress};

#[cfg(doc)]
use super::*;

const BG_OFS_ADDR_USIZE: usize = 0x400_0010;

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
impl super::Slot for Slot {
    fn register(self) -> VolAddress<BackgroundControl> {
        match self {
            Self::_0 => BG0CNT,
            Self::_1 => BG1CNT,
            Self::_2 => BG2CNT,
            Self::_3 => BG3CNT,
        }
    }
}
impl Slot {
    const fn offset_register(self) -> (WoVolAddress<u16>, WoVolAddress<u16>) {
        let stride = (self as usize) * 4;
        let address = BG_OFS_ADDR_USIZE + stride;
        // SAFETY: within the VRAM
        unsafe { (WoVolAddress::new(address), WoVolAddress::new(address + 2)) }
    }
}
impl<'a> Handle<'a, mode::Text> {
    pub fn set_size(&mut self, size: TextSize) {
        self.value = self.value.with_screen_size(size as u8);
    }
    pub fn set_x_offset(&mut self, offset: u16) {
        let register = self.bg.offset_register().0;
        register.write(offset);
    }
    pub fn set_y_offset(&mut self, offset: u16) {
        let register = self.bg.offset_register().1;
        register.write(offset);
    }
}
