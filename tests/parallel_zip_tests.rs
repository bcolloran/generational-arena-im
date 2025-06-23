extern crate generational_arena_im;
extern crate rayon;
use generational_arena_im::StandardArena as Arena;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};

#[test]
fn par_zip_test() {
    let mut arena_a = Arena::new();
    let mut arena_b = Arena::new();

    for i in 0..100 {
        arena_a.insert(i);
        arena_b.insert(i * 2);
    }

    let seq: Vec<_> = (0..100).into_iter().map(|v| v * 3).collect();

    let par: Vec<_> = arena_a
        .par_iter()
        .zip(arena_b.par_iter())
        .map(|((_, a), (_, b))| a + b)
        .collect();

    assert_eq!(seq, par);
}

#[test]
fn par_mut_zip_test() {
    let mut arena_a = Arena::new();
    let mut arena_b = Arena::new();

    for i in 0..100 {
        arena_a.insert(0);
        arena_b.insert(i);
    }

    arena_a
        .par_iter_mut()
        .zip(arena_b.par_iter())
        .for_each(|((_, a), (_, b))| *a = *b);

    let par: Vec<_> = arena_a.par_iter().map(|(_, a)| *a).collect();

    let seq: Vec<_> = (0..100u32).into_iter().collect();
    assert_eq!(seq, par);
}
