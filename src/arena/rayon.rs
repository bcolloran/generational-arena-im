use super::*;
use im::vector::rayon::{ParIterMut as ImParIterMut};
use im::vector::{Focus, Iter as ImIter};
use ::rayon::iter::plumbing::{bridge, Consumer, Producer, ProducerCallback, UnindexedConsumer};
use ::rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefMutIterator,
    ParallelIterator,
};

fn entry_to_mut<'a, T: Clone, I: ArenaIndex, G: FixedGenerationalIndex>(
    (index, entry): (usize, &'a mut Entry<T, I, G>),
) -> Option<(Index<T, I, G>, &'a mut T)> {
    match entry {
        Entry::Occupied { generation, value } => Some((Index::new(I::from_idx(index), *generation), value)),
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
    focus: Focus<'a, Entry<T, I, G>>,
    start: usize,
    len: usize,
}

/// Parallel iterator over mutable references to arena elements.
pub struct ParIterMut<'a, T, I, G>
where
    T: Clone + Send + Sync + 'a,
    I: ArenaIndex + Send + Sync + 'a,
    G: FixedGenerationalIndex + Send + Sync + 'a,
{
    inner: ImParIterMut<'a, Entry<T, I, G>>,
}

struct ArenaProducer<'a, T, I, G>
where
    T: Clone + Send + Sync + 'a,
    I: ArenaIndex + Send + Sync + 'a,
    G: FixedGenerationalIndex + Send + Sync + 'a,
{
    focus: Focus<'a, Entry<T, I, G>>,
    start: usize,
    len: usize,
}

struct ArenaProducerIter<'a, T, I, G>
where
    T: Clone + 'a,
    I: ArenaIndex + 'a,
    G: FixedGenerationalIndex + 'a,
{
    inner: core::iter::Enumerate<ImIter<'a, Entry<T, I, G>>>,
    start: usize,
    len: usize,
}

impl<'a, T, I, G> Iterator for ArenaProducerIter<'a, T, I, G>
where
    T: Clone + 'a,
    I: ArenaIndex,
    G: FixedGenerationalIndex,
{
    type Item = (Index<T, I, G>, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((offset, entry)) = self.inner.next() {
            match entry {
                Entry::Occupied { generation, value } => {
                    self.len -= 1;
                    let idx = Index::new(I::from_idx(self.start + offset), *generation);
                    return Some((idx, value));
                }
                Entry::Free { .. } => continue,
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T, I, G> ExactSizeIterator for ArenaProducerIter<'a, T, I, G>
where
    T: Clone + 'a,
    I: ArenaIndex,
    G: FixedGenerationalIndex,
{
    fn len(&self) -> usize {
        self.len
    }
}

impl<'a, T, I, G> core::iter::FusedIterator for ArenaProducerIter<'a, T, I, G>
where
    T: Clone + 'a,
    I: ArenaIndex,
    G: FixedGenerationalIndex,
{
}

impl<'a, T, I, G> core::iter::DoubleEndedIterator for ArenaProducerIter<'a, T, I, G>
where
    T: Clone + 'a,
    I: ArenaIndex,
    G: FixedGenerationalIndex,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        while let Some((offset, entry)) = self.inner.next_back() {
            match entry {
                Entry::Occupied { generation, value } => {
                    self.len -= 1;
                    let idx = Index::new(I::from_idx(self.start + offset), *generation);
                    return Some((idx, value));
                }
                Entry::Free { .. } => continue,
            }
        }
        None
    }
}

impl<'a, T, I, G> Producer for ArenaProducer<'a, T, I, G>
where
    T: Clone + Send + Sync + 'a,
    I: ArenaIndex + Send + Sync + 'a,
    G: FixedGenerationalIndex + Send + Sync + 'a,
{
    type Item = (Index<T, I, G>, &'a T);
    type IntoIter = ArenaProducerIter<'a, T, I, G>;

    fn into_iter(self) -> Self::IntoIter {
        ArenaProducerIter {
            inner: self.focus.into_iter().enumerate(),
            start: self.start,
            len: self.len,
        }
    }

    fn split_at(self, index: usize) -> (Self, Self) {
        if index == 0 {
            let (left, right) = self.focus.split_at(0);
            return (
                ArenaProducer { focus: left, start: self.start, len: 0 },
                ArenaProducer { focus: right, start: self.start, len: self.len },
            );
        }

        let mut live = 0usize;
        let mut split_pos = self.focus.len();
        for (i, entry) in self.focus.clone().into_iter().enumerate() {
            if let Entry::Occupied { .. } = entry {
                live += 1;
                if live == index {
                    split_pos = i + 1;
                    break;
                }
            }
        }

        let (left_focus, right_focus) = self.focus.split_at(split_pos);
        let left = ArenaProducer { focus: left_focus, start: self.start, len: index };
        let right = ArenaProducer { focus: right_focus, start: self.start + split_pos, len: self.len - index };
        (left, right)
    }
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
        C: UnindexedConsumer<Self::Item>,
    {
        bridge(self, consumer)
    }

    fn opt_len(&self) -> Option<usize> {
        Some(self.len)
    }
}

impl<'a, T, I, G> IndexedParallelIterator for ParIter<'a, T, I, G>
where
    T: Clone + Send + Sync + 'a,
    I: ArenaIndex + Send + Sync + 'a,
    G: FixedGenerationalIndex + Send + Sync + 'a,
{
    fn drive<C>(self, consumer: C) -> C::Result
    where
        C: Consumer<Self::Item>,
    {
        bridge(self, consumer)
    }

    fn len(&self) -> usize {
        self.len
    }

    fn with_producer<CB>(self, callback: CB) -> CB::Output
    where
        CB: ProducerCallback<Self::Item>,
    {
        callback.callback(ArenaProducer {
            focus: self.focus,
            start: self.start,
            len: self.len,
        })
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
        C: UnindexedConsumer<Self::Item>,
    {
        self.inner
            .enumerate()
            .filter_map(entry_to_mut::<T, I, G>)
            .drive_unindexed(consumer)
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
            focus: self.items.focus(),
            start: 0,
            len: self.len,
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
            inner: self.items.par_iter_mut(),
        }
    }
}
