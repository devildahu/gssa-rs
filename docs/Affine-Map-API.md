How do I want the affine map API to work?

Problems:

- `Pos` != position in memory.
- Double byte memory bus
- `Drawable` API divergence.

What I want:

- I want it to be efficient!
- I want the API to be convinient, ideally unified.

The fundamental incompatibility: efficiency for `AffineHandle` **requires** the
`set_tiles` method to drive the control flow (more or less).

But efficiency in general would advise that it's the struct-specific code that
decides the order in which tiles are drawn.

Possible solutions:

1. Ask for `Rect` and `AffineTiles` (where `AffineTiles` is a 2-align u8 slice)
2. Ask for `Rect` and a method that returns a value based on rect position
3. Per-line callback accepting an `Iterator`.


## API drafts

### Original API

#### Problem

Random access, resulting in much less efficient code, also requires making no
assumption in tile pairing, resulting in each tile write turning into a load+store

```rust
pub trait Drawable {
    fn for_each_tile<F: FnMut(Tile, Pos)>(&self, f: F);
}
// When writting to video memory:
pub fn set_tiles(&mut self, drawable: &impl Drawable) {
    drawable.for_each_tile(|tile, pos| { self.set_tile(tile, pos) });
```

### Idea nº1, slices

Ask for `Rect` and `AffineTiles` (where `AffineTiles` is a 2-align u8 slice)

#### Problem

Slices require dynamically allocated arrays. At first this seems to require
heap allocation (`Vec`), but I think it's possible to get away with one of
`tinyvec`, `staticvec`, `array-bytes`, `coca`, `partial-array`, `stackvec`, `arrayvec`, etc.

Also 2-align ends up being counterproductive if the starting memory position is odd,
or the width is odd etc.

We might be able to `slice::chunks` followed by `slice::align_to`.

```rust
pub trait Drawable {
    fn region(&self) -> Rect;
    fn tiles(&self) -> &[Tile];
}
// When writting to video memory:
pub fn set_tiles(&mut self, drawable: &impl Drawable) {
    let region = drawable.region();
    let tiles = drawable.tiles()
    for y in 0..region.height {
        for x in 0..region.width {
            let index = x + y * region.height;
            gba_hardware.draw_tile_at(tiles[index], Pos { x, y } + at);
```

An alternative would be to keep the callback-based method and split the region
in multiple lines:

```rust
pub trait Drawable {
    fn for_each_line<F: FnMut(Pos, &[Tile])>(&self, f: F);
}
// When writting to video memory:
pub fn set_tiles(&mut self, drawable: &impl Drawable) {
    let mut x = 0;
    drawable.for_each_line(|pos, tiles| {
        tiles.for_each(|tile| { gba_hardware.draw_tile_at(tile, pos + Pos { x, y:0 }) });
        x += 1;
    });
```

### Idea nº2

Ask for `Rect` and a method that returns a value based on rect position

#### Problem

Possibly less efficient, since the `Drawable` now is _driven_ by generic code,
which doesn't know how to best iterate through specific structs.

For example, to determine `region` for `Drawable for &'s str`, I'd need to
first iterate through it to count line endings and max line length, return
a `Rect`, then for each tile, just (skipping for lines shorter than max line
length too)

```rust
pub trait Drawable {
    fn region(&self) -> Rect;
    fn tile_at(&self, pos: Pos) -> Tile;
}
// When writting to video memory:
fn set_tiles(&mut self, at: Pos, drawable: &impl Drawable) {
    let region = drawable.region();
    for y in 0..region.height {
        for x in 0..region.width {
            let pos = Pos { x, y };
            gba_hardware.draw_tile_at(drawable.tile_at(pos), pos + at);
```

### Idea nº3, with `Iterator`

The line-based alternative in nº1 seems to solve the most pressing issue with
the affine tile maps.
But taking a slice as argument adds an absolutely unneeded intermediate step.
We know how to solve this: iterators \o/.

The API becomes a bit convoluted, and implementing it might be impossible (?)
but this is worth a try.

#### Problem

The `Iter` type is difficult to express nicely, and might reify callbacks into
function pointers, making some optimizations impossible.

```rust
pub trait Drawable {
    type Iter;

    fn for_each_line<F: FnMut(Pos, Self::Iter)>(&self, f: F);
}
// When writting to video memory:
pub fn set_tiles(&mut self, drawable: &impl Drawable) {
    drawable.for_each_line(|pos, tiles| {
        tiles.for_each(|tile| { gba_hardware.draw_tile_at(tile, pos) });
    });
```
