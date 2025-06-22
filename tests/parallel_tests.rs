#![cfg(feature = "rayon")]

extern crate generational_arena_im as ga;
extern crate rayon;

use rayon::prelude::*;

macro_rules! arena_parallel_tests {
    ($mod_name:ident, $arena_ty:ty) => {
        mod $mod_name {
            use super::*;
            type ArenaType = $arena_ty;

            fn new_arena() -> ArenaType {
                let mut arena: ArenaType = <ArenaType>::new();
                for i in 0..100usize {
                    arena.insert(i);
                }
                arena
            }

            #[test]
            fn par_iter_combinators() {
                let arena = new_arena();

                let mapped: Vec<_> = arena.par_iter().map(|(_, v)| *v * 2).collect();
                assert_eq!(mapped, (0..100).map(|x| x * 2).collect::<Vec<_>>());

                let filtered: Vec<_> = arena
                    .par_iter()
                    .filter(|(_, v)| *v % 2 == 0)
                    .map(|(_, v)| *v)
                    .collect();
                assert_eq!(filtered, (0..100).filter(|x| x % 2 == 0).collect::<Vec<_>>());

                let flattened: Vec<_> = arena
                    .par_iter()
                    .map(|(_, v)| vec![*v])
                    .flatten()
                    .collect();
                assert_eq!(flattened, (0..100).collect::<Vec<_>>());

                let flat_mapped: Vec<_> = arena
                    .par_iter()
                    .flat_map(|(_, v)| vec![*v, *v + 1])
                    .collect();
                let expect_flat_map: Vec<_> = (0..100).flat_map(|x| vec![x, x + 1]).collect();
                assert_eq!(flat_mapped, expect_flat_map);

                let left: Vec<_> = arena.par_iter().map(|(_, v)| *v).collect();
                let zipped: Vec<_> = left
                    .par_iter()
                    .zip((0..100usize).into_par_iter())
                    .map(|(a, b)| *a + b)
                    .collect();
                assert_eq!(zipped, (0..100).map(|x| x * 2).collect::<Vec<_>>());

                let reduce_sum: usize = arena.par_iter().map(|(_, v)| *v).reduce(|| 0usize, |a, b| a + b);
                assert_eq!(reduce_sum, (0..100).sum::<usize>());

                let fold_sum: usize = arena
                    .par_iter()
                    .map(|(_, v)| *v)
                    .fold(|| 0usize, |acc, x| acc + x)
                    .sum();
                assert_eq!(fold_sum, (0..100).sum::<usize>());

                assert_eq!(arena.par_iter().map(|(_, v)| *v).min(), Some(0));
                assert_eq!(arena.par_iter().map(|(_, v)| *v).max(), Some(99));
            }

            #[test]
            fn par_iter_mut_combinators() {
                let mut arena = new_arena();
                let sum: usize = arena
                    .par_iter_mut()
                    .map(|(_, v)| {
                        *v *= 2;
                        vec![*v]
                    })
                    .flatten()
                    .filter(|v| *v % 3 == 0)
                    .fold(|| 0usize, |acc, v| acc + v)
                    .reduce(|| 0usize, |a, b| a + b);
                let expected_sum: usize = (0..100).map(|x| x * 2).filter(|x| x % 3 == 0).sum();
                assert_eq!(sum, expected_sum);

                let mut arena = new_arena();
                let vals: Vec<_> = arena
                    .par_iter_mut()
                    .map(|(_, v)| {
                        *v += 1;
                        *v
                    })
                    .collect();
                let zipped: Vec<_> = vals
                    .par_iter()
                    .zip((0..100usize).into_par_iter())
                    .map(|(a, b)| *a + b)
                    .collect();
                let expected: Vec<_> = (0..100).map(|x| (x + 1) + x).collect();
                assert_eq!(zipped, expected);
            }

            #[test]
            fn par_sorting() {
                let mut arena: ArenaType = <ArenaType>::new();
                for i in (0..100usize).rev() {
                    arena.insert(i);
                }

                let mut vec: Vec<_> = arena.par_iter().map(|(_, v)| *v).collect();
                vec.par_sort();
                assert_eq!(vec, (0..100).collect::<Vec<_>>());

                let mut vec2: Vec<_> = arena.par_iter().map(|(_, v)| *v).collect();
                vec2.par_sort_unstable();
                assert_eq!(vec2, (0..100).collect::<Vec<_>>());
            }
        }
    };
}

arena_parallel_tests!(base_arena, ga::Arena::<usize>);
arena_parallel_tests!(u64_arena, ga::U64Arena::<usize>);
arena_parallel_tests!(standard_arena, ga::StandardArena::<usize>);
arena_parallel_tests!(small_arena, ga::SmallArena::<usize>);
arena_parallel_tests!(tiny_arena, ga::TinyArena::<usize>);
arena_parallel_tests!(tinywrap_arena, ga::TinyWrapArena::<usize>);
arena_parallel_tests!(nano_arena, ga::NanoArena::<usize>);
arena_parallel_tests!(pico_arena, ga::PicoArena::<usize>);
arena_parallel_tests!(standard_slab, ga::StandardSlab::<usize>);
arena_parallel_tests!(small_slab, ga::SmallSlab::<usize>);
arena_parallel_tests!(ptr_slab, ga::PtrSlab::<usize>);
arena_parallel_tests!(small_ptr_slab, ga::SmallPtrSlab::<usize>);
