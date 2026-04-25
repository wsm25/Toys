mod group;
mod iter;

pub(crate) use group::{GROUP_LEN, Group, GroupMask};
pub use iter::{Iter, IterMut};

use std::ptr::NonNull;

#[derive(Clone, Copy)]
pub struct Handle {
    bits: u64,
}

impl Handle {
    const GROUP_BITS: u64 = 26;
    const SLOT_BITS: u64 = 6;
    const GENERATION_SHIFT: u64 = Self::GROUP_BITS + Self::SLOT_BITS;
    const GROUP_MASK: u64 = (1 << Self::GROUP_BITS) - 1;
    const SLOT_MASK: u64 = (1 << Self::SLOT_BITS) - 1;

    #[inline]
    const fn new_unchecked(group: usize, slot: u8, generation: u32) -> Self {
        debug_assert!(group <= Self::GROUP_MASK as usize);
        debug_assert!((slot as u64) <= Self::SLOT_MASK);
        Self {
            bits: group as u64
                | ((slot as u64) << Self::GROUP_BITS)
                | ((generation as u64) << Self::GENERATION_SHIFT),
        }
    }

    #[inline]
    pub const fn group(self) -> usize {
        (self.bits & Self::GROUP_MASK) as usize
    }

    #[inline]
    pub const fn slot(self) -> usize {
        ((self.bits >> Self::GROUP_BITS) & Self::SLOT_MASK) as usize
    }

    #[inline]
    pub const fn generation(self) -> u32 {
        (self.bits >> Self::GENERATION_SHIFT) as u32
    }
}

// colony

pub struct Colony<T> {
    groups: Vec<Group<T>>,
    non_full_groups: Vec<usize>,
    tail_ptr: NonNull<Group<T>>,
    tail_group: usize,
    tail_slot: usize,
    len: usize,
}

impl<T> Default for Colony<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Colony<T> {
    #[inline]
    pub fn new() -> Self {
        Self {
            groups: Vec::new(),
            non_full_groups: Vec::new(),
            tail_ptr: NonNull::dangling(),
            tail_group: 0,
            tail_slot: 0,
            len: 0,
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        let group_count = capacity.div_ceil(GROUP_LEN);
        let mut groups = Vec::with_capacity(group_count);
        groups.resize_with(group_count, Group::new);

        let tail_ptr = NonNull::new(groups.as_mut_ptr()).unwrap_or_else(NonNull::dangling);

        Self {
            groups,
            non_full_groups: Vec::new(),
            tail_ptr,
            tail_group: 0,
            tail_slot: 0,
            len: 0,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.groups.len() * GROUP_LEN
    }

    #[inline]
    pub fn insert(&mut self, value: T) -> Handle {
        if self.non_full_groups.is_empty() {
            return self.insert_tail(value);
        }

        self.insert_reusing_hole(value)
    }

    #[inline]
    fn insert_tail(&mut self, value: T) -> Handle {
        if self.tail_group == self.groups.len() {
            self.groups.push(Group::new());
            self.tail_ptr =
                NonNull::from(self.groups.last_mut().expect("fresh group was just pushed"));
        }

        let group_index = self.tail_group;
        let slot = self.tail_slot;
        let group = unsafe { self.tail_ptr.as_mut() };
        let generation = group.insert_at(slot, value);

        self.tail_slot += 1;
        if self.tail_slot == GROUP_LEN {
            if !group.is_full() {
                self.non_full_groups.push(group_index);
            }
            self.tail_group += 1;
            self.tail_slot = 0;
            self.refresh_tail_ptr();
        }

        self.len += 1;
        Handle::new_unchecked(group_index, slot as u8, generation)
    }

    #[cold]
    fn insert_reusing_hole(&mut self, value: T) -> Handle {
        while let Some(&group_index) = self.non_full_groups.last() {
            let slot_limit = if group_index == self.tail_group {
                self.tail_slot
            } else {
                GROUP_LEN
            };
            let Some(slot) = self.groups[group_index].first_free_slot_before(slot_limit) else {
                self.non_full_groups.pop();
                continue;
            };

            let group = &mut self.groups[group_index];
            let generation = group.insert_at(slot, value);
            let is_now_full = group.is_full_before(slot_limit);
            if is_now_full {
                self.non_full_groups.pop();
                group.in_free_list = false;
            }

            self.len += 1;
            return Handle::new_unchecked(group_index, slot as u8, generation);
        }

        self.insert_tail(value)
    }

    #[inline]
    pub fn get(&self, handle: Handle) -> Option<&T> {
        self.groups
            .get(handle.group())?
            .get(handle.slot(), handle.generation())
    }

    /// Returns a reference to the value for `handle` without validating it.
    ///
    /// # Safety
    ///
    /// `handle` must be live in this colony.
    #[inline]
    pub unsafe fn get_unchecked(&self, handle: Handle) -> &T {
        debug_assert!(self.get(handle).is_some());
        unsafe {
            self.groups
                .get_unchecked(handle.group())
                .get_unchecked(handle.slot())
        }
    }

    #[inline]
    pub fn get_mut(&mut self, handle: Handle) -> Option<&mut T> {
        self.groups
            .get_mut(handle.group())?
            .get_mut(handle.slot(), handle.generation())
    }

    #[inline]
    pub fn remove(&mut self, handle: Handle) -> Option<T> {
        let group_index = handle.group();
        let is_initialized_slot = self.is_initialized_slot(group_index, handle.slot());
        let group = self.groups.get_mut(group_index)?;
        let was_full = group.is_full();
        let value = group.pop(handle.slot(), handle.generation())?;
        if is_initialized_slot && (was_full || !group.in_free_list) {
            self.non_full_groups.push(group_index);
            group.in_free_list = true;
        }
        self.len -= 1;
        Some(value)
    }

    pub fn clear(&mut self) {
        self.non_full_groups.clear();
        for group in &mut self.groups {
            group.clear();
            group.in_free_list = false;
        }
        self.tail_group = 0;
        self.tail_slot = 0;
        self.refresh_tail_ptr();
        self.len = 0;
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter::new(&self.groups, self.len)
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut::new(&mut self.groups, self.len)
    }

    #[inline]
    fn is_initialized_slot(&self, group_index: usize, slot: usize) -> bool {
        group_index < self.tail_group || (group_index == self.tail_group && slot < self.tail_slot)
    }

    #[inline]
    fn refresh_tail_ptr(&mut self) {
        self.tail_ptr = if self.tail_group < self.groups.len() {
            NonNull::from(&mut self.groups[self.tail_group])
        } else {
            NonNull::dangling()
        };
    }
}

impl<'a, T> IntoIterator for &'a Colony<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Colony<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[cfg(test)]
mod tests;
