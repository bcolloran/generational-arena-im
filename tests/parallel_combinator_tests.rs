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
                let arena = init_arena();
                let seq_zip: Vec<_> = arena
                    .iter()
                    .map(|(_, v)| *v)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .zip(0..N)
                    .map(|(v, i)| v + i)
                    .collect();
                let par_zip: Vec<_> = arena
                    .par_iter()
                    .map(|(_, v)| *v)
                    .collect::<Vec<_>>()
                    .into_par_iter()
                    .zip((0..N).into_par_iter())
                    .map(|(v, i)| v + i)
                    .collect();
                assert_eq!(seq_zip, par_zip);

                let seq_filter: Vec<_> = arena
                    .iter()
                    .filter(|item| *(item.1) % 2 == 0)
                    .map(|item| *(item.1))
                    .collect();
                let par_filter: Vec<_> = arena
                    .par_iter()
                    .filter(|item| *(item.1) % 2 == 0)
                    .map(|item| *(item.1))
                    .collect();
                assert_eq!(seq_filter, par_filter);

                let seq_flat_map: Vec<_> = arena
                    .iter()
                    .flat_map(|(_, v)| vec![*v, *v + 1])
                    .collect();
                let par_flat_map: Vec<_> = arena
                    .par_iter()
                    .flat_map(|(_, v)| vec![*v, *v + 1])
                    .collect();
                assert_eq!(seq_flat_map, par_flat_map);

                let nested_seq: Vec<Vec<_>> = arena
                    .iter()
                    .map(|(_, v)| vec![*v, *v + 1])
                    .collect();
                let nested_par: Vec<Vec<_>> = arena
                    .par_iter()
                    .map(|(_, v)| vec![*v, *v + 1])
                    .collect();
                let seq_flatten: Vec<_> = nested_seq.clone().into_iter().flatten().collect();
                let par_flatten: Vec<_> = nested_par.into_par_iter().flatten().collect();
                assert_eq!(seq_flatten, par_flatten);

                let seq_sum: usize = arena.iter().map(|(_, v)| *v).sum();
                let par_sum: usize = arena
                    .par_iter()
                    .fold(|| 0, |acc, (_, v)| acc + *v)
                    .reduce(|| 0, |a, b| a + b);
                assert_eq!(seq_sum, par_sum);

                let seq_min = arena.iter().map(|(_, v)| *v).min();
                let par_min = arena.par_iter().map(|(_, v)| *v).min();
                assert_eq!(seq_min, par_min);

                let seq_max = arena.iter().map(|(_, v)| *v).max();
                let par_max = arena.par_iter().map(|(_, v)| *v).max();
                assert_eq!(seq_max, par_max);
            }

            #[test]
            fn par_iter_mut_combinators() {
                let mut arena = init_arena();
                let mut values: Vec<_> = arena
                    .par_iter_mut()
                    .map(|(_, v)| v)
                    .collect();
                values
                    .into_par_iter()
                    .zip((0..N).into_par_iter())
                    .for_each(|(v, i)| *v += i);

                let seq_flat_map: Vec<_> = arena
                    .iter()
                    .flat_map(|(_, v)| vec![*v, *v + 1])
                    .collect();
                let par_flat_map: Vec<_> = arena
                    .par_iter_mut()
                    .flat_map(|(_, v)| vec![*v, *v + 1])
                    .collect();
                assert_eq!(seq_flat_map, par_flat_map);

                let seq_sum: usize = arena.iter().map(|(_, v)| *v).sum();
                let par_sum: usize = arena
                    .par_iter_mut()
                    .fold(|| 0, |acc, (_, v)| acc + *v)
                    .reduce(|| 0, |a, b| a + b);
                assert_eq!(seq_sum, par_sum);

                let seq_filter: Vec<_> = arena
                    .iter()
                    .filter(|item| *(item.1) % 2 == 0)
                    .map(|item| *(item.1))
                    .collect();
                let par_filter: Vec<_> = arena
                    .par_iter_mut()
                    .filter(|item| *(item.1) % 2 == 0)
                    .map(|item| *(item.1))
                    .collect();
                assert_eq!(seq_filter, par_filter);

                let seq_min = arena.iter().map(|(_, v)| *v).min();
                let par_min = arena.par_iter_mut().map(|(_, v)| *v).min();
                assert_eq!(seq_min, par_min);

                let seq_max = arena.iter().map(|(_, v)| *v).max();
                let par_max = arena.par_iter_mut().map(|(_, v)| *v).max();
                assert_eq!(seq_max, par_max);

                let nested_seq: Vec<Vec<_>> = arena.iter().map(|(_, v)| vec![*v]).collect();
                let nested_par: Vec<Vec<_>> = arena.par_iter_mut().map(|(_, v)| vec![*v]).collect();
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
