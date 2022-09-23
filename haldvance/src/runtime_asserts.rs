//! Logging utilities.
//!
//! This is mostly taken from the [`gba::debugging`] macros,
//! with the addition of the feature flag, the predicate and
//! the file and line number.

// Used in the various macros defined here.
#[doc(hidden)]
pub use gba;

/// Log something to mGBA emulator log.
#[doc = include_str!("runtime_asserts_doc_start.md")]
/// ```
/// emlog!($loglevel: ident, [($predicate: expr)]? $format_string [, $format_args]*)
/// ```
/// Where:
/// - `$loglevel`: one of `Fatal`, `Error`, `Warning`, `Info` or `Debug`;
///   the log level, configurable with the `--log-level N<128` mgba
///   command line option
#[doc = include_str!("runtime_asserts_doc_arguments.md")]
#[macro_export]
macro_rules! emlog {
    ($loglevel:ident, ($cond:expr) $fmt:literal, $($fmt_args:tt)*) => {
        #[cfg(feature = "runtime_asserts")]
        if $cond {
            use $crate::runtime_asserts::gba::debugging::mgba;
            use core::fmt::Write;
            let mut out = mgba::MGBADebug::new().unwrap();
            let _ = out.write_str(concat!("[", file!(), ":", line!(), "] "));
            let _ = write!(&mut out, $fmt , $($fmt_args)* );
            out.send(mgba::MGBADebugLevel::Warning);
        }
    };
    ($loglevel:ident, ($cond:expr) $fmt:literal) => {
        $crate::emlog!($loglevel, ($cond) $fmt,)
    };
    ($loglevel:ident,  $fmt:literal, $($fmt_args:tt)*) => {
        $crate::emlog!($loglevel, (true) $fmt, $($fmt_args)*)
    };
    ($loglevel:ident, $fmt:literal) => {
        $crate::emlog!($loglevel, (true) $fmt,)
    };
}

/// Log a fatal error to mGBA emulator log.
#[doc = include_str!("runtime_asserts_doc_start.md")]
/// ```
/// fatal!([($predicate: expr)]? $format_string [, $format_args]*)
/// ```
/// Where:
#[doc = include_str!("runtime_asserts_doc_arguments.md")]
#[macro_export]
macro_rules! fatal { ($($anything:tt)*) => { $crate::emlog!(Fatal, $($anything)*) } }

/// Log an error to mGBA emulator log.
#[doc = include_str!("runtime_asserts_doc_start.md")]
/// ```
/// error!([($predicate: expr)]? $format_string [, $format_args]*)
/// ```
/// Where:
#[doc = include_str!("runtime_asserts_doc_arguments.md")]
#[macro_export]
macro_rules! error { ($($anything:tt)*) => { $crate::emlog!(Error, $($anything)*) } }

/// Log a warning to mGBA emulator log.
#[doc = include_str!("runtime_asserts_doc_start.md")]
/// ```
/// warn!([($predicate: expr)]? $format_string [, $format_args]*)
/// ```
/// Where:
#[doc = include_str!("runtime_asserts_doc_arguments.md")]
#[macro_export]
macro_rules! warn { ($($anything:tt)*) => { $crate::emlog!(Warn, $($anything)*) } }

/// Log an info to mGBA emulator log.
#[doc = include_str!("runtime_asserts_doc_start.md")]
/// ```
/// info!([($predicate: expr)]? $format_string [, $format_args]*)
/// ```
/// Where:
#[doc = include_str!("runtime_asserts_doc_arguments.md")]
#[macro_export]
macro_rules! info { ($($anything:tt)*) => { $crate::emlog!(Info, $($anything)*) } }

/// Log a debug message to mGBA emulator log.
#[doc = include_str!("runtime_asserts_doc_start.md")]
/// ```
/// debug!([($predicate: expr)]? $format_string [, $format_args]*)
/// ```
/// Where:
#[doc = include_str!("runtime_asserts_doc_arguments.md")]
#[macro_export]
macro_rules! debug { ($($anything:tt)*) => { $crate::emlog!(Debug, $($anything)*) } }
