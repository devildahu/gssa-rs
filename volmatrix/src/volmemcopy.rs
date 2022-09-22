use voladdress::{Safe, VolBlock};

/// Extension trait to [`VolBlock`] for bulk volatile load/store.
///
/// Currently just a very basic call to ptr::write_volatile, but later may
/// use optimized intrisics (currently want to get it working, AND need to
/// define myself the optimized intrisics, so it's not ready tomorrow)
///
/// [`VolMatrix`]: crate::VolMatrix
pub trait VolMemcopy<T>: Sized {
    /// Write content of `slice` into the volatile store starting
    /// at `offset`.
    ///
    /// If the `slice` is larger than (store_len - offset), then
    /// the remaining values are not written to store.
    fn write_slice_at_offset(self, offset: usize, slice: &[T]);

    /// Read content of the volatile store starting at `offset` into `slice`.
    ///
    /// Only at most `min(slice.len, store_len - offset)` elements are read.
    fn read_offset_into_slice(self, offset: usize, slice: &mut [T]);

    /// Write content of `slice` into the volatile store.
    ///
    /// If the `slice` is larger than store_len, then
    /// the remaining values are not written to store.
    fn write_slice(self, slice: &[T]) {
        self.write_slice_at_offset(0, slice);
    }

    /// Read content of the volatile store into `slice`.
    ///
    /// Only at most `min(slice.len, store_len)` elements are read.
    fn read_into_slice(self, slice: &mut [T]) {
        self.read_offset_into_slice(0, slice);
    }
}
impl<T: Copy, const C: usize> VolMemcopy<T> for VolBlock<T, Safe, Safe, C> {
    fn write_slice_at_offset(self, offset: usize, slice: &[T]) {
        let iter = self.iter().skip(offset).zip(slice.iter());
        iter.for_each(|(addr, value)| addr.write(*value));
    }
    fn read_offset_into_slice(self, offset: usize, slice: &mut [T]) {
        let iter = self.iter().skip(offset).zip(slice.iter_mut());
        iter.for_each(|(addr, value)| *value = addr.read());
    }
}
