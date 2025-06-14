extern crate typed_generational_arena;
use typed_generational_arena::StandardArena as Arena;

// Cloning should freeze a snapshot so later modifications do not affect it.
#[test]
fn snapshots_are_immutable_after_clone() {
    let mut arena = Arena::new();
    let snap0 = arena.clone();

    let idx = arena.insert(1);
    let snap1 = arena.clone();

    *arena.get_mut(idx).unwrap() = 2;
    let snap2 = arena.clone();

    assert_eq!(snap0.len(), 0);
    assert_eq!(snap1[idx], 1);
    assert_eq!(snap2[idx], 2);

    arena.remove(idx);
    assert!(arena.get(idx).is_none());
    assert_eq!(snap1[idx], 1);
    assert_eq!(snap2[idx], 2);
}
// Snapshots should remain consistent through multiple arena changes.

#[test]
fn snapshots_survive_multiple_modifications() {
    let mut arena = Arena::new();
    let first = arena.insert(10);
    let snap_insert = arena.clone();

    *arena.get_mut(first).unwrap() = 11;
    let snap_update = arena.clone();

    arena.remove(first);
    let snap_remove = arena.clone();

    assert_eq!(snap_insert[first], 10);
    assert_eq!(snap_update[first], 11);
    assert!(snap_remove.get(first).is_none());
}
// Verify snapshots remain valid after retain and drain operations.
#[test]
fn snapshots_after_retain_and_drain() {
    let mut arena = Arena::new();
    let a = arena.insert(1);
    let b = arena.insert(2);
    let c = arena.insert(3);

    let snap_full = arena.clone();

    arena.retain(|_, v| *v % 2 == 0);
    let snap_retained = arena.clone();

    for _ in arena.drain() {}

    assert_eq!(snap_full[a], 1);
    assert_eq!(snap_full[b], 2);
    assert_eq!(snap_full[c], 3);

    assert!(snap_retained.get(a).is_none());
    assert_eq!(snap_retained[b], 2);
    assert!(snap_retained.get(c).is_none());

    assert_eq!(arena.len(), 0);
    assert!(arena.get(a).is_none());
    assert!(arena.get(b).is_none());
    assert!(arena.get(c).is_none());
}
