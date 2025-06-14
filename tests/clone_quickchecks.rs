extern crate typed_generational_arena;
#[macro_use]
extern crate quickcheck;

use typed_generational_arena::StandardArena as Arena;
use typed_generational_arena::StandardIndex as Index;
use quickcheck::{Arbitrary, Gen};

#[derive(Clone, Debug)]
enum Op {
    Insert(usize),
    Update(usize),
    Remove(usize),
    Clear,
    Drain,
    Retain(bool),
}

impl Arbitrary for Op {
    fn arbitrary(g: &mut Gen) -> Self {
        match u8::arbitrary(g) % 6 {
            0 => Op::Insert(usize::arbitrary(g)),
            1 => Op::Update(usize::arbitrary(g)),
            2 => Op::Remove(usize::arbitrary(g)),
            3 => Op::Clear,
            4 => Op::Drain,
            _ => Op::Retain(bool::arbitrary(g)),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(std::iter::empty())
    }
}

struct Snapshot {
    arena: Arena<usize>,
    live: Vec<(Index<usize>, usize)>,
    later: Vec<(Index<usize>, usize)>, // indices and values inserted after this snapshot
}
// Property: cloned snapshots must preserve and ignore future operations on the original arena.

quickcheck! {
    fn snapshots_survive_ops(ops: Vec<Op>) -> bool {
        let mut arena = Arena::new();
        let mut live: Vec<(Index<usize>, usize)> = Vec::new();
        let mut snaps: Vec<Snapshot> = Vec::new();

        for (i, op) in ops.into_iter().enumerate() {
            match op {
                Op::Insert(v) => {
                    let idx = arena.insert(v);
                    live.push((idx, v));
                    for s in snaps.iter_mut() {
                        s.later.push((idx, v));
                    }
                }
                Op::Update(v) => {
                    if !live.is_empty() {
                        let pos = v % live.len();
                        let (idx, _) = live[pos];
                        if let Some(slot) = arena.get_mut(idx) {
                            *slot = v;
                        }
                        live[pos].1 = v;
                    }
                }
                Op::Remove(r) => {
                    if !live.is_empty() {
                        let pos = r % live.len();
                        let (idx, val) = live.remove(pos);
                        assert_eq!(arena.remove(idx).unwrap(), val);
                    }
                }
                Op::Clear => {
                    arena.clear();
                    live.clear();
                }
                Op::Drain => {
                    for _ in arena.drain() {}
                    live.clear();
                }
                Op::Retain(keep_even) => {
                    arena.retain(|_, v| (*v % 2 == 0) == keep_even);
                    live.retain(|&(_, v)| (v % 2 == 0) == keep_even);
                }
            }

            if i % 3 == 0 {
                snaps.push(Snapshot {
                    arena: arena.clone(),
                    live: live.clone(),
                    later: Vec::new(),
                });
            }
        }

        for snap in snaps {
            for (idx, val) in &snap.live {
                match snap.arena.get(*idx) {
                    Some(v) if *v == *val => {}
                    _ => return false,
                }
            }
            for (idx, _val) in snap.later {
                match snap.live.iter().find(|(i, _)| *i == idx) {
                    Some(&(_, orig)) => {
                        if snap.arena.get(idx).map(|v| *v) != Some(orig) {
                            return false;
                        }
                    }
                    None => {
                        if snap.arena.get(idx).is_some() {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }
}
