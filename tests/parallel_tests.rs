extern crate typed_generational_arena;
extern crate rayon;
use typed_generational_arena::StandardArena as Arena;
use rayon::iter::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};

#[test]
fn par_iter_matches_sequential() {
    let mut arena = Arena::new();
    for i in 0..100 {
        arena.insert(i);
    }
    let mut seq: Vec<_> = arena.iter().map(|(_, v)| *v).collect();
    let mut par: Vec<_> = (&arena).par_iter().map(|(_, v)| *v).collect();
    seq.sort();
    par.sort();
    assert_eq!(seq, par);
}

#[test]
fn par_iter_mut_updates() {
    let mut arena = Arena::new();
    for i in 0..100 {
        arena.insert(i);
    }
    (&mut arena).par_iter_mut().for_each(|(_, v)| *v *= 2);
    let mut values: Vec<_> = arena.iter().map(|(_, v)| *v).collect();
    values.sort();
    assert_eq!(values, (0..100).map(|x| x * 2).collect::<Vec<_>>());
}
