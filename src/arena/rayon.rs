use super::*;
use im::vector::{Focus, FocusMut, Iter as ImIter, IterMut as ImIterMut};
use ::rayon::iter::plumbing::{bridge, Consumer, Producer, ProducerCallback};
use ::rayon::iter::{IntoParallelIterator, ParallelIterator, IndexedParallelIterator};


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
    focus: FocusMut<'a, Entry<T, I, G>>,
    start: usize,
    len: usize,
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
        C: ::rayon::iter::plumbing::UnindexedConsumer<Self::Item>,
    {
        bridge(self, consumer)
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
        C: ::rayon::iter::plumbing::UnindexedConsumer<Self::Item>,
    {
        bridge(self, consumer)
    }
}

impl<'a, T, I, G> IndexedParallelIterator for ParIterMut<'a, T, I, G>
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
        callback.callback(ArenaMutProducer {
            focus: self.focus,
            start: self.start,
        })
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
        let len = self.len;
        let focus = self.items.focus_mut();
        ParIterMut {
            focus,
            start: 0,
            len,
        }
    }
}

struct ArenaProducer<'a, T, I, G>
where
    T: Clone + Send + Sync,
    I: ArenaIndex + Send + Sync,
    G: FixedGenerationalIndex + Send + Sync,
{
    focus: Focus<'a, Entry<T, I, G>>,
    start: usize,
}

struct ArenaMutProducer<'a, T, I, G>
where
    T: Clone + Send + Sync,
    I: ArenaIndex + Send + Sync,
    G: FixedGenerationalIndex + Send + Sync,
{
    focus: FocusMut<'a, Entry<T, I, G>>,
    start: usize,
}

struct SeqIter<'a, T, I, G> {
    start: usize,
    len: usize,
    inner: core::iter::Enumerate<ImIter<'a, Entry<T, I, G>>>,
}

struct SeqIterMut<'a, T, I, G> {
    start: usize,
    len: usize,
    inner: core::iter::Enumerate<ImIterMut<'a, Entry<T, I, G>>>,
}

impl<'a, T, I, G> Iterator for SeqIter<'a, T, I, G>
where
    T: Clone,
    I: ArenaIndex,
    G: FixedGenerationalIndex,
{
    type Item = (Index<T, I, G>, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next() {
                Some((_, &Entry::Free { .. })) => continue,
                Some((i, &Entry::Occupied { generation, ref value })) => {
                    self.len -= 1;
                    let idx = Index::new(I::from_idx(self.start + i), generation);
                    return Some((idx, value));
                }
                None => {
                    return None;
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T, I, G> ExactSizeIterator for SeqIter<'a, T, I, G>
where
    T: Clone,
    I: ArenaIndex,
    G: FixedGenerationalIndex,
{
    fn len(&self) -> usize {
        self.len
    }
}

impl<'a, T, I, G> DoubleEndedIterator for SeqIter<'a, T, I, G>
where
    T: Clone,
    I: ArenaIndex,
    G: FixedGenerationalIndex,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next_back() {
                Some((_, &Entry::Free { .. })) => continue,
                Some((i, &Entry::Occupied { generation, ref value })) => {
                    self.len -= 1;
                    let idx = Index::new(I::from_idx(self.start + i), generation);
                    return Some((idx, value));
                }
                None => {
                    return None;
                }
            }
        }
    }
}

impl<'a, T, I, G> Iterator for SeqIterMut<'a, T, I, G>
where
    T: Clone,
    I: ArenaIndex,
    G: FixedGenerationalIndex,
{
    type Item = (Index<T, I, G>, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next() {
                Some((_, &mut Entry::Free { .. })) => continue,
                Some((i, &mut Entry::Occupied { generation, ref mut value })) => {
                    self.len -= 1;
                    let idx = Index::new(I::from_idx(self.start + i), generation);
                    return Some((idx, value));
                }
                None => {
                    return None;
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T, I, G> ExactSizeIterator for SeqIterMut<'a, T, I, G>
where
    T: Clone,
    I: ArenaIndex,
    G: FixedGenerationalIndex,
{
    fn len(&self) -> usize {
        self.len
    }
}

impl<'a, T, I, G> DoubleEndedIterator for SeqIterMut<'a, T, I, G>
where
    T: Clone,
    I: ArenaIndex,
    G: FixedGenerationalIndex,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next_back() {
                Some((_, &mut Entry::Free { .. })) => continue,
                Some((i, &mut Entry::Occupied { generation, ref mut value })) => {
                    self.len -= 1;
                    let idx = Index::new(I::from_idx(self.start + i), generation);
                    return Some((idx, value));
                }
                None => {
                    return None;
                }
            }
        }
    }
}

impl<'a, T, I, G> Producer for ArenaProducer<'a, T, I, G>
where
    T: Clone + Send + Sync + 'a,
    I: ArenaIndex + Send + Sync + 'a,
    G: FixedGenerationalIndex + Send + Sync + 'a,
{
    type Item = (Index<T, I, G>, &'a T);
    type IntoIter = SeqIter<'a, T, I, G>;

    fn into_iter(self) -> Self::IntoIter {
        SeqIter {
            start: self.start,
            len: self.focus.len(),
            inner: self.focus.into_iter().enumerate(),
        }
    }

    fn split_at(self, index: usize) -> (Self, Self) {
        let (left, right) = self.focus.split_at(index);
        (
            ArenaProducer {
                focus: left,
                start: self.start,
            },
            ArenaProducer {
                focus: right,
                start: self.start + index,
            },
        )
    }
}

impl<'a, T, I, G> Producer for ArenaMutProducer<'a, T, I, G>
where
    T: Clone + Send + Sync + 'a,
    I: ArenaIndex + Send + Sync + 'a,
    G: FixedGenerationalIndex + Send + Sync + 'a,
{
    type Item = (Index<T, I, G>, &'a mut T);
    type IntoIter = SeqIterMut<'a, T, I, G>;

    fn into_iter(self) -> Self::IntoIter {
        SeqIterMut {
            start: self.start,
            len: self.focus.len(),
            inner: self.focus.into_iter().enumerate(),
        }
    }

    fn split_at(self, index: usize) -> (Self, Self) {
        let (left, right) = self.focus.split_at(index);
        (
            ArenaMutProducer {
                focus: left,
                start: self.start,
            },
            ArenaMutProducer {
                focus: right,
                start: self.start + index,
            },
        )
    }
}
