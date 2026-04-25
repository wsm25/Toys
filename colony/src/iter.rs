use core::iter::{ExactSizeIterator, FusedIterator};
use core::marker::PhantomData;
use std::ptr::NonNull;

use crate::{Group, GroupMask};

struct OccupyIter<T> {
    slots: NonNull<T>,
    occupied: GroupMask,
}

impl<T> OccupyIter<T> {
    #[inline]
    fn empty() -> Self {
        Self {
            slots: NonNull::dangling(),
            occupied: 0,
        }
    }

    #[inline]
    fn new(group: &Group<T>) -> Self {
        Self {
            slots: NonNull::new(group.slots.as_ptr().cast::<T>().cast_mut())
                .expect("group slots pointer must not be null"),
            occupied: group.occupied,
        }
    }

    #[inline]
    fn next(&mut self) -> Option<NonNull<T>> {
        if self.occupied == 0 {
            return None;
        }
        let slot = self.occupied.trailing_zeros() as usize;
        // This is 2x faster than (self.occupied ^= 1 << slot);
        self.occupied &= self.occupied - 1;
        Some(unsafe { self.slots.add(slot) })
    }
}

pub struct Iter<'a, T> {
    current_ptr: *const Group<T>,
    end_ptr: *const Group<T>,
    current_iter: OccupyIter<T>,
    remaining: usize,
    marker: PhantomData<&'a T>,
}

impl<'a, T> Iter<'a, T> {
    #[inline]
    pub(crate) fn new(groups: &'a [Group<T>], remaining: usize) -> Self {
        let current_ptr = groups.as_ptr();
        let end_ptr = unsafe { current_ptr.add(groups.len()) };
        let current_iter = if groups.is_empty() {
            OccupyIter::empty()
        } else {
            OccupyIter::new(unsafe { &*current_ptr })
        };

        Self {
            current_ptr,
            end_ptr,
            current_iter,
            remaining,
            marker: PhantomData,
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_ptr == self.end_ptr {
            return None;
        }

        loop {
            if let Some(value) = self.current_iter.next() {
                self.remaining -= 1;
                return Some(unsafe { value.as_ref() });
            }

            self.current_ptr = unsafe { self.current_ptr.add(1) };
            if self.current_ptr == self.end_ptr {
                return None;
            }
            self.current_iter = OccupyIter::new(unsafe { &*self.current_ptr });
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }

    #[inline]
    fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let mut acc = init;

        while self.current_ptr != self.end_ptr {
            while let Some(value) = self.current_iter.next() {
                self.remaining -= 1;
                acc = f(acc, unsafe { value.as_ref() });
            }

            self.current_ptr = unsafe { self.current_ptr.add(1) };
            if self.current_ptr != self.end_ptr {
                self.current_iter = OccupyIter::new(unsafe { &*self.current_ptr });
            }
        }

        acc
    }
}

impl<T> FusedIterator for Iter<'_, T> {}
impl<T> ExactSizeIterator for Iter<'_, T> {}

pub struct IterMut<'a, T> {
    current_ptr: *mut Group<T>,
    end_ptr: *mut Group<T>,
    current_iter: OccupyIter<T>,
    remaining: usize,
    marker: PhantomData<&'a mut T>,
}

impl<'a, T> IterMut<'a, T> {
    #[inline]
    pub(crate) fn new(groups: &'a mut [Group<T>], remaining: usize) -> Self {
        let current_ptr = groups.as_mut_ptr();
        let end_ptr = unsafe { current_ptr.add(groups.len()) };
        let current_iter = if groups.is_empty() {
            OccupyIter::empty()
        } else {
            OccupyIter::new(unsafe { &*current_ptr })
        };

        Self {
            current_ptr,
            end_ptr,
            current_iter,
            remaining,
            marker: PhantomData,
        }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_ptr == self.end_ptr {
            return None;
        }

        loop {
            if let Some(value) = self.current_iter.next() {
                self.remaining -= 1;
                return Some(unsafe { &mut *value.as_ptr() });
            }

            self.current_ptr = unsafe { self.current_ptr.add(1) };
            if self.current_ptr == self.end_ptr {
                return None;
            }
            self.current_iter = OccupyIter::new(unsafe { &*self.current_ptr });
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }

    #[inline]
    fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let mut acc = init;

        while self.current_ptr != self.end_ptr {
            while let Some(value) = self.current_iter.next() {
                self.remaining -= 1;
                acc = f(acc, unsafe { &mut *value.as_ptr() });
            }

            self.current_ptr = unsafe { self.current_ptr.add(1) };
            if self.current_ptr != self.end_ptr {
                self.current_iter = OccupyIter::new(unsafe { &*self.current_ptr });
            }
        }

        acc
    }
}

impl<T> FusedIterator for IterMut<'_, T> {}
impl<T> ExactSizeIterator for IterMut<'_, T> {}
