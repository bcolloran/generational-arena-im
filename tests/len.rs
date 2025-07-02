extern crate generational_arena_im;
extern crate rayon;

use generational_arena_im::StandardArena as Arena;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

#[test]
fn par_iter_len_invariant() {
    let mut arena = Arena::new();
    let a = arena.insert(1);
    let _b = arena.insert(2);
    arena.remove(a);
    // 1 occupied, 1 free

    let v: Vec<_> = arena.par_iter().collect();
    assert_eq!(v.len(), 1);
}

/// Holes in the middle should be just as bad.
#[test]
fn par_iter_panics_when_middle_removed() {
    let mut arena = Arena::new();
    let k0 = arena.insert(0);
    let _k1 = arena.insert(1);
    let k2 = arena.insert(2);
    let _k3 = arena.insert(3);
    // Remove two entries in the middle
    arena.remove(k0);
    arena.remove(k2);

    // Now occupied count = 2, raw slots = 4
    let v: Vec<_> = arena.par_iter().map(|(_, x)| *x).collect();
    assert_eq!(v, vec![1, 3]); // sanity check
}

/// Removing the *first two* of three slots leaves exactly one occupied:
/// par_iter().collect().len() must be 1.
#[test]
fn par_iter_front_removed_len_is_one() {
    let mut arena = Arena::new();
    let k0 = arena.insert("a");
    let k1 = arena.insert("b");
    let _k2 = arena.insert("c");
    arena.remove(k0);
    arena.remove(k1);

    // should collect exactly 1 element
    let v: Vec<_> = arena.par_iter().collect();
    assert_eq!(v.len(), 1, "expected 1 occupied, got {}", v.len());
}

/// Removing a hole in the middle of four slots should leave two occupied:
/// par_iter().collect().len() must be 2.
#[test]
fn par_iter_middle_removed_len_is_two() {
    let mut arena = Arena::new();
    let k0 = arena.insert(0);
    let _k1 = arena.insert(1);
    let k2 = arena.insert(2);
    let _k3 = arena.insert(3);

    // remove one at front and one in the middle
    arena.remove(k0);
    arena.remove(k2);

    // occupied count is 2 out of a raw length of 4
    let v: Vec<_> = arena.par_iter().collect();
    assert_eq!(v.len(), 2, "expected 2 occupied, got {}", v.len());
}
