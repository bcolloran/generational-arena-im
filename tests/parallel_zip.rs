extern crate generational_arena_im;
extern crate rayon;
use generational_arena_im::StandardArena as Arena;
use rayon::iter::{
    IndexedParallelIterator,
    IntoParallelRefIterator,
    IntoParallelRefMutIterator,
    ParallelIterator,
};

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

#[test]
fn par_iter_mut_zips() {
    let mut a1_seq = Arena::new();
    let mut a2_seq = Arena::new();
    let mut a1_par = Arena::new();
    let mut a2_par = Arena::new();
    for i in 0..100 {
        a1_seq.insert(i);
        a2_seq.insert(i * 2);
        a1_par.insert(i);
        a2_par.insert(i * 2);
    }

    // sequential mutation
    for ((_, x), (_, y)) in a1_seq.iter_mut().zip(a2_seq.iter_mut()) {
        *x += *y;
        *y += *x;
    }

    // parallel mutation
    a1_par
        .par_iter_mut()
        .zip(a2_par.par_iter_mut())
        .for_each(|((_, x), (_, y))| {
            *x += *y;
            *y += *x;
        });

    let seq1: Vec<_> = a1_seq.iter().map(|(_, v)| *v).collect();
    let seq2: Vec<_> = a2_seq.iter().map(|(_, v)| *v).collect();
    let par1: Vec<_> = a1_par.iter().map(|(_, v)| *v).collect();
    let par2: Vec<_> = a2_par.iter().map(|(_, v)| *v).collect();
    assert_eq!(seq1, par1);
    assert_eq!(seq2, par2);
}
