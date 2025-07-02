extern crate generational_arena_im;
extern crate rayon;
use generational_arena_im::StandardArena as Arena;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator,
    IntoParallelRefMutIterator, ParallelIterator,
};

#[test]
fn par_zip_test() {
    let mut arena_a = Arena::new();
    let mut arena_b = Arena::new();

    for i in 0..100 {
        arena_a.insert(i);
        arena_b.insert(i * 2);
    }

    let seq: Vec<_> = (0..100u32).into_iter().map(|v| v * 3).collect();

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

    let seq: Vec<_> = (0..100i32).into_iter().map(|v| v).collect();

    arena_a
        .par_iter_mut()
        .zip(arena_b.par_iter())
        .for_each(|((_, a), (_, b))| {
            // eprintln!(
            //     "Updating: ({} <- {}) on thread {:?}",
            //     *a,
            //     *b,
            //     rayon::current_thread_index()
            // );
            *a = *b
        });

    let par: Vec<_> = arena_a.iter().map(|(_, a)| *a).collect();

    assert_eq!(seq, par);
}

#[test]
fn test_multiple_arena_operations() {
    let mut arena_a = Arena::new();
    let mut arena_b = Arena::new();

    for i in 0..100 {
        arena_a.insert(Vec::new());
        arena_b.insert(i);
    }

    arena_a
        .par_iter_mut()
        .zip(arena_b.par_iter())
        .for_each(|((_, a), (_, b))| {
            let mut v = Vec::new();
            for j in 0..*b {
                v.push(j);
            }
            *a = v;
        });

    let vec: Vec<_> = arena_a
        .par_iter()
        .flat_map(|(_, v)| v.into_par_iter())
        .map(|v| v * 2)
        .collect();

    assert_eq!(vec.len(), 4950); // Sum of first 100 natural numbers multiplied by 2
}

/// Zipping two arenas where one has holes should still produce exactly
/// N pairs (no more, no fewer).
#[test]
fn par_iter_zip_with_holes_len_matches() {
    let mut arena_1 = Arena::new();
    let mut arena_2 = Arena::new();

    // A: insert 6, remove 2 in the middle → 4 occupied
    let a0 = arena_1.insert(10);
    let _a1 = arena_1.insert(11);
    let a2 = arena_1.insert(12);
    let _a3 = arena_1.insert(13);
    let _a4 = arena_1.insert(14);
    let _a5 = arena_1.insert(15);
    arena_1.remove(a2);
    arena_1.remove(a0);

    // B: 4 inserts, no removes
    for i in 0..4 {
        arena_2.insert(i);
    }

    // zip should give exactly 4 pairs
    let pairs: Vec<_> = arena_1.par_iter().zip(arena_2.par_iter()).collect();

    assert_eq!(
        pairs.len(),
        4,
        "expected 4 zipped entries, got {}",
        pairs.len()
    );
}

/// par_iter_mut + for_each should touch exactly the occupied slots, no more.
#[test]
fn par_iter_mut_with_holes_updates_correct_count() {
    let mut arena = Arena::new();
    let k0 = arena.insert(100);
    let _k1 = arena.insert(200);
    let k2 = arena.insert(300);
    // now 3 slots, 2 occupied after removals:
    arena.remove(k0);
    arena.remove(k2);

    // double every occupied slot
    arena.par_iter_mut().for_each(|(_, v)| *v *= 2);

    // collect sequentially and confirm length == 1 (only the middle slot was occupied)
    let seq: Vec<_> = arena.iter().map(|(_, v)| *v).collect();
    assert_eq!(seq.len(), 1, "expected 1 updated value, got {}", seq.len());
}

