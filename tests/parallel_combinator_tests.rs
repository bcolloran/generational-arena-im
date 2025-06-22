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
            fn par_iter_combinators() {
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

                let seq_filter: Vec<_> = arena_a
                    .iter()
                    .filter(|item| *(item.1) % 2 == 0)
                    .map(|item| *(item.1))
                    .collect();
                let par_filter: Vec<_> = arena_a
                    .par_iter()
                    .filter(|item| *(item.1) % 2 == 0)
                    .map(|item| *(item.1))
                    .collect();
                assert_eq!(seq_filter, par_filter);

                let seq_flat_map: Vec<_> = arena_a
                    .iter()
                    .flat_map(|(_, v)| vec![*v, *v + 1])
                    .collect();
                let par_flat_map: Vec<_> = arena_a
                    .par_iter()
                    .flat_map(|(_, v)| vec![*v, *v + 1])
                    .collect();
                assert_eq!(seq_flat_map, par_flat_map);

                let nested_seq: Vec<Vec<_>> = arena_a
                    .iter()
                    .map(|(_, v)| vec![*v, *v + 1])
                    .collect();
                let nested_par: Vec<Vec<_>> = arena_a
                    .par_iter()
                    .map(|(_, v)| vec![*v, *v + 1])
                    .collect();
                let seq_flatten: Vec<_> = nested_seq.clone().into_iter().flatten().collect();
                let par_flatten: Vec<_> = nested_par.into_par_iter().flatten().collect();
                assert_eq!(seq_flatten, par_flatten);

                let seq_sum: usize = arena_a.iter().map(|(_, v)| *v).sum();
                let par_sum: usize = arena_a
                    .par_iter()
                    .fold(|| 0, |acc, (_, v)| acc + *v)
                    .reduce(|| 0, |a, b| a + b);
                assert_eq!(seq_sum, par_sum);

                let seq_min = arena_a.iter().map(|(_, v)| *v).min();
                let par_min = arena_a.par_iter().map(|(_, v)| *v).min();
                assert_eq!(seq_min, par_min);

                let seq_max = arena_a.iter().map(|(_, v)| *v).max();
                let par_max = arena_a.par_iter().map(|(_, v)| *v).max();
                assert_eq!(seq_max, par_max);
            }

            #[test]
            fn par_iter_mut_combinators() {
                let mut arena_a = init_arena();
                let arena_b = init_arena();

                let left: Vec<_> = arena_a.par_iter_mut().map(|(_, v)| v).collect();
                let right: Vec<_> = arena_b.par_iter().map(|(_, v)| *v).collect();
                left
                    .into_par_iter()
                    .zip(right.into_par_iter())
                    .for_each(|(a, b)| *a += b);

                let mut seq_arena_a = init_arena();
                let seq_arena_b = init_arena();
                for ((_, a), (_, b)) in seq_arena_a.iter_mut().zip(seq_arena_b.iter()) {
                    *a += *b;
                }

                let par_result: Vec<_> = arena_a.iter().map(|(_, v)| *v).collect();
                let seq_result: Vec<_> = seq_arena_a.iter().map(|(_, v)| *v).collect();
                assert_eq!(par_result, seq_result);

                let seq_flat_map: Vec<_> = arena_a
                    .iter()
                    .flat_map(|(_, v)| vec![*v, *v + 1])
                    .collect();
                let par_flat_map: Vec<_> = arena_a
                    .par_iter_mut()
                    .flat_map(|(_, v)| vec![*v, *v + 1])
                    .collect();
                assert_eq!(seq_flat_map, par_flat_map);

                let seq_sum: usize = arena_a.iter().map(|(_, v)| *v).sum();
                let par_sum: usize = arena_a
                    .par_iter_mut()
                    .fold(|| 0, |acc, (_, v)| acc + *v)
                    .reduce(|| 0, |a, b| a + b);
                assert_eq!(seq_sum, par_sum);

                let seq_filter: Vec<_> = arena_a
                    .iter()
                    .filter(|item| *(item.1) % 2 == 0)
                    .map(|item| *(item.1))
                    .collect();
                let par_filter: Vec<_> = arena_a
                    .par_iter_mut()
                    .filter(|item| *(item.1) % 2 == 0)
                    .map(|item| *(item.1))
                    .collect();
                assert_eq!(seq_filter, par_filter);

                let seq_min = arena_a.iter().map(|(_, v)| *v).min();
                let par_min = arena_a.par_iter_mut().map(|(_, v)| *v).min();
                assert_eq!(seq_min, par_min);

                let seq_max = arena_a.iter().map(|(_, v)| *v).max();
                let par_max = arena_a.par_iter_mut().map(|(_, v)| *v).max();
                assert_eq!(seq_max, par_max);

                let nested_seq: Vec<Vec<_>> = arena_a.iter().map(|(_, v)| vec![*v]).collect();
                let nested_par: Vec<Vec<_>> = arena_a.par_iter_mut().map(|(_, v)| vec![*v]).collect();
                let seq_flatten: Vec<_> = nested_seq.into_iter().flatten().collect();
                let par_flatten: Vec<_> = nested_par.into_par_iter().flatten().collect();
                assert_eq!(seq_flatten, par_flatten);
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
