use crate::{vec, Vec};
use core::{cmp, iter::{self, FusedIterator, FromIterator}, mem, slice, ops};
use crate::index::{ArenaIndex, Index};
use crate::generation::{FixedGenerationalIndex, GenerationalIndex};

///
/// [See the module-level documentation for example usage and motivation.](./index.html)
#[derive(Clone, Debug)]
pub struct Arena<T, I = usize, G = usize> {
    // It is a breaking change to modify these three members, as they are needed for serialization
    items: Vec<Entry<T, I, G>>,
    generation: G,
    len: usize,
    free_list_head: Option<I>,
}

#[derive(Clone, Debug)]
enum Entry<T, I = usize, G = u64> {
    Free { next_free: Option<I> },
    Occupied { generation: G, value: T },
}

/// An index (and generation) into an `Arena`.
///
/// To get an `Index`, insert an element into an `Arena`, and the `Index` for
/// that element will be returned.
///
/// # Examples
///
/// ```
/// use typed_generational_arena::StandardArena;
///
/// let mut arena = StandardArena::new();
/// let idx = arena.insert(123);
/// assert_eq!(arena[idx], 123);
/// ```

const DEFAULT_CAPACITY: usize = 4;

impl<T, I: ArenaIndex, G: FixedGenerationalIndex> Arena<T, I, G> {
    /// Constructs a new, empty `Arena`.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_generational_arena::StandardArena;
    ///
    /// let mut arena = StandardArena::<usize>::new();
    /// # let _ = arena;
    /// ```
    pub fn new() -> Arena<T, I, G> {
        Arena::with_capacity(DEFAULT_CAPACITY)
    }

    /// Constructs a new, empty `Arena<T>` with the specified capacity.
    ///
    /// The `Arena<T>` will be able to hold `n` elements without further allocation.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_generational_arena::StandardArena;
    ///
    /// let mut arena = StandardArena::with_capacity(10);
    ///
    /// // These insertions will not require further allocation.
    /// for i in 0..10 {
    ///     assert!(arena.try_insert(i).is_ok());
    /// }
    ///
    /// // But now we are at capacity, and there is no more room.
    /// assert!(arena.try_insert(99).is_err());
    /// ```
    pub fn with_capacity(n: usize) -> Arena<T, I, G> {
        let n = cmp::max(n, 1);
        let mut arena = Arena {
            items: Vec::new(),
            generation: G::first_generation(),
            free_list_head: None,
            len: 0,
        };
        arena.reserve(n);
        arena
    }

    /// Clear all the items inside the arena, but keep its allocation.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_generational_arena::StandardArena;
    ///
    /// let mut arena = StandardArena::with_capacity(1);
    /// arena.insert(42);
    /// arena.insert(43);
    ///
    /// arena.clear();
    ///
    /// assert_eq!(arena.capacity(), 2);
    /// ```
    pub fn clear(&mut self) {
        self.items.clear();

        let end = self.items.capacity();
        self.items.extend((0..end).map(|i| {
            if i == end - 1 {
                Entry::Free { next_free: None }
            } else {
                Entry::Free {
                    next_free: Some(I::from_idx(i + 1)),
                }
            }
        }));
        self.free_list_head = Some(I::from_idx(0));
        self.len = 0;
    }

    /// Attempts to insert `value` into the arena using existing capacity.
    ///
    /// This method will never allocate new capacity in the arena.
    ///
    /// If insertion succeeds, then the `value`'s index is returned. If
    /// insertion fails, then `Err(value)` is returned to give ownership of
    /// `value` back to the caller.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_generational_arena::StandardArena;
    ///
    /// let mut arena = StandardArena::new();
    ///
    /// match arena.try_insert(42) {
    ///     Ok(idx) => {
    ///         // Insertion succeeded.
    ///         assert_eq!(arena[idx], 42);
    ///     }
    ///     Err(x) => {
    ///         // Insertion failed.
    ///         assert_eq!(x, 42);
    ///     }
    /// };
    /// ```
    #[inline]
    pub fn try_insert(&mut self, value: T) -> Result<Index<T, I, G>, T> {
        match self.free_list_head {
            None => Err(value),
            Some(i) => {
                let idx = i.to_idx();
                match &self.items[idx] {
                    Entry::Occupied { .. } => panic!("corrupt free list"),
                    Entry::Free { next_free } => {
                        self.free_list_head = *next_free;
                        self.len += 1;
                        self.items[idx] = Entry::Occupied {
                            generation: self.generation,
                            value,
                        };
                        Ok(Index::new(i, self.generation))
                    }
                }
            }
        }
    }

