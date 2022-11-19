## How to manage the life cycle of objects

Where "object" here means: bullets, enemies, item pickups and potentially
other.

### Problem

When an object is spawned, it requires initializing data in the "graphics" phase.
When it "dies" it needs a similar treatment.

Also storing objects in an `ArrayVec` causes additional overhead, because each
frame copy the whole vec and rebuild it after trimming the objects marked for
deletion (which are tracked using a bitmap)

I should use the `coca::SlotMap` crate instead.

Object:
- object slot.
- sprite (because that's why you'd want an object slot)
- setup: It's always going to be fn(something: What?, &mut object::Handle)
- update: Require access to rest of world
- maybe visual update
- cleanup

Central "callback registry" where I keep track of what to do (fn pointer) per
slot?

* But what about additional state such as Bullet sprite sheet?
* Not possible, at all time the object struct must be owned by the end-user,
  because no heap = no Box, hence no type-agnostic storage
  * Doubt: Possible to do type-agnostic storage with `TypeId`.
  
Maybe can define an interface on GameState to let haldvance::exec iterate
itself all the objects?

Following is **BAD**. Because it doens't make sense:

* `Live` trait still requires user to manage life state, it's just more noisy
* Running over an Iterator of function pointers is equivalent to letting user
  just run some code!

```rust
enum LifeStage {
  Born,
  Living,
  Dead,
}
trait Live {
  fn status(&self) -> LifeStage;
  fn slot(&self) -> object::Slot;
}
fn object_list(&mut self) -> impl Iterator<Item = &dyn Live>{}
// FnOnce not possible because FnOnce not dyn. (or maybe this is an exception?)
fn object_list(&mut self) -> impl Iterator<Item = &dyn Fn(&mut video::Control)> {}
struct DeadObject {
  slot: object::Slot,
}
```

Issue is that *only* GameState is allowed to store state, therefore know what was
created and deleted.
IMPLIES burden is on user to _manage_ this state.
BUT can provide tools to lessen that burden. (eg: generic structs that implement
general live state management)

**What about `bevy_ptr`?**

PROBLEM: I want to avoid allocation at all cost! This means I have to have
fixed-size buffers at least somewhere. `bevy_ptr` might be helpful if I opt to
do generalize over storage/implement an ECS framework for the GBA.


**Shared behavior**

ECS tells you this: it is useful to reason about behavior independently of
specific entity.


**ECS?**

What if I wrote an ECS framework for the GBA?

* Seems feasable, but I don't want to spend the time doing it
* Kyren's talk: <https://kyren.github.io/2018/09/14/rustconf-talk.html>
* Hecs: <https://github.com/ralith/hecs>

```rust
struct Archetype<const ENTS: usize, const COMPS: usize> {
  components: Bitset<COMPS>,
  entities: Bitset<ENTS>,
}
trait Query<const COMPS: usize> {
  const COMPONENT_INDICES: Bitset<COMPS>;
  type CursorItem: (impl Component, ..);
}
struct Cursor<'a, const ENTS: usize, const COMPS: usize, variable(COMPS) WC: Component> {
  world: *mut World<ENTS, COMPS, WC>,
  _world_life: PhantomData<&'a ()>,
  entities: Bitset<ENTS>,
}
impl<'a, const ENTS: usize, const COMPS: usize, variable(COMPS) WC: Component>
  Cursor<'a, ENTS, COMPS, WC>
{
  fn iter<C>(&self) -> impl Iterator<Item=&C>
  where
    C: (impl Component, ..),
  {
    todo!()
  }
}
struct World<const ENTS: usize, const COMPS: usize, variable(COMPS) T: Component> {
  /// Component data table, each row contain values, each column is a different
  /// component. Due to `MaybeUninit`, the content may be uninitialized.
  table: ([MaybeUninit<T>; ENTS], ..COMPS),
  /// Tells which component is held by which entity.
  occupancy: [Bitset<ENTS>; COMPS],
  cached_archetypes: Vec<Archetype<ENTS, COMPS>>,
}
impl<const ENTS: usize, const COMPS: usize, variable(COMPS) T: Component> World<ENTS, COMPS, T> {
  fn new() -> Self {
    Self {
      table: ([MaybeUninit::uninit(); ENTS], ..COMPS),
      occupancy: [Bitset::empty(); COMPS],
      cached_archetypes: Vec::new(),
    }
  }
  fn extract<Q: Query<COMPS>>(&mut self) -> Cursor<'_, ENTS, COMPS, Query::CursorItem, T> {
    let components: Bitset<COMPS> = Q::COMPONENT_INDICES;
    let is_requested = |arch: &Archetype| arch.components == components;
    let entities = match self.cached_archetypes.iter().find(is_requested) {
      Some(precomputed) => precomputed.entities,
      None => {
        let entities = components.limit_to_enabled(&self.occupancy).fold(
          Bitset::<ENTS>::empty(),
          |acc, ents| *acc = *acc & ents,
        );
        let archetype = Archetype { components, entities };
        self.cached_archetypes.push(archetype);
        entities
      }
    }
    Cursor {
      world: ptr::from(self),
      _world_life: PhatonData,
      entities,
    }
  }
  fn run_system(&mut self, sys: impl FnMut(impl Query,..)) {
    sys(self.extract(), ..)
  }
}
```

meh XD

## Conclusion

1. Create a "lifetimed collection" where I can track the lifetime state of items
   in the collection.
