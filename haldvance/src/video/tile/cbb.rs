use volmatrix::rw::{VolBlock, VolMatrix};

/// A specific CBB slot.
///
/// See [`super::VideoControl::load_tileset`] for explanations on CBB.
#[derive(Clone, Copy)]
pub struct Slot(usize);
impl Slot {
    /// Get the offset-th next CBB, None if no such thing exists.
    pub(super) const fn add(&self, offset: usize) -> Option<Self> {
        if self.0 + offset < Self::MAX_BLOCKS {
            // SAFETY: we make sure to not go over MAX_BLOCKS
            Some(unsafe { Self::new_unchecked(self.0 + offset) })
        } else {
            None
        }
    }
    /// How many Cbb slot there is.
    pub const MAX_BLOCKS: usize = super::CBB_COUNT;
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
}
