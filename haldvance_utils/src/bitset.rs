//! Bitset for the object allocator.

use const_default::ConstDefault;

#[derive(Clone, Copy, PartialEq, Eq, ConstDefault)]
pub struct Bitset128(u128);
impl Bitset128 {
    /// Number of indexes supported by this bitset.
    pub const INDEX_COUNT: u32 = u128::BITS;

    /// Return the first non-taken index.
    /// `None` if all indices are taken.
    #[must_use]
    pub const fn first_free(&self) -> Option<u32> {
        let first = self.0.trailing_ones();
        // TODO: when const_bool_to_option stabilize, replace this with `.then`
        if first < Self::INDEX_COUNT {
            Some(first)
        } else {
            None
        }
    }
    /// Reserve given `index % 128`, return `true` if the index was already in use.
    pub fn reserve(&mut self, index: u32) -> bool {
        let mask: u128 = 1 << index;
        let already_taken = self.0 & mask != 0;
        self.0 |= mask;
        already_taken
    }
    /// Free given `index % 128`, return `true` if the index was already free.
    pub fn free(&mut self, index: u32) -> bool {
        let mask: u128 = 1 << index;
        let already_free = self.0 & mask == 0;
        self.0 &= !mask;
        already_free
    }
}
