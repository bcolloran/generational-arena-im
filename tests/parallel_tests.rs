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
fn into_par_iter_mut_enumerate_updates() {
    let mut arena = Arena::new();
    for _i in 0..578 {
        arena.insert(0i16);
    }

    arena
        .par_iter_mut()
        .enumerate()
        .for_each(|(i, (_idx, v))| *v = i as i16);

    let values: Vec<_> = arena.iter().map(|(_, v)| *v).collect();
    assert_eq!(values, (0..578i16).collect::<Vec<_>>());
}