#[test]
fn par_mut_zip_test_with_removals() {
    let mut arena_a = Arena::new();
    let mut arena_b = Arena::new();

    for i in 0..100 {
        let a1 = arena_a.insert(0);
        let a2 = arena_a.insert(1);
        let _a3 = arena_a.insert(2);
        let b1 = arena_b.insert(i);
        let b2 = arena_b.insert(i * 2);
        let _b3 = arena_b.insert(i * 3);
        arena_a.remove(a1);
        arena_a.remove(a2);
        arena_b.remove(b1);
        arena_b.remove(b2);
    }

    let seq: Vec<_> = (0..100i32).into_iter().map(|v| v * 3).collect();

    arena_a
        .par_iter_mut()
        .zip(arena_b.par_iter())
        .for_each(|((_, a), (_, b))| *a = *b);

    let par: Vec<_> = arena_a.iter().map(|(_, a)| *a).collect();

    assert_eq!(seq, par);
}

#[test]
fn par_into_iter_test_with_removals() {
    let mut arena_a = Arena::new();
    let mut arena_b = Arena::new();
    let mut arena_c = Arena::new();

    const N: usize = 30_000;
    for i in 0..N {
        let a1 = arena_a.insert(0);
        let a2 = arena_a.insert(1);
        let _a3 = arena_a.insert(2);
        arena_a.remove(a1);
        arena_a.remove(a2);

        let b1 = arena_b.insert(i);
        let b2 = arena_b.insert(i * 2);
        let _b3 = arena_b.insert(i * 3);
        arena_b.remove(b1);
        arena_b.remove(b2);

        let c1 = arena_c.insert(i + 1);
        let c2 = arena_c.insert(i + 2);
        let _c3 = arena_c.insert(i + 3);
        arena_c.remove(c1);
        arena_c.remove(c2);
    }

    (
        arena_a.par_iter_mut(),
        arena_b.par_iter(),
        arena_c.par_iter(),
    )
        .into_par_iter()
        .for_each(|((_, a), (_, b), (_, c))| *a = *b + *c);

    let par: Vec<_> = arena_a.iter().map(|(_, a)| *a).collect();

    let seq: Vec<_> = (0..N).into_iter().map(|x| (x * 3) + (x + 3)).collect();

    assert_eq!(seq, par);
}

#[test]
fn par_into_iter_test_with_unbalanced_removals() {
    let mut arena_a = Arena::new();
    let mut arena_b = Arena::new();
    let mut arena_c = Arena::new();

    const N: usize = 30_000; // large enough to force splitting
    for i in 0..N {
        let a1 = arena_a.insert(0);
        if i > N / 3 {
            arena_a.remove(a1);
        }
        let b1 = arena_b.insert(2 * i);
        if i > N / 3 {
            arena_b.remove(b1);
        }
        let c1 = arena_c.insert(i + 1);
        if i > N / 3 {
            arena_c.remove(c1);
        }
    }

    (
        arena_a.par_iter_mut(),
        arena_b.par_iter(),
        arena_c.par_iter(),
    )
        .into_par_iter()
        .for_each(|((_, a), (_, b), (_, c))| *a = *b + *c);

    let par: Vec<_> = arena_a.iter().map(|(_, a)| *a).collect();

    let seq: Vec<_> = (0..N)
        .into_iter()
        .filter_map(|x| {
            if x > N / 3 {
                None // skip the section where removals happened
            } else {
                Some((2 * x) + (x + 1)) // compute the expected value
            }
        })
        .collect();

    assert_eq!(seq, par);
}

// #[test]
// #[should_panic(expected = "too many values pushed to consumer")]
// fn par_zip_panics_on_mismatched_lengths() {
//     const N: usize = 30_000;
//     let mut arena_a = Arena::new();
//     let mut arena_b = Arena::new();

//     let handles_a: Vec<_> = (0..N).map(|i| arena_a.insert(i)).collect();
//     let handles_b: Vec<_> = (0..N).map(|i| arena_b.insert(i)).collect();

//     // Remove elements in *different* patterns so the two
//     // `ParIter`s no longer have the same real length.
//     for h in handles_a.iter().step_by(2) {
//         arena_a.remove(*h);
//     } // every 2nd
//     for h in handles_b.iter().step_by(3) {
//         arena_b.remove(*h);
//     } // every 3rd

//     // Each iterator lies about its length in a *different* way,
//     // so `zip()` asks one side to split at an impossible point.
//     let _collected: Vec<_> = arena_a.par_iter().zip(arena_b.par_iter()).collect();
//     // should panic
// }