    /// Insert `value` into the arena, allocating more capacity if necessary.
    ///
    /// The `value`'s associated index in the arena is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_generational_arena::StandardArena;
    ///
    /// let mut arena = StandardArena::new();
    ///
    /// let idx = arena.insert(42);
    /// assert_eq!(arena[idx], 42);
    /// ```
    #[inline]
    pub fn insert(&mut self, value: T) -> Index<T, I, G> {
        match self.try_insert(value) {
            Ok(i) => i,
            Err(value) => self.insert_slow_path(value),
        }
    }

    #[inline(never)]
    fn insert_slow_path(&mut self, value: T) -> Index<T, I, G> {
        let len = self.items.len();
        self.reserve(len);
        self.try_insert(value)
            .map_err(|_| ())
            .expect("inserting will always succeed after reserving additional space")
    }

    /// Is the element at index `i` in the arena?
    ///
    /// Returns `true` if the element at `i` is in the arena, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_generational_arena::StandardArena;
    ///
    /// let mut arena = StandardArena::new();
    /// let idx = arena.insert(42);
    ///
    /// assert!(arena.contains(idx));
    /// arena.remove(idx);
    /// assert!(!arena.contains(idx));
    /// ```
    pub fn contains(&self, i: Index<T, I, G>) -> bool {
        self.get(i).is_some()
    }

    /// Get a shared reference to the element at index `i` if it is in the
    /// arena.
    ///
    /// If the element at index `i` is not in the arena, then `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_generational_arena::StandardArena;
    ///
    /// let mut arena = StandardArena::new();
    /// let idx = arena.insert(42);
    ///
    /// assert_eq!(arena.get(idx), Some(&42));
    /// arena.remove(idx);
    /// assert!(arena.get(idx).is_none());
    /// ```
    pub fn get(&self, i: Index<T, I, G>) -> Option<&T> {
        match self.items.get(i.index.to_idx()) {
            Some(Entry::Occupied {
                generation,
                ref value,
            }) if *generation == i.generation => Some(value),
            _ => None,
        }
    }

    /// Get an exclusive reference to the element at index `i` if it is in the
    /// arena.
    ///
    /// If the element at index `i` is not in the arena, then `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_generational_arena::StandardArena;
    ///
    /// let mut arena = StandardArena::new();
    /// let idx = arena.insert(42);
    ///
    /// *arena.get_mut(idx).unwrap() += 1;
    /// assert_eq!(arena.remove(idx), Some(43));
    /// assert!(arena.get_mut(idx).is_none());
    /// ```
    pub fn get_mut(&mut self, i: Index<T, I, G>) -> Option<&mut T> {
        match self.items.get_mut(i.index.to_idx()) {
            Some(Entry::Occupied {
                generation,
                ref mut value,
            }) if *generation == i.generation => Some(value),
            _ => None,
        }
    }

    /// Get a pair of exclusive references to the elements at index `i1` and `i2` if it is in the
    /// arena.
    ///
    /// If the element at index `i1` or `i2` is not in the arena, then `None` is returned for this
    /// element.
    ///
    /// # Panics
    ///
    /// Panics if `i1` and `i2` are pointing to the same item of the arena.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_generational_arena::StandardArena;
    ///
    /// let mut arena = StandardArena::new();
    /// let idx1 = arena.insert(0);
    /// let idx2 = arena.insert(1);
    ///
    /// {
    ///     let (item1, item2) = arena.get2_mut(idx1, idx2);
    ///
    ///     *item1.unwrap() = 3;
    ///     *item2.unwrap() = 4;
    /// }
    ///
    /// assert_eq!(arena[idx1], 3);
    /// assert_eq!(arena[idx2], 4);
    /// ```
    pub fn get2_mut(
        &mut self,
        i1: Index<T, I, G>,
        i2: Index<T, I, G>,
    ) -> (Option<&mut T>, Option<&mut T>) {
        let len = self.items.len();
        let idx = (i1.index.to_idx(), i2.index.to_idx());
        let gen = (i1.generation, i2.generation);

        if idx.0 == idx.1 {
            assert!(gen.0 != gen.1);

            if gen.1.generation_lt(&gen.0) {
                return (self.get_mut(i1), None);
            }
            return (None, self.get_mut(i2));
        }

        if idx.0 >= len {
            return (None, self.get_mut(i2));
        } else if idx.1 >= len {
            return (self.get_mut(i1), None);
        }

        let (raw_item1, raw_item2) = {
            let (xs, ys) = self.items.split_at_mut(cmp::max(idx.0, idx.1));
            if idx.0 < idx.1 {
                (&mut xs[idx.0], &mut ys[0])
            } else {
                (&mut ys[0], &mut xs[idx.1])
            }
        };

        let item1 = match raw_item1 {
            Entry::Occupied {
                generation,
                ref mut value,
            } if *generation == gen.0 => Some(value),
            _ => None,
        };

        let item2 = match raw_item2 {
            Entry::Occupied {
                generation,
                ref mut value,
            } if *generation == gen.1 => Some(value),
            _ => None,
        };

        (item1, item2)
    }

