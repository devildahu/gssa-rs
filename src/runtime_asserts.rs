/// Print a mGBA or no$ emulator warning if the "runtime_asserts" feature is enabled.
#[macro_export]
macro_rules! warn {
    (($cond:expr) $fmt:literal, $($fmt_args:tt)*) => {
        #[cfg(feature = "runtime_asserts")]
        if $cond {
            use core::fmt::Write;
            let mut out = gba::debugging::mgba::MGBADebug::new().unwrap();
            let _ = out.write_str(concat!("[", file!(), ":", line!(), "]"));
            let _ = write!(&mut out, $fmt , $($fmt_args)* );
            out.send(gba::debugging::mgba::MGBADebugLevel::Warning);
        }
    };
    ( $fmt:literal, $($fmt_args:tt)*) => {
        warn!((true) $fmt, $($fmt_args)*)
    };
    ( $fmt:literal) => {
        warn!((true) $fmt ,)
    };
}
