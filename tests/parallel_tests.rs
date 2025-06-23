extern crate generational_arena_im;
extern crate rayon;
use generational_arena_im::StandardArena as Arena;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator,
    IntoParallelRefMutIterator, ParallelIterator,
};

#[test]
fn par_iter_matches_sequential() {
    let mut arena = Arena::new();
    for i in 0..100 {
        arena.insert(i);
    }

    let seq: Vec<_> = arena.iter().map(|(_, v)| *v).collect();
    let par: Vec<_> = arena.par_iter().map(|(_, v)| *v).collect();

    assert_eq!(seq, par);
}

#[test]
fn into_par_iter_matches_sequential() {
    let mut arena = Arena::new();
    for i in 0..100 {
        arena.insert(i);
    }
    let seq: Vec<_> = arena.iter().map(|(_, v)| *v).collect();
    let par: Vec<_> = arena.into_par_iter().map(|(_, v)| *v).collect();

    assert_eq!(seq, par);
}

#[test]
fn par_iter_mut_updates() {
    let mut arena = Arena::new();
    for i in 0..100 {
        arena.insert(i);
    }
    arena.par_iter_mut().for_each(|(_, v)| *v *= 2);
    let values: Vec<_> = arena.iter().map(|(_, v)| *v).collect();
    assert_eq!(values, (0..100).map(|x| x * 2).collect::<Vec<_>>());
}

#[test]
fn into_par_iter_mut_updates() {
    let mut arena = Arena::new();
    for i in 0..100 {
        arena.insert(i);
    }
    (&mut arena).into_par_iter().for_each(|(_, v)| *v *= 2);
    let values: Vec<_> = arena.iter().map(|(_, v)| *v).collect();
    assert_eq!(values, (0..100).map(|x| x * 2).collect::<Vec<_>>());
}

#[test]
fn par_iter_zips() {
    let mut a1 = Arena::new();
    let mut a2 = Arena::new();
    for i in 0..100 {
        a1.insert(i);
        a2.insert(i * 2);
    }

    let seq: Vec<_> = a1
        .iter()
        .zip(a2.iter())
        .map(|((_, x), (_, y))| *x + *y)
        .collect();
    let par: Vec<_> = a1
        .par_iter()
        .zip(a2.par_iter())
        .map(|((_, x), (_, y))| *x + *y)
        .collect();

    assert_eq!(seq, par);
}
