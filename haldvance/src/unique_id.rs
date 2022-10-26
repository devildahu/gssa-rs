//! Const-time unique ids based on [`TypeId`], see [`UniqueId`].
//!
//! See [this playground link] for details on how this works.
//!
//! [this playground link]: https://play.rust-lang.org/?version=nightly&gist=9d201bf0d0881cd289aabc09f5813d99

use core::any::TypeId;

/// A unique ID.
///
/// Use the [`unique_id!`] macro to create a unique id.
///
/// Unique ids can be compared for equality, each call to `unique_id` generates
/// a new unique id, which can be compared for equality. This is typically useful
/// to generate `const`s that can be checked independently for identity.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct UniqueId(TypeId);

impl UniqueId {
    #[doc(hidden)]
    #[must_use]
    pub const fn new<T: 'static>() -> Self {
        Self(TypeId::of::<T>())
    }
}

/// Generate a [`UniqueId`] unique to this macro invocation.
///
/// See [`UniqueId`] for details.
#[macro_export]
macro_rules! unique_id {
    () => {{
        enum ImSoSpecialUwu {}
        $crate::UniqueId::new::<ImSoSpecialUwu>()
    }};
}
