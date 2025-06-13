extern crate typed_generational_arena;
use typed_generational_arena::StandardArena as Arena;

#[test]
fn cloned_arenas_are_independent() {
    let mut arena = Arena::new();
    let a = arena.insert(1);
    let b = arena.insert(2);
    let c = arena.insert(3);

    let mut snapshot = arena.clone();

    // Mutate the original arena
    *arena.get_mut(b).unwrap() = 20;
    arena.remove(c);
    let d = arena.insert(4);

    // Mutate the cloned snapshot differently
    *snapshot.get_mut(a).unwrap() = 10;
    snapshot.remove(b);
    let e = snapshot.insert(5);

    // Verify each arena only reflects its own changes
    assert_eq!(arena.get(a), Some(&1));
    assert_eq!(arena.get(b), Some(&20));
    assert!(arena.get(c).is_none());
    assert_eq!(arena.get(d), Some(&4));
    assert!(arena.get(e).is_none());

    assert_eq!(snapshot.get(a), Some(&10));
    assert!(snapshot.get(b).is_none());
    assert_eq!(snapshot.get(c), Some(&3));
    assert_eq!(snapshot.get(e), Some(&5));
    assert!(snapshot.get(d).is_none());
}
