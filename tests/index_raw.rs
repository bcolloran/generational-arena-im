extern crate generational_arena_im;
use generational_arena_im::*;

fn check_roundtrip<T, I, G>(index: I, gen: G)
where
    I: ArenaIndex + Copy + PartialEq + core::fmt::Debug,
    G: FixedGenerationalIndex + Copy + PartialEq + core::fmt::Debug,
{
    let idx = Index::<T, I, G>::from_raw(index, gen);
    assert_eq!(idx.to_raw(), (index, gen));
    let round = Index::<T, I, G>::from_raw(idx.to_raw().0, idx.to_raw().1);
    assert_eq!(idx, round);
}

#[test]
fn u64_index_roundtrip_edges() {
    check_roundtrip::<(), usize, u64>(0usize, 0u64);
    check_roundtrip::<(), usize, u64>(usize::MAX, u64::MAX);
}

#[test]
fn standard_index_roundtrip_edges() {
    let gen = NonzeroGeneration::<usize>::first_generation();
    check_roundtrip::<(), usize, NonzeroGeneration<usize>>(0usize, gen);
    check_roundtrip::<(), usize, NonzeroGeneration<usize>>(usize::MAX, gen);
}

#[test]
fn small_index_roundtrip_edges() {
    let gen = NonzeroGeneration::<u32>::first_generation();
    check_roundtrip::<(), u32, NonzeroGeneration<u32>>(0u32, gen);
    check_roundtrip::<(), u32, NonzeroGeneration<u32>>(u32::MAX, gen);
}

#[test]
fn tiny_index_roundtrip_edges() {
    let gen = NonzeroGeneration::<u16>::first_generation();
    check_roundtrip::<(), u16, NonzeroGeneration<u16>>(0u16, gen);
    check_roundtrip::<(), u16, NonzeroGeneration<u16>>(u16::MAX, gen);
}

#[test]
fn tiny_wrap_index_roundtrip_edges() {
    let gen = NonzeroWrapGeneration::<u16>::first_generation();
    check_roundtrip::<(), u16, NonzeroWrapGeneration<u16>>(0u16, gen);
    check_roundtrip::<(), u16, NonzeroWrapGeneration<u16>>(u16::MAX, gen);
}

#[test]
fn nano_index_roundtrip_edges() {
    check_roundtrip::<(), u8, core::num::Wrapping<u8>>(0u8, core::num::Wrapping(0));
    check_roundtrip::<(), u8, core::num::Wrapping<u8>>(u8::MAX, core::num::Wrapping(u8::MAX));
}

#[test]
fn pico_index_roundtrip_edges() {
    let gen = NonzeroWrapGeneration::<u8>::first_generation();
    check_roundtrip::<(), u8, NonzeroWrapGeneration<u8>>(0u8, gen);
    check_roundtrip::<(), u8, NonzeroWrapGeneration<u8>>(u8::MAX, gen);
}

#[test]
fn standard_slab_index_roundtrip_edges() {
    check_roundtrip::<(), usize, DisableRemoval>(0usize, DisableRemoval);
    check_roundtrip::<(), usize, DisableRemoval>(usize::MAX, DisableRemoval);
}

#[test]
fn small_slab_index_roundtrip_edges() {
    check_roundtrip::<(), u32, DisableRemoval>(0u32, DisableRemoval);
    check_roundtrip::<(), u32, DisableRemoval>(u32::MAX, DisableRemoval);
}

#[test]
fn ptr_slab_index_roundtrip_edges() {
    let min = NonZeroIndex::<usize>::from_idx(0);
    let max = NonZeroIndex::<usize>::from_idx(usize::MAX - 1);
    check_roundtrip::<(), NonZeroIndex<usize>, DisableRemoval>(min, DisableRemoval);
    check_roundtrip::<(), NonZeroIndex<usize>, DisableRemoval>(max, DisableRemoval);
}

#[test]
fn small_ptr_slab_index_roundtrip_edges() {
    let min = NonZeroIndex::<u32>::from_idx(0);
    let max = NonZeroIndex::<u32>::from_idx(u32::MAX as usize - 1);
    check_roundtrip::<(), NonZeroIndex<u32>, DisableRemoval>(min, DisableRemoval);
    check_roundtrip::<(), NonZeroIndex<u32>, DisableRemoval>(max, DisableRemoval);
}

