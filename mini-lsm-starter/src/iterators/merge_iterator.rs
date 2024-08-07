use std::cmp::{self};
use std::collections::binary_heap::PeekMut;
use std::collections::BinaryHeap;

use anyhow::{Ok, Result};

use crate::key::KeySlice;

use super::StorageIterator;
struct HeapWrapper<I: StorageIterator>(pub usize, pub Box<I>);

impl<I: StorageIterator> PartialEq for HeapWrapper<I> {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other).unwrap() == cmp::Ordering::Equal
    }
}

impl<I: StorageIterator> Eq for HeapWrapper<I> {}

impl<I: StorageIterator> PartialOrd for HeapWrapper<I> {
    #[allow(clippy::non_canonical_partial_ord_impl)]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match self.1.key().cmp(&other.1.key()) {
            cmp::Ordering::Greater => Some(cmp::Ordering::Greater),
            cmp::Ordering::Less => Some(cmp::Ordering::Less),
            cmp::Ordering::Equal => self.0.partial_cmp(&other.0),
        }
        .map(|x| x.reverse())
    }
}

impl<I: StorageIterator> Ord for HeapWrapper<I> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

/// Merge multiple iterators of the same type. If the same key occurs multiple times in some
/// iterators, prefer the one with smaller index.
pub struct MergeIterator<I: StorageIterator> {
    iters: BinaryHeap<HeapWrapper<I>>,
    current: Option<HeapWrapper<I>>,
}

impl<I: StorageIterator> MergeIterator<I> {
    pub fn create(iters: Vec<Box<I>>) -> Self {
        let mut heap = BinaryHeap::new();
        if iters.is_empty() {
            return Self {
                iters: heap,
                current: None,
            };
        }

        // if iters.iter().all(|x| !x.is_valid()) {
        //     // All invalid, select the last one as the current.
        //     let mut iters = iters;
        //     return Self {
        //         iters: heap,
        //         current: Some(HeapWrapper(0, iters.pop().unwrap())),
        //     };
        // }

        for (i, iter) in iters.into_iter().enumerate() {
            if iter.is_valid() {
                heap.push(HeapWrapper(i, iter));
            }
        }

        let current = heap.pop();
        Self {
            iters: heap,
            current,
        }
    }
}

impl<I: 'static + for<'a> StorageIterator<KeyType<'a> = KeySlice<'a>>> StorageIterator
    for MergeIterator<I>
{
    type KeyType<'a> = KeySlice<'a>;

    fn key(&self) -> KeySlice {
        self.current.as_ref().unwrap().1.key()
    }

    fn value(&self) -> &[u8] {
        self.current.as_ref().unwrap().1.value()
    }

    fn is_valid(&self) -> bool {
        self.current
            .as_ref()
            .map(|x| x.1.is_valid())
            .unwrap_or(false)
    }

    fn next(&mut self) -> Result<()> {
        let current = self.current.as_mut().unwrap();
        while let Some(mut iter) = self.iters.peek_mut() {
            if iter.1.key() == current.1.key() {
                let res = iter.1.next();
                if let Err(e) = res {
                    PeekMut::pop(iter);
                    return Err(e);
                } else if !iter.1.is_valid() {
                    PeekMut::pop(iter);
                }
            } else {
                break;
            }
        }

        current.1.next()?;

        if !current.1.is_valid() {
            if let Some(iter) = self.iters.pop() {
                *current = iter;
            }
        } else {
            // if the current key is smaller, swap it with the top of the heap
            // e.g. current "e" 101 < heap top iter key "d" 100
            // PartialOrd for HeapWrapper will reverse the ordering
            // so that the top of the heap is the smallest key
            if let Some(mut iter) = self.iters.peek_mut() {
                if !(*iter < *current) {
                    std::mem::swap(&mut *iter, current);
                }
            }
        }
        Ok(())
    }
}