    /// Get the length of this arena.
    ///
    /// The length is the number of elements the arena holds.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_generational_arena::StandardArena;
    ///
    /// let mut arena = StandardArena::new();
    /// assert_eq!(arena.len(), 0);
    ///
    /// let idx = arena.insert(42);
    /// assert_eq!(arena.len(), 1);
    ///
    /// let _ = arena.insert(0);
    /// assert_eq!(arena.len(), 2);
    ///
    /// assert_eq!(arena.remove(idx), Some(42));
    /// assert_eq!(arena.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the arena contains no elements
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_generational_arena::StandardArena;
    ///
    /// let mut arena = StandardArena::new();
    /// assert!(arena.is_empty());
    ///
    /// let idx = arena.insert(42);
    /// assert!(!arena.is_empty());
    ///
    /// assert_eq!(arena.remove(idx), Some(42));
    /// assert!(arena.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the capacity of this arena.
    ///
    /// The capacity is the maximum number of elements the arena can hold
    /// without further allocation, including however many it currently
    /// contains.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_generational_arena::StandardArena;
    ///
    /// let mut arena = StandardArena::with_capacity(10);
    /// assert_eq!(arena.capacity(), 10);
    ///
    /// // `try_insert` does not allocate new capacity.
    /// for i in 0..10 {
    ///     assert!(arena.try_insert(1).is_ok());
    ///     assert_eq!(arena.capacity(), 10);
    /// }
    ///
    /// // But `insert` will if the arena is already at capacity.
    /// arena.insert(0);
    /// assert!(arena.capacity() > 10);
    /// ```
    pub fn capacity(&self) -> usize {
        self.items.len()
    }

    /// Allocate space for `additional_capacity` more elements in the arena.
    ///
    /// # Panics
    ///
    /// Panics if this causes the capacity to overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_generational_arena::StandardArena;
    ///
    /// let mut arena = StandardArena::with_capacity(10);
    /// arena.reserve(5);
    /// assert_eq!(arena.capacity(), 15);
    /// # let _: StandardArena<usize> = arena;
    /// ```
    pub fn reserve(&mut self, additional_capacity: usize) {
        let start = self.items.len();
        let end = self.items.len() + additional_capacity;
        let old_head = self.free_list_head;
        self.items.reserve_exact(additional_capacity);
        self.items.extend((start..end).map(|i| {
            if i == end - 1 {
                Entry::Free {
                    next_free: old_head,
                }
            } else {
                Entry::Free {
                    next_free: Some(I::from_idx(i + 1)),
                }
            }
        }));
        self.free_list_head = Some(I::from_idx(start));
    }

    /// Iterate over shared references to the elements in this arena.
    ///
    /// Yields pairs of `(Index<T>, &T)` items.
    ///
    /// Order of iteration is not defined.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_generational_arena::StandardArena;
    ///
    /// let mut arena = StandardArena::new();
    /// for i in 0..10 {
    ///     arena.insert(i * i);
    /// }
    ///
    /// for (idx, value) in arena.iter() {
    ///     println!("{} is at index {:?}", value, idx);
    /// }
    /// ```
    pub fn iter(&self) -> Iter<T, I, G> {
        Iter {
            len: self.len,
            inner: self.items.iter().enumerate(),
        }
    }

    /// Iterate over exclusive references to the elements in this arena.
    ///
    /// Yields pairs of `(Index<T>, &mut T)` items.
    ///
    /// Order of iteration is not defined.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_generational_arena::StandardArena;
    ///
    /// let mut arena = StandardArena::new();
    /// for i in 0..10 {
    ///     arena.insert(i * i);
    /// }
    ///
    /// for (_idx, value) in arena.iter_mut() {
    ///     *value += 5;
    /// }
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<T, I, G> {
        IterMut {
            len: self.len,
            inner: self.items.iter_mut().enumerate(),
        }
    }

