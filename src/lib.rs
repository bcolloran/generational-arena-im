/*!
[![](https://docs.rs/typed-generational-arena/badge.svg)](https://docs.rs/typed-generational-arena/)
[![](https://img.shields.io/crates/v/typed-generational-arena.svg)](https://crates.io/crates/typed-generational-arena)
[![](https://img.shields.io/crates/d/typed-generational-arena.svg)](https://crates.io/crates/typed-generational-arena)

A safe arena allocator that allows deletion without suffering from [the ABA
problem](https://en.wikipedia.org/wiki/ABA_problem) by using generational type-safe
indices. Forked from [generational-arena](https://github.com/fitzgen/generational-arena/).

Inspired by [Catherine West's closing keynote at RustConf
2018](http://rustconf.com/program.html#closingkeynote), where these ideas
were presented in the context of an Entity-Component-System for games
programming.

## What? Why?

Imagine you are working with a graph and you want to add and delete individual
nodes at a time, or you are writing a game and its world consists of many
inter-referencing objects with dynamic lifetimes that depend on user
input. These are situations where matching Rust's ownership and lifetime rules
can get tricky.

It doesn't make sense to use shared ownership with interior mutability (ie
`Rc<RefCell<T>>` or `Arc<Mutex<T>>`) nor borrowed references (ie `&'a T` or `&'a
mut T`) for structures. The cycles rule out reference counted types, and the
required shared mutability rules out borrows. Furthermore, lifetimes are dynamic
and don't follow the borrowed-data-outlives-the-borrower discipline.

In these situations, it is tempting to store objects in a `Vec<T>` and have them
reference each other via their indices. No more borrow checker or ownership
problems! Often, this solution is good enough.

However, now we can't delete individual items from that `Vec<T>` when we no
longer need them, because we end up either

* messing up the indices of every element that follows the deleted one, or

* suffering from the [ABA
  problem](https://en.wikipedia.org/wiki/ABA_problem). To elaborate further, if
  we tried to replace the `Vec<T>` with a `Vec<Option<T>>`, and delete an
  element by setting it to `None`, then we create the possibility for this buggy
  sequence:

    * `obj1` references `obj2` at index `i`

    * someone else deletes `obj2` from index `i`, setting that element to `None`

    * a third thing allocates `obj3`, which ends up at index `i`, because the
      element at that index is `None` and therefore available for allocation

    * `obj1` attempts to get `obj2` at index `i`, but incorrectly is given
      `obj3`, when instead the get should fail.

By introducing a monotonically increasing generation counter to the collection,
associating each element in the collection with the generation when it was
inserted, and getting elements from the collection with the *pair* of index and
the generation at the time when the element was inserted, then we can solve the
aforementioned ABA problem. When indexing into the collection, if the index
pair's generation does not match the generation of the element at that index,
then the operation fails.

## Features

* Zero `unsafe`
* Well tested, including quickchecks
* `no_std` compatibility
* All the trait implementations you expect: `IntoIterator`, `FromIterator`,
  `Extend`, etc...

## Usage

First, add `typed-generational-arena` to your `Cargo.toml`:

```toml
[dependencies]
typed-generational-arena = "0.2"
```

Then, import the crate and use one of the variations of the
[`generational_arena_im::Arena`](./struct.Arena.html) type!
In these examples, we use `generational_arena_im::StandardArena`,
but you can use any combination of index and generation ID
best fits your purposes.

```rust
extern crate generational_arena_im;
use generational_arena_im::StandardArena;

let mut arena = StandardArena::new();

// Insert some elements into the arena.
let rza = arena.insert("Robert Fitzgerald Diggs");
let gza = arena.insert("Gary Grice");
let bill = arena.insert("Bill Gates");

// Inserted elements can be accessed infallibly via indexing (and missing
// entries will panic).
assert_eq!(arena[rza], "Robert Fitzgerald Diggs");

// Alternatively, the `get` and `get_mut` methods provide fallible lookup.
if let Some(genius) = arena.get(gza) {
    println!("The gza gza genius: {}", genius);
}
if let Some(val) = arena.get_mut(bill) {
    *val = "Bill Gates doesn't belong in this set...";
}

// We can remove elements.
arena.remove(bill);

// Insert a new one.
let murray = arena.insert("Bill Murray");

// The arena does not contain `bill` anymore, but it does contain `murray`, even
// though they are almost certainly at the same index within the arena in
// practice. Ambiguities are resolved with an associated generation tag.
assert!(!arena.contains(bill));
assert!(arena.contains(murray));

// Iterate over everything inside the arena.
for (idx, value) in &arena {
    println!("{:?} is at {:?}", value, idx);
}
```

### Snapshot immutability

Cloning an arena is an `O(1)` operation that returns an immutable snapshot.
Further mutations to the original arena do not affect snapshots.

```rust
use generational_arena_im::StandardArena;

let mut arena = StandardArena::new();
let idx = arena.insert(1);
let snapshot = arena.clone();

*arena.get_mut(idx).unwrap() = 2;

assert_eq!(snapshot[idx], 1);
```

## `no_std`

To enable `no_std` compatibility, disable the on-by-default "std" feature. This
currently requires nightly Rust and `feature(alloc)` to get access to `Vec`.

```toml
[dependencies]
typed-generational-arena = { version = "0.2", default-features = false }
```

 */

#![forbid(unsafe_code, missing_docs, missing_debug_implementations)]
#![no_std]
#![cfg_attr(not(feature = "std"), feature(alloc))]

extern crate nonzero_ext;
extern crate num_traits;
#[macro_use]
extern crate cfg_if;
extern crate im;
extern crate rayon;

cfg_if! {
    if #[cfg(feature = "std")] {
        extern crate std;
    } else {
        extern crate alloc;
    }
}

mod presets;

pub use presets::*;

mod arena;
mod generation;
mod index;

pub use arena::{Arena, Drain, IntoIter, Iter, IterMut};
pub use generation::{
    DisableRemoval, FixedGenerationalIndex, GenerationalIndex, IgnoreGeneration, IgnoredGeneration,
    NonzeroGeneration, NonzeroWrapGeneration,
};
pub use index::{ArenaIndex, Index, NonZeroIndex};
