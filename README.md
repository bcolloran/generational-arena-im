# `generational-arena-im`

A safe arena allocator that allows deletion without suffering from [the ABA
problem](https://en.wikipedia.org/wiki/ABA_problem) by using generational type-safe
indices. Forked from https://gitlab.com/tekne/typed-generational-arena.

Immutable via Im.

## Snapshot immutability

Cloning an arena is cheap and returns an immutable snapshot. Mutating the
original arena afterwards does not change existing snapshots.

```rust
use generational_arena_im::StandardArena;

let mut arena = StandardArena::new();
let idx = arena.insert(1);
let snapshot = arena.clone();

*arena.get_mut(idx).unwrap() = 2;

assert_eq!(snapshot[idx], 1);
```

