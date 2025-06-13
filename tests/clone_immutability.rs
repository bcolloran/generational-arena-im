extern crate typed_generational_arena;
use typed_generational_arena::StandardArena as Arena;

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
