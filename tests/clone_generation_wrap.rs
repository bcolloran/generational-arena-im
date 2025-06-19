extern crate generational_arena_im;

use generational_arena_im::{TinyWrapArena, TinyWrapIndex};

#[test]
fn clone_insert_remove_until_wrap() {
    const WRAP_LIMIT: usize = (std::u16::MAX as usize) + 10;
    let mut arena = TinyWrapArena::<usize>::with_capacity(1);
    let mut snaps: Vec<(TinyWrapArena<usize>, Vec<(TinyWrapIndex<usize>, usize)>)> = Vec::new();

    // Insert enough elements to force capacity growth
    let a = arena.insert(0);
    snaps.push((arena.clone(), vec![(a, 0)]));
    let b = arena.insert(1);
    snaps.push((arena.clone(), vec![(a, 0), (b, 1)]));
    let c = arena.insert(2);
    snaps.push((arena.clone(), vec![(a, 0), (b, 1), (c, 2)]));
    assert!(arena.capacity() > 1);

    // Remove all and record snapshots
    assert_eq!(arena.remove(a), Some(0));
    snaps.push((arena.clone(), vec![(b, 1), (c, 2)]));
    assert_eq!(arena.remove(b), Some(1));
    snaps.push((arena.clone(), vec![(c, 2)]));
    assert_eq!(arena.remove(c), Some(2));
    snaps.push((arena.clone(), vec![]));
    assert_eq!(arena.len(), 0);
    let mut last_cap = arena.capacity();

    // Repeat insert/remove while cloning until generations wrap
    for i in 0..WRAP_LIMIT {
        let idx = arena.insert(i);
        assert_eq!(arena.len(), 1);
        assert!(arena.capacity() >= last_cap);
        snaps.push((arena.clone(), vec![(idx, i)]));
        assert_eq!(arena.remove(idx), Some(i));
        assert_eq!(arena.len(), 0);
        snaps.push((arena.clone(), vec![]));
        last_cap = arena.capacity();
    }

    // Validate all snapshots retained their data
    for (snap, expected) in snaps {
        assert_eq!(snap.len(), expected.len());
        for (idx, val) in expected {
            assert_eq!(snap.get(idx).copied(), Some(val));
        }
    }
}
