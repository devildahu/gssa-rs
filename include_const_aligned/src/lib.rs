//! Macros to handle including arbitrary data into executables.
//!
//! Note that `include_const_transmutted` requires the `const_mut_refs`
//! nightly feature (see <https://github.com/rust-lang/rust/issues/67456>).
//! Meaning that if you want to use it, you must add `#![feature(const_mut_refs)]`
//! to your crate's `lib.rs` or `main.rs`.
#![no_std]

/// Like [`include_bytes!`], but allow specification of the alignment.
#[macro_export]
macro_rules! include_const_aligned {
    ($align_to:literal, $path:expr $(,)*) => {{
        #[repr(C, align($align_to))]
        struct Aligned<T: ?Sized> {
            data: T,
        }
        const DATA: &Aligned<[u8]> = &Aligned { data: *include_bytes!($path) };
        &DATA.data
    }};
    ($align_to:ty, $path:expr $(,)*) => {{
        #[repr(C)]
        struct Aligned<Align, T: ?Sized> {
            _align: [Align; 0],
            data: T,
        }
        const DATA: &Aligned<$align_to, [u8]> =
            &Aligned { _align: [], data: *include_bytes!($path) };
        &DATA.data
    }};
}

/// Like [`include_bytes!`], but allow transmuttation to arbitrary types.
///
/// # Compile time Panics
///
/// When the size of the file in `$path` is not a multiple of `size_of<$T>`.
///
/// # Safety
///
/// The bit pattern in file specified with `$path` must be a valid `$T`.
#[macro_export]
macro_rules! include_const_transmutted {
    (@data $data:ident, $T:ty, $path:expr) => {{
        let len = $data.len();
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
        ::core::slice::from_raw_parts($data.as_ptr().cast::<$T>(), t_len)
    }};
    ($align_to:literal, $T:ty, $path:expr $(,)*) => {{
        const unsafe fn read_data() -> &'static [$T] {
            let data: &[u8] = $crate::include_const_aligned!($align_to, $path);
            $crate::include_const_transmutted!(@data data, $T, $path)
        }
        read_data()
    }};
    ($T:ty, $path:expr $(,)*) => {{
        // Define in a const fn to make sure we are not
        // accidentally adding runtime overhead.
        const unsafe fn read_data() -> &'static [$T] {
            let data: &[u8] = $crate::include_const_aligned!($T, $path);
            $crate::include_const_transmutted!(@data data, $T, $path)
        }
        read_data()
    }};
}