    /// Iterate over elements of the arena and remove them.
    ///
    /// Yields pairs of `(Index<T>, T)` items.
    ///
    /// Order of iteration is not defined.
    ///
    /// Note: All elements are removed even if the iterator is only partially consumed or not consumed at all.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_generational_arena::StandardArena;
    ///
    /// let mut arena = StandardArena::new();
    /// let idx_1 = arena.insert("hello");
    /// let idx_2 = arena.insert("world");
    ///
    /// assert!(arena.get(idx_1).is_some());
    /// assert!(arena.get(idx_2).is_some());
    /// for (idx, value) in arena.drain() {
    ///     assert!((idx == idx_1 && value == "hello") || (idx == idx_2 && value == "world"));
    /// }
    /// assert!(arena.get(idx_1).is_none());
    /// assert!(arena.get(idx_2).is_none());
    /// ```
    pub fn drain(&mut self) -> Drain<T, I, G> {
        Drain {
            inner: self.items.drain(..).enumerate(),
        }
    }
}

impl<T, I: ArenaIndex, G: GenerationalIndex> Arena<T, I, G> {
    /// Remove the element at index `i` from the arena.
    ///
    /// If the element at index `i` is still in the arena, then it is
    /// returned. If it is not in the arena, then `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_generational_arena::StandardArena;
    ///
    /// let mut arena = StandardArena::new();
    /// let idx = arena.insert(42);
    ///
    /// assert_eq!(arena.remove(idx), Some(42));
    /// assert_eq!(arena.remove(idx), None);
    /// ```
    pub fn remove(&mut self, i: Index<T, I, G>) -> Option<T> {
        if i.index.to_idx() >= self.items.len() {
            return None;
        }

        let entry = mem::replace(
            &mut self.items[i.index.to_idx()],
            Entry::Free {
                next_free: self.free_list_head,
            },
        );
        match entry {
            Entry::Occupied { generation, value } => {
                if generation == i.generation {
                    self.generation.increment_generation();
                    self.free_list_head = Some(i.index);
                    self.len -= 1;
                    Some(value)
                } else {
                    self.items[i.index.to_idx()] = Entry::Occupied { generation, value };
                    None
                }
            }
            e @ Entry::Free { .. } => {
                self.items[i.index.to_idx()] = e;
                None
            }
        }
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all indices such that `predicate(index, &value)` returns `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_generational_arena::StandardArena;
    ///
    /// let mut crew = StandardArena::new();
    /// crew.extend(&["Jim Hawkins", "John Silver", "Alexander Smollett", "Israel Hands"]);
    /// let pirates = ["John Silver", "Israel Hands"]; // too dangerous to keep them around
    /// crew.retain(|_index, member| !pirates.contains(member));
    /// let mut crew_members = crew.iter().map(|(_, member)| **member);
    /// assert_eq!(crew_members.next(), Some("Jim Hawkins"));
    /// assert_eq!(crew_members.next(), Some("Alexander Smollett"));
    /// assert!(crew_members.next().is_none());
    /// ```
    pub fn retain(&mut self, mut predicate: impl FnMut(Index<T, I, G>, &T) -> bool) {
        for i in 0..self.items.len() {
            let remove = match &self.items[i] {
                Entry::Occupied { generation, value } => {
                    let index = Index::new(I::from_idx(i), *generation);
                    if predicate(index, value) {
                        None
                    } else {
                        Some(index)
                    }
                }
                _ => None,
            };
            if let Some(index) = remove {
                self.remove(index);
            }
        }
    }
}

impl<T, I: ArenaIndex, G: FixedGenerationalIndex> IntoIterator for Arena<T, I, G> {
    type Item = T;
    type IntoIter = IntoIter<T, I, G>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            len: self.len,
            inner: self.items.into_iter(),
        }
    }
}

/// An iterator over the elements in an arena.
///
/// Yields `T` items.
///
/// Order of iteration is not defined.
///
/// # Examples
///
/// ```
/// use typed_generational_arena::StandardArena;
///
/// let mut arena = StandardArena::new();
/// for i in 0..10 {
///     arena.insert(i * i);
/// }
///
/// for value in arena {
///     assert!(value < 100);
/// }
/// ```
#[derive(Clone, Debug)]
pub struct IntoIter<T, I: ArenaIndex, G: FixedGenerationalIndex> {
    len: usize,
    inner: vec::IntoIter<Entry<T, I, G>>,
}

