use volmatrix::rw::{VolBlock, VolMatrix};

#[cfg(doc)]
use crate::video;

/// A specific CBB slot.
///
/// See [`video::Control::load_tileset`] for explanations on CBB.
#[derive(Clone, Copy)]
pub struct Slot(usize);
impl Slot {
    /// Get the offset-th next CBB, None if no such thing exists.
    #[must_use]
    pub(super) const fn add(self, offset: usize) -> Option<Self> {
        if self.0 + offset < Self::MAX_BLOCKS {
            // SAFETY: we make sure to not go over MAX_BLOCKS
            Some(unsafe { Self::new_unchecked(self.0 + offset) })
        } else {
            None
        }
    }
    /// How many Cbb slot there is.
    pub const MAX_BLOCKS: usize = super::CBB_COUNT;

    /// Create a new CBB slot.
    ///
    /// # Panics
    ///
    /// (const) when `inner >= Self::MAX_BLOCKS`.
    #[must_use]
    pub const fn new(inner: usize) -> Self {
        assert!(inner < Self::MAX_BLOCKS);
        Self(inner)
    }
    /// SAFETY: `inner` must be lower than [`Self::MAX_BLOCKS`]
    pub(super) const unsafe fn new_unchecked(inner: usize) -> Self {
        Self(inner)
    }
    pub(super) const fn index_volmatrix<T, const C: usize>(
        self,
        volmatrix: VolMatrix<T, C, { Self::MAX_BLOCKS }>,
    ) -> VolBlock<T, C> {
        // SAFETY: It is impossible to build a CbbSlot of higher value than Self::MAX_BLOCK.
        unsafe { volmatrix.row_unchecked(self.0) }
    }

    // SAFETY: for all the following const definitions: all values are bellow Self::MAX_BLOCKS
    pub const _0: Self = unsafe { Self::new_unchecked(0) };
    pub const _1: Self = unsafe { Self::new_unchecked(1) };
    pub const _2: Self = unsafe { Self::new_unchecked(2) };
    pub const _3: Self = unsafe { Self::new_unchecked(3) };

    /// Return value.
    ///
    /// By definition, the return value is smaller than `Self::MAX_BLOCKS`.
    #[allow(clippy::cast_possible_truncation)]
    pub(super) const fn get(self) -> u16 {
        self.0 as u16
    }
}
