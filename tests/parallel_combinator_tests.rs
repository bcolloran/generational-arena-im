#![cfg(feature = "rayon")]

extern crate generational_arena_im;
extern crate rayon;

use generational_arena_im::*;
use rayon::prelude::*;

macro_rules! arena_parallel_tests {
    ($mod_name:ident, $arena_ty:ty) => {
        mod $mod_name {
            use super::*;
            const N: usize = 64;

            fn init_arena() -> $arena_ty {
                let mut arena: $arena_ty = <$arena_ty>::new();
                for i in 0..N {
                    arena.insert(i);
                }
                arena
            }

            #[test]
            fn par_iter_zip() {
                let arena_a = init_arena();
                let arena_b = init_arena();

                let seq_zip: Vec<_> = arena_a
                    .iter()
                    .map(|(_, v)| *v)
                    .zip(arena_b.iter().map(|(_, v)| *v))
                    .map(|(a, b)| a + b)
                    .collect();

                let left: Vec<_> = arena_a.par_iter().map(|(_, v)| *v).collect();
                let right: Vec<_> = arena_b.par_iter().map(|(_, v)| *v).collect();
                let par_zip: Vec<_> = left
                    .into_par_iter()
                    .zip(right.into_par_iter())
                    .map(|(a, b)| a + b)
                    .collect();

                assert_eq!(seq_zip, par_zip);
            }

            #[test]
            fn par_iter_filter() {
                let arena = init_arena();

                let seq: Vec<_> = arena
                    .iter()
                    .filter(|item| *(item.1) % 2 == 0)
                    .map(|item| *(item.1))
                    .collect();
                let par: Vec<_> = arena
                    .par_iter()
                    .filter(|item| *(item.1) % 2 == 0)
                    .map(|item| *(item.1))
                    .collect();
                assert_eq!(seq, par);
            }

            #[test]
            fn par_iter_map() {
                let arena = init_arena();

                let seq: Vec<_> = arena.iter().map(|(_, v)| v * 2).collect();
                let par: Vec<_> = arena.par_iter().map(|(_, v)| v * 2).collect();
                assert_eq!(seq, par);
            }

            #[test]
            fn par_iter_flat_map() {
                let arena = init_arena();

                let seq: Vec<_> = arena
                    .iter()
                    .flat_map(|(_, v)| vec![*v, *v + 1])
                    .collect();
                let par: Vec<_> = arena
                    .par_iter()
                    .flat_map(|(_, v)| vec![*v, *v + 1])
                    .collect();
                assert_eq!(seq, par);
            }

            #[test]
            fn par_iter_flatten() {
                let arena = init_arena();
                let nested_seq: Vec<Vec<_>> =
                    arena.iter().map(|(_, v)| vec![*v, *v + 1]).collect();
                let nested_par: Vec<Vec<_>> =
                    arena.par_iter().map(|(_, v)| vec![*v, *v + 1]).collect();
                let seq: Vec<_> = nested_seq.clone().into_iter().flatten().collect();
                let par: Vec<_> = nested_par.into_par_iter().flatten().collect();
                assert_eq!(seq, par);
            }

            #[test]
            fn par_iter_fold_reduce() {
                let arena = init_arena();
                let seq_sum: usize = arena.iter().map(|(_, v)| *v).sum();
                let par_sum: usize = arena
                    .par_iter()
                    .fold(|| 0, |acc, (_, v)| acc + *v)
                    .reduce(|| 0, |a, b| a + b);
                assert_eq!(seq_sum, par_sum);
            }

            #[test]
            fn par_iter_min() {
                let arena = init_arena();
                let seq = arena.iter().map(|(_, v)| *v).min();
                let par = arena.par_iter().map(|(_, v)| *v).min();
                assert_eq!(seq, par);
            }

            #[test]
            fn par_iter_max() {
                let arena = init_arena();
                let seq = arena.iter().map(|(_, v)| *v).max();
                let par = arena.par_iter().map(|(_, v)| *v).max();
                assert_eq!(seq, par);
            }

            #[test]
            fn par_iter_mut_zip() {
                let mut arena_a = init_arena();
                let arena_b = init_arena();

                let left: Vec<_> = arena_a.par_iter_mut().map(|(_, v)| v).collect();
                let right: Vec<_> = arena_b.par_iter().map(|(_, v)| *v).collect();
                left
                    .into_par_iter()
                    .zip(right.into_par_iter())
                    .for_each(|(a, b)| *a += b);

                let mut seq_a = init_arena();
                let seq_b = init_arena();
                for ((_, a), (_, b)) in seq_a.iter_mut().zip(seq_b.iter()) {
                    *a += *b;
                }

                let par_result: Vec<_> = arena_a.iter().map(|(_, v)| *v).collect();
                let seq_result: Vec<_> = seq_a.iter().map(|(_, v)| *v).collect();
                assert_eq!(par_result, seq_result);
            }

            #[test]
            fn par_iter_mut_filter() {
                let mut arena = init_arena();
                let seq: Vec<_> = arena
                    .iter()
                    .filter(|item| *(item.1) % 2 == 0)
                    .map(|item| *(item.1))
                    .collect();
                let par: Vec<_> = arena
                    .par_iter_mut()
                    .filter(|item| *(item.1) % 2 == 0)
                    .map(|item| *(item.1))
                    .collect();
                assert_eq!(seq, par);
            }

            #[test]
            fn par_iter_mut_map() {
                let mut arena = init_arena();
                arena.par_iter_mut().for_each(|(_, v)| *v *= 2);
                let values: Vec<_> = arena.iter().map(|(_, v)| *v).collect();
                assert_eq!(values, (0..N).map(|x| x * 2).collect::<Vec<_>>());
            }

            #[test]
            fn par_iter_mut_flat_map() {
                let mut arena = init_arena();

                let seq: Vec<_> = arena
                    .iter()
                    .flat_map(|(_, v)| vec![*v, *v + 1])
                    .collect();
                let par: Vec<_> = arena
                    .par_iter_mut()
                    .flat_map(|(_, v)| vec![*v, *v + 1])
                    .collect();
                assert_eq!(seq, par);
            }

            #[test]
            fn par_iter_mut_flatten() {
                let mut arena = init_arena();
                let nested_seq: Vec<Vec<_>> = arena.iter().map(|(_, v)| vec![*v]).collect();
                let nested_par: Vec<Vec<_>> = arena.par_iter_mut().map(|(_, v)| vec![*v]).collect();
                let seq: Vec<_> = nested_seq.into_iter().flatten().collect();
                let par: Vec<_> = nested_par.into_par_iter().flatten().collect();
                assert_eq!(seq, par);
            }

            #[test]
            fn par_iter_mut_fold_reduce() {
                let mut arena = init_arena();
                let seq_sum: usize = arena.iter().map(|(_, v)| *v).sum();
                let par_sum: usize = arena
                    .par_iter_mut()
                    .fold(|| 0, |acc, (_, v)| acc + *v)
                    .reduce(|| 0, |a, b| a + b);
                assert_eq!(seq_sum, par_sum);
            }

            #[test]
            fn par_iter_mut_min() {
                let mut arena = init_arena();
                let seq = arena.iter().map(|(_, v)| *v).min();
                let par = arena.par_iter_mut().map(|(_, v)| *v).min();
                assert_eq!(seq, par);
            }

            #[test]
            fn par_iter_mut_max() {
                let mut arena = init_arena();
                let seq = arena.iter().map(|(_, v)| *v).max();
                let par = arena.par_iter_mut().map(|(_, v)| *v).max();
                assert_eq!(seq, par);
            }

            #[test]
            fn par_sorting() {
                let mut arena: $arena_ty = <$arena_ty>::new();
                for i in (0..N).rev() {
                    arena.insert(i);
                }
                let mut expected: Vec<_> = arena.iter().map(|(_, v)| *v).collect();
                expected.sort();

                let mut v1: Vec<_> = arena.par_iter().map(|(_, v)| *v).collect();
                v1.par_sort();
                assert_eq!(v1, expected);

                let mut v2: Vec<_> = arena.par_iter().map(|(_, v)| *v).collect();
                v2.par_sort_unstable();
                assert_eq!(v2, expected);
            }
        }
    };
}

arena_parallel_tests!(generic_arena, Arena<usize>);
arena_parallel_tests!(u64_arena, U64Arena<usize>);
arena_parallel_tests!(standard_arena, StandardArena<usize>);
arena_parallel_tests!(small_arena, SmallArena<usize>);
arena_parallel_tests!(tiny_arena, TinyArena<usize>);
arena_parallel_tests!(tiny_wrap_arena, TinyWrapArena<usize>);
arena_parallel_tests!(nano_arena, NanoArena<usize>);
arena_parallel_tests!(pico_arena, PicoArena<usize>);
arena_parallel_tests!(standard_slab, StandardSlab<usize>);
arena_parallel_tests!(small_slab, SmallSlab<usize>);
arena_parallel_tests!(ptr_slab, PtrSlab<usize>);
arena_parallel_tests!(small_ptr_slab, SmallPtrSlab<usize>);
