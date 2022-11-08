I've the following code:

```rust
// (1)
fn write_slice_at_offset(self, offset: usize, slice: &[T]) {
    let iter = self.iter().skip(offset).zip(slice.iter());
    iter.for_each(|(addr, value)| addr.write(*value));
}
// (2)
fn write_slice_at_offset(self, offset: usize, slice: &[T]) {
    for (i, elem) in slice.iter().enumerate() {
        if let Some(addr) = self.get(i + offset) {
            addr.write(*elem);
        } else {
            break;
        }
    }
}
```

I call it on a `const` value, where `T` is:

```rust
#[repr(transparent)]
#[derive(Clone, Copy)]
struct Entry(u16);

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Tile([Entry; 32]);
```

So fine, the two functions generate different assembly. But (1) actually skips
the first entry in `slice`, am I missing something?

Digging into the source of `voladdress`, it turns out the `nth` method on the
`VolBlock` iterator is not correctly implemented. It is victim of the
off-by-one ~~horror~~ error.