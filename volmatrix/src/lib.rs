//! Extensions to the [`voladdress`] crate, including [`VolMatrix`] and [`VolMemcopy`].
#![no_std]
#![warn(clippy::pedantic, clippy::nursery)]
#![forbid(missing_docs)]
// allow(incomplete_features): We only use generic_const_exprs
// when the "nightly" feature is active, meaning that the user
// can simply disable the feature if it comes to break.
// We also only use full generic_const_exprs to check for
// the column methods on VolMatrix and for compile-time check on from_block
#![cfg_attr(feature = "nightly", allow(incomplete_features))]
#![cfg_attr(feature = "nightly", feature(generic_const_exprs))]

mod volmatrix;
mod volmemcopy;

pub use voladdress::{Safe, Unsafe, VolAddress, VolBlock, VolSeries};
pub use volmatrix::VolMatrix;
pub use volmemcopy::VolMemcopy;

/// Safe Read/Write [`voladdress`] structs.
pub mod rw {
    use voladdress::Safe;

    /// Shortcut for [`voladdress::VolAddress<T, Safe, Safe>`].
    pub type VolAddress<T> = super::VolAddress<T, Safe, Safe>;
    /// Shortcut for [`voladdress::VolBlock<T, Safe, Safe, C>`].
    pub type VolBlock<T, const C: usize> = super::VolBlock<T, Safe, Safe, C>;
    /// Shortcut for [`crate::VolMatrix<T, Safe, Safe, W, H>`].
    pub type VolMatrix<T, const W: usize, const H: usize> = super::VolMatrix<T, Safe, Safe, W, H>;
}
