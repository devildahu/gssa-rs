I've came up with something based on [this discussion]. It's two macros:

1. `include_aligned!`: an `include_bytes!` equivalent with proper alignment
   returning `[u8;_]`
2. `include_type!`: an `include_bytes!` with a little pointer casting (unsafe)
   that compile-time panics if the included file doesn't have a byte count
   multiple of the cast target size.

**Minimally supported rust version (MSRV)**: Since the code is conditionally
added to the final compiled artifact, each macro has a different MSRV:

* `include_aligned` should be available on any rust version that supports macro
  by example.
* `include_type` stable since 1.64.0

The code is licensed under Zlib OR Apache2.0. See [the source file].

[the source file]: https://gitlab.com/nicopap/gssa-rs/-/blob/b225be31abc710e644d2f0f4ab0a14405dc0e9b2/include_const_aligned/src/lib.rs
[this discussion]: https://users.rust-lang.org/t/can-i-conveniently-compile-bytes-into-a-rust-program-with-a-specific-alignment/24049/2