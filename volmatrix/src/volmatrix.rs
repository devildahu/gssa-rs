use core::mem;

use voladdress::{VolAddress, VolBlock, VolSeries};

/// A 2D version of [`VolBlock`] with a given `WIDTH` and `HEIGHT`,
/// see the [`VolAddress`] documentation for details.
pub struct VolMatrix<T, R, W, const WIDTH: usize, const HEIGHT: usize> {
    vol_address: VolAddress<T, R, W>,
}

/// Direct index access methods.
impl<T, R, W, const WIDTH: usize, const HEIGHT: usize> VolMatrix<T, R, W, WIDTH, HEIGHT> {
    /// Create a two-dimensional table from a 1d `VolBlock`.
    ///
    /// Note that trying to create a `VolMatrix` from a `VolBlock` with less than
    /// `WIDTH * HEIGHT` elements results in a compilation failure ("attempt to
    /// create a negative array size").
    #[cfg(feature = "nightly")]
    #[must_use]
    pub const fn from_block<const B: usize>(block: VolBlock<T, R, W, B>) -> Self
    where
        // compile time check that B > W*H
        [(); B - WIDTH * HEIGHT]: Sized,
    {
        // SAFETY: block's safety requirement is that all VolAddress accessible within
        // it are safe, Self can only access those addresses, so Self::new requirement
        // is fulfilled.
        unsafe { Self::new(block.index(0).as_usize()) }
    }
    /// Create a two-dimensional table from a 1d `VolBlock`.
    ///
    /// # Panics
    ///
    /// Panics if `B < WIDTH * HEIGHT` (ie: unsafe). Since B, WIDTH and HEIGHT are
    /// `const`, the panic should be unconditional.
    ///
    /// Use the `nightly` feature to replace the assertion with a type error.
    #[cfg(not(feature = "nightly"))]
    #[must_use]
    pub const fn from_block<const B: usize>(block: VolBlock<T, R, W, B>) -> Self {
        assert!(B >= WIDTH * HEIGHT);
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
    ///     let accessible = address + mem::size_of::<T>() * (X + WIDTH * Y);
    ///     assert_valid_voladdress(accessible);
    /// ```
    #[must_use]
    pub const unsafe fn new(address: usize) -> Self {
        Self {
            vol_address: VolAddress::new(address),
        }
    }
    /// Get the [`VolAddress`] at specified matrix location, returns
    /// `None` if out of bound.
    ///
    /// Use [`VolMatrix::get_unchecked`] to skip bound checks.
    #[must_use]
    pub const fn get(self, x: usize, y: usize) -> Option<VolAddress<T, R, W>> {
        if x < WIDTH && y < HEIGHT {
            // SAFETY: if x < WIDTH && y < HEIGHT
            Some(unsafe { self.get_unchecked(x, y) })
        } else {
            None
        }
    }
    /// Get the [`VolAddress`] at specified matrix location.
    ///
    /// Use [`VolMatrix::get`] for a safe version.
    ///
    /// # Safety
    ///
    /// `x + y * WIDTH` must be lower than WIDTH * HEIGHT.
    ///
    /// Though, semantically, you should probably make sure that `x < WIDTH` and `y < HEIGHT`.
    #[must_use]
    pub const unsafe fn get_unchecked(self, x: usize, y: usize) -> VolAddress<T, R, W> {
        // SAFETY: upheld by function safety requirements
        self.vol_address.add(x + y * WIDTH)
    }
}

/// Row access methods.
impl<T, R, W, const WIDTH: usize, const HEIGHT: usize> VolMatrix<T, R, W, WIDTH, HEIGHT> {
    /// Get a signle row of the matrix as a [`VolBlock`].
    ///
    /// Use [`VolMatrix::get_row`] for a safe version.
    ///
    /// # Safety
    ///
    /// `y < HEIGHT`.
    #[must_use]
    pub const unsafe fn row_unchecked(self, y: usize) -> VolBlock<T, R, W, WIDTH> {
        // SAFETY:
        // - function safety condition: `y < HEIGHT`
        // - `VolMatrix::new` safety condition guarentees that all addresses
        //   constructible for `VolBlock<T, WIDTH>` are valid `VolAddress`,
        //   which is the safety condition of `VolBlock::new`.
        VolBlock::new(self.vol_address.add(y * WIDTH).as_usize())
    }
    /// Get a signle row of the matrix as a [`VolBlock`].
    ///
    /// Use [`VolMatrix::row_unchecked`] to skip bound checks.
    #[must_use]
    pub const fn get_row(self, y: usize) -> Option<VolBlock<T, R, W, WIDTH>> {
        if y < HEIGHT {
            // SAFETY: if y < HEIGHT
            Some(unsafe { self.row_unchecked(y) })
        } else {
            None
        }
    }
}

/// Column access methods.
#[cfg(feature = "nightly")]
impl<T, R, W, const WIDTH: usize, const HEIGHT: usize> VolMatrix<T, R, W, WIDTH, HEIGHT> {
    /// Get a signle column of the matrix as a [`VolSeries`].
    ///
    /// Use [`VolMatrix::get_column`] for a safe version.
    /// # Safety
    ///
    /// `x < WIDTH`.
    #[must_use]
    pub const unsafe fn column_unchecked(
        self,
        x: usize,
    ) -> VolSeries<T, R, W, HEIGHT, { WIDTH * mem::size_of::<T>() }> {
        // SAFETY:
        // - function safety condition: `y < HEIGHT`
        // - `VolMatrix::new` safety condition guarentees that all addresses
        //   constructible for `VolSeries<T, HEIGHT, WIDTH * mem::size_of::<T>()>` are valid `VolAddress`,
        //   which is the safety condition of `VolSeries::new`.
        VolSeries::new(self.vol_address.add(x).as_usize())
    }
    /// Get a signle column of the matrix as a [`VolSeries`].
    ///
    /// Use [`VolMatrix::column_unchecked`] to skip bound checks.
    #[must_use]
    pub const fn get_column(
        self,
        x: usize,
    ) -> Option<VolSeries<T, R, W, HEIGHT, { WIDTH * mem::size_of::<T>() }>> {
        if x < WIDTH {
            // SAFETY: `if x < WIDTH`
            Some(unsafe { self.column_unchecked(x) })
        } else {
            None
        }
    }
}
