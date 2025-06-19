use crate::generation::{FixedGenerationalIndex, IgnoredGeneration};
use core::cmp::Ordering;
use core::fmt::Debug;
use core::hash::Hash;
use nonzero_ext::{NonZero, NonZeroAble};
use num_traits::{FromPrimitive, ToPrimitive};

/// A type which can be used as an index to an arena
pub trait ArenaIndex: Copy {
    /// Create an arena index from a usize
    fn from_idx(idx: usize) -> Self;
    /// Transform an arena index into a usize
    fn to_idx(self) -> usize;
}
impl<T: ToPrimitive + FromPrimitive + Copy> ArenaIndex for T {
    #[inline(always)]
    fn from_idx(idx: usize) -> Self {
        Self::from_usize(idx).unwrap()
    }
    #[inline(always)]
    fn to_idx(self) -> usize {
        self.to_usize().unwrap()
    }
}

/// An arena index which is always nonzero. Useful for Option<T> size optimizations
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct NonZeroIndex<T: NonZeroAble> {
    idx: T::NonZero,
}

impl<T> ArenaIndex for NonZeroIndex<T>
where
    T: NonZeroAble + FromPrimitive,
    NonZeroIndex<T>: Copy,
    <<T as NonZeroAble>::NonZero as NonZero>::Primitive: ToPrimitive,
{
    #[inline(always)]
    fn from_idx(idx: usize) -> Self {
        NonZeroIndex {
            idx: T::from_usize(idx + 1).unwrap().into_nonzero().unwrap(),
        }
    }
    #[inline(always)]
    fn to_idx(self) -> usize {
        self.idx.get().to_usize().unwrap() - 1
    }
}

/// An index (and generation) into an `Arena`.
///
/// To get an `Index`, insert an element into an `Arena`, and the `Index` for
/// that element will be returned.
///
/// # Examples
///
/// ```
/// use generational_arena_im::StandardArena;
///
/// let mut arena = StandardArena::new();
/// let idx = arena.insert(123);
/// assert_eq!(arena[idx], 123);
/// ```
pub struct Index<T, I = usize, G = u64> {
    /// The array index of the given value
    pub(crate) index: I,
    /// The generation of the given value
    pub(crate) generation: G,
    _phantom: core::marker::PhantomData<fn() -> T>,
}

impl<T, I: ArenaIndex + Copy, G: FixedGenerationalIndex + Copy> Index<T, I, G> {
    /// Get this index as a `usize`
    pub fn to_idx(&self) -> usize {
        self.index.to_idx()
    }
    /// Get this index's array index into the arena
    pub fn arr_idx(&self) -> I {
        self.index
    }
    /// Get this index's generation
    pub fn gen(&self) -> G {
        self.generation
    }

    /// Get the raw index and generation as a tuple
    #[inline]
    pub fn to_raw(&self) -> (I, G) {
        (self.index, self.generation)
    }

    /// Create a new index from a raw index and generation
    #[inline]
    pub fn from_raw(index: I, generation: G) -> Self {
        Index {
            index,
            generation,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<T, I: ArenaIndex + Copy, G: FixedGenerationalIndex> Index<T, I, G> {
    /// Convert a `usize` to an index at the first generation
    #[inline]
    pub fn from_idx_first_gen(n: usize) -> Self {
        Index {
            index: I::from_idx(n),
            generation: G::first_generation(),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<T, I: Debug, G: Debug> Debug for Index<T, I, G> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("Index")
            .field("index", &self.index)
            .field("generation", &self.generation)
            .finish()
    }
}

impl<T, I: Copy, G: Copy> Copy for Index<T, I, G> {}

impl<T, I: Clone, G: Clone> Clone for Index<T, I, G> {
    fn clone(&self) -> Self {
        Index {
            index: self.index.clone(),
            generation: self.generation.clone(),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<T, I: Eq, G: Eq> Eq for Index<T, I, G> {}

impl<T, I: PartialEq, G: PartialEq> PartialEq for Index<T, I, G> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.generation == other.generation
    }
}

impl<T, I: Hash, G: Hash> Hash for Index<T, I, G> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
        self.generation.hash(state);
    }
}

impl<T, I: ArenaIndex + Copy, G: IgnoredGeneration> Index<T, I, G> {
    /// Convert a `usize` to an index (with generations ignored)
    #[inline(always)]
    pub fn from_idx(n: usize) -> Self {
        Self::from_idx_first_gen(n)
    }
}

impl<T, I: ArenaIndex, G: FixedGenerationalIndex> Index<T, I, G> {
    /// Create a new index from a given array index and generation
    #[inline]
    pub fn new(index: I, generation: G) -> Index<T, I, G> {
        Index {
            index: index,
            generation: generation,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, I: PartialOrd, G: FixedGenerationalIndex> PartialOrd for Index<T, I, G> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.index.partial_cmp(&other.index) {
            Some(ordering) => {
                if ordering == Ordering::Equal {
                    if self.generation.generation_lt(&other.generation) {
                        Some(Ordering::Less)
                    } else if self.generation == other.generation {
                        Some(Ordering::Equal)
                    } else {
                        Some(Ordering::Greater)
                    }
                } else {
                    Some(ordering)
                }
            }
            None => {
                if self.generation.generation_lt(&other.generation) {
                    Some(Ordering::Less)
                } else if self.generation == other.generation {
                    None
                } else {
                    Some(Ordering::Greater)
                }
            }
        }
    }
}

impl<T, I: Ord, G: FixedGenerationalIndex> Ord for Index<T, I, G> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