impl<T, I: ArenaIndex, G: FixedGenerationalIndex> Iterator for IntoIter<T, I, G> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next() {
                Some(Entry::Free { .. }) => continue,
                Some(Entry::Occupied { value, .. }) => {
                    self.len -= 1;
                    return Some(value);
                }
                None => {
                    debug_assert_eq!(self.len, 0);
                    return None;
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<T, I: ArenaIndex, G: FixedGenerationalIndex> DoubleEndedIterator for IntoIter<T, I, G> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next_back() {
                Some(Entry::Free { .. }) => continue,
                Some(Entry::Occupied { value, .. }) => {
                    self.len -= 1;
                    return Some(value);
                }
                None => {
                    debug_assert_eq!(self.len, 0);
                    return None;
                }
            }
        }
    }
}

impl<T, I: ArenaIndex, G: FixedGenerationalIndex> ExactSizeIterator for IntoIter<T, I, G> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<T, I: ArenaIndex, G: FixedGenerationalIndex> FusedIterator for IntoIter<T, I, G> {}

impl<'a, T, I: ArenaIndex, G: FixedGenerationalIndex> IntoIterator for &'a Arena<T, I, G> {
    type Item = (Index<T, I, G>, &'a T);
    type IntoIter = Iter<'a, T, I, G>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An iterator over shared references to the elements in an arena.
///
/// Yields pairs of `(Index<T>, &T)` items.
///
/// Order of iteration is not defined.
///
/// # Examples
///
/// ```
/// use typed_generational_arena::StandardArena;
///
/// let mut arena = StandardArena::new();
/// for i in 0..10 {
///     arena.insert(i * i);
/// }
///
/// for (idx, value) in &arena {
///     println!("{} is at index {:?}", value, idx);
/// }
/// ```
#[derive(Clone, Debug)]
pub struct Iter<'a, T: 'a, I: 'a + ArenaIndex, G: 'a + FixedGenerationalIndex> {
    len: usize,
    inner: iter::Enumerate<slice::Iter<'a, Entry<T, I, G>>>,
}

impl<'a, T, I: 'a + ArenaIndex, G: 'a + FixedGenerationalIndex> Iterator for Iter<'a, T, I, G> {
    type Item = (Index<T, I, G>, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next() {
                Some((_, &Entry::Free { .. })) => continue,
                Some((
                    index,
                    &Entry::Occupied {
                        generation,
                        ref value,
                    },
                )) => {
                    self.len -= 1;
                    let idx = Index::new(I::from_idx(index), generation);
                    return Some((idx, value));
                }
                None => {
                    debug_assert_eq!(self.len, 0);
                    return None;
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T, I: 'a + ArenaIndex, G: 'a + FixedGenerationalIndex> DoubleEndedIterator
    for Iter<'a, T, I, G>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next_back() {
                Some((_, &Entry::Free { .. })) => continue,
                Some((
                    index,
                    &Entry::Occupied {
                        generation,
                        ref value,
                    },
                )) => {
                    self.len -= 1;
                    let idx = Index::new(I::from_idx(index), generation);
                    return Some((idx, value));
                }
                None => {
                    debug_assert_eq!(self.len, 0);
                    return None;
                }
            }
        }
    }
}

impl<'a, T, I: ArenaIndex, G: FixedGenerationalIndex> ExactSizeIterator for Iter<'a, T, I, G> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<'a, T, I: ArenaIndex, G: FixedGenerationalIndex> FusedIterator for Iter<'a, T, I, G> {}

impl<'a, T, I: ArenaIndex, G: FixedGenerationalIndex> IntoIterator for &'a mut Arena<T, I, G> {
    type Item = (Index<T, I, G>, &'a mut T);
    type IntoIter = IterMut<'a, T, I, G>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// An iterator over exclusive references to elements in this arena.
///
/// Yields pairs of `(Index<T>, &mut T)` items.
///
/// Order of iteration is not defined.
///
/// # Examples
///
/// ```
/// use typed_generational_arena::StandardArena;
///
/// let mut arena = StandardArena::new();
/// for i in 0..10 {
///     arena.insert(i * i);
/// }
///
/// for (_idx, value) in &mut arena {
///     *value += 5;
/// }
/// ```
#[derive(Debug)]
pub struct IterMut<'a, T: 'a, I: 'a + ArenaIndex, G: 'a + FixedGenerationalIndex> {
    len: usize,
    inner: iter::Enumerate<slice::IterMut<'a, Entry<T, I, G>>>,
}

impl<'a, T, I: 'a + ArenaIndex, G: 'a + FixedGenerationalIndex> Iterator for IterMut<'a, T, I, G> {
    type Item = (Index<T, I, G>, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next() {
                Some((_, &mut Entry::Free { .. })) => continue,
                Some((
                    index,
                    &mut Entry::Occupied {
                        generation,
                        ref mut value,
                    },
                )) => {
                    self.len -= 1;
                    let idx = Index::new(I::from_idx(index), generation);
                    return Some((idx, value));
                }
                None => {
                    debug_assert_eq!(self.len, 0);
                    return None;
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T, I: 'a + ArenaIndex, G: 'a + FixedGenerationalIndex> DoubleEndedIterator
    for IterMut<'a, T, I, G>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next_back() {
                Some((_, &mut Entry::Free { .. })) => continue,
                Some((
                    index,
                    &mut Entry::Occupied {
                        generation,
                        ref mut value,
                    },
                )) => {
                    self.len -= 1;
                    let idx = Index::new(I::from_idx(index), generation);
                    return Some((idx, value));
                }
                None => {
                    debug_assert_eq!(self.len, 0);
                    return None;
                }
            }
        }
    }
}

impl<'a, T, I: 'a + ArenaIndex, G: 'a + FixedGenerationalIndex> ExactSizeIterator
    for IterMut<'a, T, I, G>
{
    fn len(&self) -> usize {
        self.len
    }
}

impl<'a, T, I: 'a + ArenaIndex, G: 'a + FixedGenerationalIndex> FusedIterator
    for IterMut<'a, T, I, G>
{
}

/// An iterator that removes elements from the arena.
///
/// Yields pairs of `(Index<T>, T)` items.
///
/// Order of iteration is not defined.
///
/// Note: All elements are removed even if the iterator is only partially consumed or not consumed at all.
///
/// # Examples
///
/// ```
/// use typed_generational_arena::StandardArena;
///
/// let mut arena = StandardArena::new();
/// let idx_1 = arena.insert("hello");
/// let idx_2 = arena.insert("world");
///
/// assert!(arena.get(idx_1).is_some());
/// assert!(arena.get(idx_2).is_some());
/// for (idx, value) in arena.drain() {
///     assert!((idx == idx_1 && value == "hello") || (idx == idx_2 && value == "world"));
/// }
/// assert!(arena.get(idx_1).is_none());
/// assert!(arena.get(idx_2).is_none());
/// ```
#[derive(Debug)]
pub struct Drain<'a, T: 'a, I: ArenaIndex, G: FixedGenerationalIndex> {
    inner: iter::Enumerate<vec::Drain<'a, Entry<T, I, G>>>,
}

impl<'a, T, I: ArenaIndex, G: FixedGenerationalIndex> Iterator for Drain<'a, T, I, G> {
    type Item = (Index<T, I, G>, T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next() {
                Some((_, Entry::Free { .. })) => continue,
                Some((index, Entry::Occupied { generation, value })) => {
                    let idx = Index::new(I::from_idx(index), generation);
                    return Some((idx, value));
                }
                None => return None,
            }
        }
    }
}

impl<T, Idx: ArenaIndex, G: FixedGenerationalIndex> Extend<T> for Arena<T, Idx, G> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for t in iter {
            self.insert(t);
        }
    }
}

impl<T, Idx: ArenaIndex, G: FixedGenerationalIndex> FromIterator<T> for Arena<T, Idx, G> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let (lower, upper) = iter.size_hint();
        let cap = upper.unwrap_or(lower);
        let cap = cmp::max(cap, 1);
        let mut arena = Arena::with_capacity(cap);
        arena.extend(iter);
        arena
    }
}

impl<T, I: ArenaIndex, G: FixedGenerationalIndex> ops::Index<Index<T, I, G>> for Arena<T, I, G> {
    type Output = T;

    fn index(&self, index: Index<T, I, G>) -> &Self::Output {
        self.get(index).expect("No element at index")
    }
}

impl<T, I: ArenaIndex, G: FixedGenerationalIndex> ops::IndexMut<Index<T, I, G>> for Arena<T, I, G> {
    fn index_mut(&mut self, index: Index<T, I, G>) -> &mut Self::Output {
        self.get_mut(index).expect("No element at index")
    }
}
