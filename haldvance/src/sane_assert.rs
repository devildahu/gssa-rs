//! Defines [`crate::sane_assert!`].

/// Like `assert!`, but only active when `"sane_assert"` cargo flag is active.
#[macro_export]
macro_rules! sane_assert {
    ($($args:tt)*) => {
        #[cfg(feature = "sane_assert")]
        assert!($($args)*)
    }
}
