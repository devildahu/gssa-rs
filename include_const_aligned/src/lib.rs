//! Macros to handle including arbitrary data into executables.
//!
//! Note that `include_const_transmutted` requires the `const_mut_refs`
//! nightly feature (see <https://github.com/rust-lang/rust/issues/67456>).
#![no_std]

/// Like [`include_bytes!`], but allow specification of the alignment.
#[macro_export]
macro_rules! include_const_aligned {
    ($align_to:expr, $path:expr) => {{
        #[repr(align($align_to))]
        struct Aligned<T: ?Sized>(T);
        const DATA: &'static Aligned<[u8]> = &Aligned(*include_bytes!($path));
        &DATA.0
    }};
}

/// Like [`include_bytes!`], but allow transmuttation to arbitrary types.
///
/// # Panics
///
/// When the size of the file in `$path` must be a multiple of `size_of<$T>`.
///
/// # Safety
///
/// - The bit pattern in file specified with `$path` must be a valid `$T`
/// - `$align_to` must be a valid alignment for `$T`
#[macro_export]
macro_rules! include_const_transmutted {
    ($align_to:expr, $path:expr, $T:ty $(,)?) => {{
        // Define in a const fn to make sure we are not
        // accidentally adding runtime overhead.
        const unsafe fn read_data() -> &'static [$T] {
            let data = $crate::include_const_aligned!($align_to, $path);
            let len = data.len();
            let t_size = ::core::mem::size_of::<$T>();
            if len % t_size != 0 {
                panic!(concat!(
                    "in include_const_transmutted, file at: \"",
                    $path,
                    "\" doesn't have a size multiple of `size_of::<",
                    stringify!($T),
                    ">()`",
                ));
            }
            let t_len = len / t_size;
            ::core::slice::from_raw_parts(data.as_ptr().cast::<$T>(), t_len)
        }
        read_data()
    }};
}
