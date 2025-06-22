use crate::arena::{Arena, Entry};
use crate::generation::FixedGenerationalIndex;
use crate::index::{ArenaIndex, Index};
use im::vector::rayon::{ParIter as ImParIter, ParIterMut as ImParIterMut};
use rayon::iter::{
    IntoParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator,
    ParallelIterator, IndexedParallelIterator,
};

fn entry_to_ref<'a, T: Clone, I: ArenaIndex, G: FixedGenerationalIndex>(
    (index, entry): (usize, &'a Entry<T, I, G>),
) -> Option<(Index<T, I, G>, &'a T)> {
    match entry {
        Entry::Occupied { generation, value } => {
            Some((Index::new(I::from_idx(index), *generation), value))
        }
        _ => None,
    }
}

fn entry_to_mut<'a, T: Clone, I: ArenaIndex, G: FixedGenerationalIndex>(
    (index, entry): (usize, &'a mut Entry<T, I, G>),
) -> Option<(Index<T, I, G>, &'a mut T)> {
    match entry {
        Entry::Occupied { generation, value } => {
            Some((Index::new(I::from_idx(index), *generation), value))
        }
        _ => None,
    }
}

/// Parallel iterator over shared references to arena elements.
pub struct ParIter<'a, T, I, G>
where
    T: Clone + Send + Sync + 'a,
    I: ArenaIndex + Send + Sync + 'a,
    G: FixedGenerationalIndex + Send + Sync + 'a,
{
    inner: rayon::iter::FilterMap<
        rayon::iter::Enumerate<ImParIter<'a, Entry<T, I, G>>>,
        fn((usize, &'a Entry<T, I, G>)) -> Option<(Index<T, I, G>, &'a T)>,
    >,
}

/// Parallel iterator over mutable references to arena elements.
pub struct ParIterMut<'a, T, I, G>
where
    T: Clone + Send + Sync + 'a,
    I: ArenaIndex + Send + Sync + 'a,
    G: FixedGenerationalIndex + Send + Sync + 'a,
{
    inner: rayon::iter::FilterMap<
        rayon::iter::Enumerate<ImParIterMut<'a, Entry<T, I, G>>>,
        fn((usize, &'a mut Entry<T, I, G>)) -> Option<(Index<T, I, G>, &'a mut T)>,
    >,
}

impl<'a, T, I, G> core::fmt::Debug for ParIter<'a, T, I, G>
where
    T: Clone + Send + Sync + 'a,
    I: ArenaIndex + Send + Sync + 'a,
    G: FixedGenerationalIndex + Send + Sync + 'a,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ParIter").finish()
    }
}

impl<'a, T, I, G> core::fmt::Debug for ParIterMut<'a, T, I, G>
where
    T: Clone + Send + Sync + 'a,
    I: ArenaIndex + Send + Sync + 'a,
    G: FixedGenerationalIndex + Send + Sync + 'a,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ParIterMut").finish()
    }
}

impl<'a, T, I, G> ParallelIterator for ParIter<'a, T, I, G>
where
    T: Clone + Send + Sync + 'a,
    I: ArenaIndex + Send + Sync + 'a,
    G: FixedGenerationalIndex + Send + Sync + 'a,
{
    type Item = (Index<T, I, G>, &'a T);

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: rayon::iter::plumbing::UnindexedConsumer<Self::Item>,
    {
        self.inner.drive_unindexed(consumer)
    }
}

impl<'a, T, I, G> ParallelIterator for ParIterMut<'a, T, I, G>
where
    T: Clone + Send + Sync + 'a,
    I: ArenaIndex + Send + Sync + 'a,
    G: FixedGenerationalIndex + Send + Sync + 'a,
{
    type Item = (Index<T, I, G>, &'a mut T);

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: rayon::iter::plumbing::UnindexedConsumer<Self::Item>,
    {
        self.inner.drive_unindexed(consumer)
    }
}

impl<'a, T, I, G> IntoParallelIterator for &'a Arena<T, I, G>
where
    T: Clone + Send + Sync + 'a,
    I: ArenaIndex + Send + Sync + 'a,
    G: FixedGenerationalIndex + Send + Sync + 'a,
{
    type Item = (Index<T, I, G>, &'a T);
    type Iter = ParIter<'a, T, I, G>;

    fn into_par_iter(self) -> Self::Iter {
        ParIter {
            inner: self
                .items
                .par_iter()
                .enumerate()
                .filter_map(entry_to_ref::<T, I, G>),
        }
    }
}

impl<'a, T, I, G> IntoParallelIterator for &'a mut Arena<T, I, G>
where
    T: Clone + Send + Sync + 'a,
    I: ArenaIndex + Send + Sync + 'a,
    G: FixedGenerationalIndex + Send + Sync + 'a,
{
    type Item = (Index<T, I, G>, &'a mut T);
    type Iter = ParIterMut<'a, T, I, G>;

    fn into_par_iter(self) -> Self::Iter {
        ParIterMut {
            inner: self
                .items
                .par_iter_mut()
                .enumerate()
                .filter_map(entry_to_mut::<T, I, G>),
        }
    }
}
