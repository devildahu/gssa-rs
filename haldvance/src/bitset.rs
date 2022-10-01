//! Bitset for the object allocator.

use const_default::ConstDefault;

#[derive(Clone, Copy, PartialEq, Eq, ConstDefault)]
pub(crate) struct Bitset128(u128);
impl Bitset128 {
    /// Return the first non-taken index.
    /// `None` if all indices are taken.
    #[must_use]
    pub(crate) const fn first_free(&self) -> Option<u32> {
        let first = self.0.leading_ones();
        // TODO: when const_bool_to_option stabilize, replace this with `.then`
        if first == u128::BITS {
            None
        } else {
            Some(first)
        }
    }
    /// Reserve given `index`, return `true` if the index was already in use.
    pub(crate) fn take(&mut self, index: u32) -> bool {
        let mask: u128 = 1 << index;
        let already_taken = self.0 & mask != 0;
        self.0 |= mask;
        already_taken
    }
    /// Free given `index`, return `true` if the index was already free.
    pub(crate) fn free(&mut self, index: u32) -> bool {
        let mask: u128 = 1 << index;
        let already_free = self.0 & mask == 0;
        self.0 &= !mask;
        already_free
    }
}
