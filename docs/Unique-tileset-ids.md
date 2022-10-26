The assets are embed in the binary with include_bytes and defined as const,
so I can't just compare the reference.

Ideally, this would be in the form of a macro that expands to a value that can
be compared, and each invocation of this macro would expand to a unique value.

Now that I lay down the question, I guess a see a few options. Notably:

× Make my assets static rather than const, although at the likely cost of a few
  optimizations
× Create a dummy ZST static in the macro and use (though I don't know if ZST
  static references are guaranteed to be unique in rust)
× Create a dummy type in the macro and call core::any::TypeId on it (although
  TypeId is u64 and it's kinda a waste on the 32-bit GBA processor, though I
  don't expect to do a sprite ID comparison more than once every few thousand
  frames)

I just read this and the last paragraph made me consider whether I should
make the assets static
<https://doc.rust-lang.org/reference/items/static-items.html>

Problem is that if we rely on the user defining their assets as `static`, we
have now a huge footgun in the API. Although maybe it's worth considering, since
`const`ing might result in way too much data duplication.

## `TypeId`

Let's try with a `TypeId`:

```rust
#![no_std]
#![feature(const_type_id)]

use core::any::TypeId;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
struct UniqueId(TypeId);
impl UniqueId {
  #[doc(hidden)]
  pub const fn new<T: 'static>() -> Self {
    Self(TypeId::of::<T>())
  }
}

macro_rules! unique_id {
  () => {{
    enum Boo {}
    UniqueId::new::<Boo>()
  }}
}

const FOO: UniqueId = unique_id!();
const BAR: UniqueId = unique_id!();
const BAZ: UniqueId = unique_id!();
const QUX: UniqueId = unique_id!();
const CUD: UniqueId = unique_id!();

fn main() {
  assert_eq!(FOO, FOO);
  assert_ne!(FOO, BAR);
  assert_ne!(FOO, BAZ);
  assert_ne!(FOO, QUX);
  assert_ne!(FOO, CUD);
  assert_eq!(BAR, BAR);
  assert_ne!(BAR, BAZ);
  assert_ne!(BAR, QUX);
  assert_ne!(BAR, CUD);
  assert_eq!(BAZ, BAZ);
  assert_ne!(BAZ, QUX);
  assert_ne!(BAZ, CUD);
  assert_eq!(QUX, QUX);
  assert_ne!(QUX, CUD);
  assert_eq!(CUD, CUD);
}
```

This works but requires nightly (`#![feature(const_type_id)]`).
Since `haldvance` already requires nightly, it's probably fine.

## `static` reference

`static` cannot be refered in `const` context, meaning it's impossible to define
assets as `const`. This has advantages as mentioned in the `static` rust reference,
but it does restrict what we can do.

This would require the `tileset` macro to "force" prevent `const`-ing assets, so
that the API can use the reference as a comparison.

## Future

Honestly, we should at least consider the `static` solution, as it doesn't require
any feature, and seems to be the "accepted" solution, but right now I'll content
myself with the `TypeId`-based solution.
