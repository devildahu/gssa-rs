/// Define an enum with the `next` and `prev` methods, allowing
/// to cycle through each enum variants.
///
/// This only works with C-style, dataless enums.
#[macro_export]
macro_rules! cycling_enum {
    ($(#[$attrs:meta])* $privacy:vis enum $name:ident { $( $entries:ident ),* $(,)?}) => {
        $crate::cycling_enum!{@inner
            ([$(stringify!($entries),)*].len()),
            $name,
            [$($attrs)*],
            [$($entries,)*],
            $privacy,
        }
    };
    (@inner
        $max_count:expr,
        $name:ident,
        [$($attrs:meta)*],
        [$( $entries:ident, )*],
        $privacy:vis,
    ) => {
        $(#[$attrs])*
        #[repr(isize)]
        #[allow(dead_code)]
        $privacy enum $name { $($entries),* }
        impl $name {
            #[doc(hidden)]
            const LEN: isize = $max_count as isize;
            $privacy const fn next(self) -> Self {
                let index = self as isize;
                let next: isize = match index + 1 {
                    Self::LEN => 0,
                    any_else => any_else,
                };
                // SAFETY: we guarentee we are within bounds of the enum, since
                // we define it here, and make sure they are within the range
                // (0..$entries.len())
                unsafe { ::core::mem::transmute(next) }
            }
            $privacy const fn prev(self) -> Self {
                let index = self as isize;
                let previous: isize = match index - 1 {
                    -1 => Self::LEN - 1,
                    any_else => any_else,
                };
                // SAFETY: we guarentee we are within bounds of the enum, since
                // we define it here, and make sure they are within the range
                // (0..$entries.len())
                unsafe { ::core::mem::transmute(previous) }
            }
        }
    }
}
