extern crate typed_generational_arena;

use typed_generational_arena::{NanoArena, SmallArena, StandardArena, TinyArena, TinyWrapArena};

// Macro to stress-test snapshot cloning for various arena types.
macro_rules! snapshot_stress_test {
    ($name:ident, $arena_ty:ty) => {
        #[test]
        fn $name() {
            let mut arena: $arena_ty = <$arena_ty>::new();

            let a = arena.insert(1);
            let b = arena.insert(2);
            let c = arena.insert(3);
            let snap0 = arena.clone();

            *arena.get_mut(a).unwrap() = 10;
            assert_eq!(arena.remove(b), Some(2));
            let d = arena.insert(4);
            let snap1 = arena.clone();

            assert_eq!(snap0[a], 1);
            assert_eq!(snap0[b], 2);
            assert_eq!(snap0[c], 3);
            assert!(snap0.get(d).is_none());

            assert_eq!(snap1[a], 10);
            assert!(snap1.get(b).is_none());
            assert_eq!(snap1[c], 3);
            assert_eq!(snap1[d], 4);

            arena.retain(|idx, _| idx == a || idx == d);
            assert_eq!(snap0[c], 3);
            assert_eq!(snap1[c], 3);

            let drained: Vec<_> = arena.drain().collect();
            assert_eq!(drained.len(), 2);
            assert!(arena.get(a).is_none());
            assert!(arena.get(d).is_none());

            assert_eq!(snap0[a], 1);
            assert_eq!(snap1[d], 4);

            let e = arena.insert(5);
            assert!(snap0.get(e).is_none());
            assert!(snap1.get(e).is_none());

            arena.clear();
            assert_eq!(arena.len(), 0);

            assert_eq!(snap0[a], 1);
            assert_eq!(snap1[d], 4);

            let f = arena.insert(6);
            assert!(snap0.get(f).is_none());
            assert!(snap1.get(f).is_none());
        }
    };
}

// Stress test snapshot behavior in NanoArena
snapshot_stress_test!(nano_snapshot_stress, NanoArena<usize>);
// Stress test snapshot behavior in SmallArena
snapshot_stress_test!(small_snapshot_stress, SmallArena<usize>);
// Stress test snapshot behavior in StandardArena
snapshot_stress_test!(standard_snapshot_stress, StandardArena<usize>);
// Stress test snapshot behavior in TinyArena
snapshot_stress_test!(tiny_snapshot_stress, TinyArena<usize>);
// Stress test snapshot behavior in TinyWrapArena
snapshot_stress_test!(tinywrap_snapshot_stress, TinyWrapArena<usize>);
