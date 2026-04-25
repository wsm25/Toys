use std::mem::MaybeUninit;

pub(crate) const GROUP_LEN: usize = 64;
pub(crate) type GroupMask = u64;
const FULL_MASK: GroupMask = GroupMask::MAX;

pub(crate) struct Group<T> {
    pub(crate) occupied: GroupMask,
    pub(crate) in_free_list: bool,
    generations: [u32; GROUP_LEN],
    pub(crate) slots: [MaybeUninit<T>; GROUP_LEN],
}

impl<T> Group<T> {
    #[inline]
    pub(crate) const fn new() -> Self {
        Self {
            occupied: 0,
            in_free_list: false,
            generations: [0; GROUP_LEN],
            slots: [const { MaybeUninit::uninit() }; GROUP_LEN],
        }
    }

    #[inline]
    pub(crate) const fn is_full(&self) -> bool {
        self.occupied == FULL_MASK
    }

    #[inline]
    pub(crate) fn insert_at(&mut self, slot: usize, value: T) -> u32 {
        debug_assert!(slot < GROUP_LEN);
        debug_assert!(!self.slot_occupied(slot));
        let generation = self.generations[slot];
        self.occupied |= Self::slot_mask(slot);
        self.slots[slot].write(value); // SAFETY: slot is empty
        generation
    }

    #[inline]
    pub(crate) fn first_free_slot_before(&self, limit: usize) -> Option<usize> {
        debug_assert!(limit <= GROUP_LEN);
        let initialized = if limit == GROUP_LEN {
            FULL_MASK
        } else {
            (1 << limit) - 1
        };
        Self::first_one_bit(!self.occupied & initialized)
    }

    #[inline]
    pub(crate) fn is_full_before(&self, limit: usize) -> bool {
        debug_assert!(limit <= GROUP_LEN);
        let initialized = if limit == GROUP_LEN {
            FULL_MASK
        } else {
            (1 << limit) - 1
        };
        (self.occupied & initialized) == initialized
    }

    #[inline]
    pub(crate) const fn get(&self, slot: usize, generation: u32) -> Option<&T> {
        if !self.slot_matches(slot, generation) {
            return None;
        }
        Some(unsafe { self.slots[slot].assume_init_ref() })
    }

    #[inline]
    pub(crate) unsafe fn get_unchecked(&self, slot: usize) -> &T {
        debug_assert!(slot < GROUP_LEN);
        unsafe { self.slots.get_unchecked(slot).assume_init_ref() }
    }

    #[inline]
    pub(crate) const fn get_mut(&mut self, slot: usize, generation: u32) -> Option<&mut T> {
        if !self.slot_matches(slot, generation) {
            return None;
        }
        Some(unsafe { self.slots[slot].assume_init_mut() })
    }

    #[inline]
    pub(crate) const fn pop(&mut self, slot: usize, generation: u32) -> Option<T> {
        if !self.slot_matches(slot, generation) {
            return None;
        }
        self.occupied ^= Self::slot_mask(slot);
        self.generations[slot] = generation.wrapping_add(1);
        Some(unsafe { self.slots[slot].assume_init_read() })
    }

    #[inline]
    // occupied and generation matches
    const fn slot_matches(&self, slot: usize, generation: u32) -> bool {
        debug_assert!(slot < GROUP_LEN);
        self.generations[slot] == generation && self.slot_occupied(slot)
    }

    #[inline]
    const fn slot_occupied(&self, slot: usize) -> bool {
        (self.occupied & Self::slot_mask(slot)) != 0
    }

    #[inline]
    const fn first_one_bit(mask: GroupMask) -> Option<usize> {
        if mask == 0 {
            None
        } else {
            Some(mask.trailing_zeros() as usize)
        }
    }

    #[inline]
    const fn slot_mask(slot: usize) -> GroupMask {
        1 << slot
    }

    pub(crate) fn clear(&mut self) {
        while self.occupied != 0 {
            let slot = self.occupied.trailing_zeros() as usize;
            let mask = 1 << slot;
            // drop guard
            self.occupied ^= mask;
            self.generations[slot] = self.generations[slot].wrapping_add(1);
            // this may panic
            unsafe { self.slots[slot].assume_init_drop() };
        }
    }
}

impl<T> Drop for Group<T> {
    fn drop(&mut self) {
        self.clear();
    }
}
