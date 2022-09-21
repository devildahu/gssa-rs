//! A 2d array of volatile addresses.
use voladdress::{Safe, VolAddress as GVolAddress, VolBlock as GVolBlock};

use crate::warn;

pub(crate) type VolAddress<T> = GVolAddress<T, Safe, Safe>;
pub(crate) type VolBlock<T, const C: usize> = GVolBlock<T, Safe, Safe, C>;

pub(crate) struct VolMatrix<T, const WIDTH: usize, const HEIGHT: usize> {
    vol_address: VolAddress<T>,
}
/// Direct index access methods.
impl<T, const WIDTH: usize, const HEIGHT: usize> VolMatrix<T, WIDTH, HEIGHT> {
    /// Create a two-dimensional table from a 1d `VolBlock`.
    ///
    /// Note that trying to create a `VolMatrix` from a `VolBlock` with less than
    /// `WIDTH * HEIGHT` elements results in a compilation failure.
    pub(crate) const fn from_block<const B: usize>(block: VolBlock<T, B>) -> Self
    where
        [(); B - WIDTH * HEIGHT]: Sized,
    {
        // SAFETY: block's safety requirement is that all VolAddress accessible within
        // it are safe, Self can only access those addresses, so Self::new requirement
        // is fulfilled.
        unsafe { Self::new(block.index(0).as_usize()) }
    }
    /// A [`VolAddress`] with matrix-style access pattern.
    ///
    /// # Safety
    ///
    /// The given address must be a valid [`VolAddress`] at each position in the
    /// matrix:
    ///
    /// ```text
    /// for all (X, Y) in (0..WIDTH, 0..HEIGHT):
    /// 	let accessible = address + size_of::<T>() * (X + WIDTH * Y);
    ///     assert_valid_voladdress(accessible);
    /// ```
    #[inline(always)]
    pub(crate) const unsafe fn new(address: usize) -> Self {
        Self {
            vol_address: VolAddress::new(address),
        }
    }
    #[inline(always)]
    pub(crate) const fn get(self, x: usize, y: usize) -> Option<VolAddress<T>> {
        if x < WIDTH && y < HEIGHT {
            // SAFETY: if x < WIDTH && y < HEIGHT
            Some(unsafe { self.get_unchecked(x, y) })
        } else {
            None
        }
    }
    /// # Safety
    ///
    /// `x + y * WIDTH` must be lower than WIDTH * HEIGHT.
    ///
    /// Though, semantically, you should probably make sure that `x < WIDTH` and `y < HEIGHT`.
    #[inline(always)]
    pub(crate) const unsafe fn get_unchecked(self, x: usize, y: usize) -> VolAddress<T> {
        // SAFETY: upheld by function safety requirements
        self.vol_address.add(x + y * WIDTH)
    }
}

/// Row access methods.
impl<T, const WIDTH: usize, const HEIGHT: usize> VolMatrix<T, WIDTH, HEIGHT> {
    /// # Safety
    ///
    /// `y < HEIGHT`.
    #[inline(always)]
    pub(crate) const unsafe fn row_unchecked(self, y: usize) -> VolBlock<T, WIDTH> {
        // SAFETY:
        // - function safety condition: `y < HEIGHT`
        // - `VolMatrix::new` safety condition guarentees that all addresses
        //   constructible for `VolBlock<T, WIDTH>` are valid `VolAddress`,
        //   which is the safety condition of `VolBlock::new`.
        VolBlock::new(self.vol_address.add(y * WIDTH).as_usize())
    }
    #[inline(always)]
    pub(crate) const fn get_row(self, y: usize) -> Option<VolBlock<T, WIDTH>> {
        if y < HEIGHT {
            // SAFETY: if y < HEIGHT
            Some(unsafe { self.row_unchecked(y) })
        } else {
            None
        }
    }
}

// /// Column access methods.
// impl<T, const WIDTH: usize, const HEIGHT: usize> VolMatrix<T, WIDTH, HEIGHT> {
//     /// # Safety
//     ///
//     /// `x < WIDTH`.
//     #[inline(always)]
//     pub(crate) const unsafe fn column_unchecked(self, x: usize) -> VolSeries<T, HEIGHT, {WIDTH * size_of::<T>()}> {
//         // SAFETY:
//         // - function safety condition: `y < HEIGHT`
//         // - `VolMatrix::new` safety condition guarentees that all addresses
//         //   constructible for `VolSeries<T, HEIGHT, WIDTH * size_of::<T>()>` are valid `VolAddress`,
//         //   which is the safety condition of `VolSeries::new`.
//         VolSeries::new(self.vol_address.offset(x as isize).as_usize())
//     }
// }

/// Extension trait to [`VolBlock`] and [`VolMatrix`] for bulk volatile
/// load/store.
///
/// Currently just a very basic call to ptr::write_volatile, but later may
/// use optimized intrisics (currently want to get it working, AND need to
/// define myself the optimized intrisics, so it's not ready tomorrow)
pub(crate) trait VolMemcopy<T>: Sized {
    // TODO: consider const_generic versions
    fn write_slice_at_offset(self, offset: usize, slice: &[T]);
    fn read_offset_into_slice(self, offset: usize, slice: &mut [T]);
    fn write_slice(self, slice: &[T]) {
        self.write_slice_at_offset(0, slice);
    }
    fn read_into_slice(self, slice: &mut [T]) {
        self.read_offset_into_slice(0, slice);
    }
}
/// Print a warning on specific situation.
///
/// `write_slice_at_offset` and `read_offset_into_slice` are
/// not unsound if the requested len is larger than the available one,
/// but it is likely to be a programming error, so we warn if we detect
/// that we cannot fill/read fully the passed slice.
fn warn_out_of_bound<T>(offset: usize, len: usize, max: usize) {
    warn!((offset + len > max) "{}: {offset} + {len} gt {max}", core::any::type_name::<T>(),)
}
impl<T: Copy, const C: usize> VolMemcopy<T> for VolBlock<T, C> {
    fn write_slice_at_offset(self, offset: usize, slice: &[T]) {
        warn_out_of_bound::<Self>(offset, slice.len(), C);
        let iter = self.iter().skip(offset).zip(slice.iter());
        iter.for_each(|(addr, value)| addr.write(*value))
    }
    fn read_offset_into_slice(self, offset: usize, slice: &mut [T]) {
        warn_out_of_bound::<Self>(offset, slice.len(), C);
        let iter = self.iter().skip(offset).zip(slice.iter_mut());
        iter.for_each(|(addr, value)| *value = addr.read())
    }
}
