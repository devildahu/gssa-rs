//! Manage free/unfree space on a 1d line.
//!
//! A [`Block`] represents something take takes [`Block::size`] space or a gap
//! in space.
//!
//! [`Blocks`] acts like a heap, where you can add and remove things.
#![cfg_attr(not(feature = "test"), no_std)]
#![warn(clippy::pedantic, clippy::nursery)]
#![feature(const_mut_refs)]

#[cfg(all(test, target = "thumbv4t-none-eabi"))]
compile_error!("Tests cannot be ran in thumbv4t mode, you should use the host's architecture");

use arrayvec::ArrayVec;

struct Gap {
    index: usize,
    gap_size: u16,
}

/// A `Block` represents something take takes [`Block::size`].
/// Each occupied `Block` is identified with `Id`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Block<Id: PartialEq> {
    /// A void of size `u16` left from something that was removed.
    Gap(u16),
    /// Something identified by `Id` that takes `u16` space.
    Full(Id, u16),
}
impl<Id: PartialEq> Block<Id> {
    const fn size(&self) -> u16 {
        let (Self::Gap(size) | Self::Full(_, size)) = self;
        *size
    }
    #[allow(clippy::missing_const_for_fn)]
    fn has_id(&self, id: &Id) -> bool {
        matches!(self, Self::Full(self_id, _) if self_id == id)
    }
}
/// `Blocks` manage resource allocation on a 1D line.
///
/// Resources all have a unique `Id`, and attempts to allocate twice the same
/// `Id` will result in the existing allocation being returned.
#[derive(Debug)]
pub struct Blocks<Id: PartialEq, const MAX_BLOCKS: usize> {
    blocks: ArrayVec<Block<Id>, MAX_BLOCKS>,
    /// Maximum size of blocks.
    full_size: u16,
}
impl<Id, const MAX_BLOCKS: usize> Blocks<Id, MAX_BLOCKS>
where
    Id: PartialEq + Copy,
{
    /// Create a new empty `Blocks`.
    #[must_use]
    pub const fn new(full_size: u16) -> Self {
        Self { blocks: ArrayVec::new_const(), full_size }
    }
    // TODO: we could minimize creating small gaps if we check all gaps and
    // prefer gaps of requested size instead of picking up the first large
    // enough gap.
    /// This assumes, `Self` is [cleaned up](Blocks::cleanup).
    fn first_gap_of_size(&self, size: u16) -> Option<Gap> {
        self.blocks
            .iter()
            .enumerate()
            .find(|(_, block)| matches!(block, Block::Gap(gap) if gap >= &size))
            .map(|(index, block)| Gap { index, gap_size: block.size() })
    }
    /// Inserts `id` into blocks, returning the blocks index.
    ///
    /// Returns `None` if:
    /// - trying to add at the end and there already is `MAX_BLOCKS` blocks.
    /// - trying to partially fill an existing gap and there already is `MAX_BLOCKS` blocks.
    fn replace_gap(&mut self, id: Id, size: u16) -> Option<usize> {
        let to_insert = Block::Full(id, size);
        match self.first_gap_of_size(size) {
            Some(Gap { index, gap_size }) if gap_size > size => {
                // crate::debug!("Found a gap of size {gap_size}, inserting");
                // SAFETY: `index` is always within `blocks` because it results
                // from `first_gap_of_size` which only returns indices from
                // existing elements.
                let to_update = unsafe { self.blocks.get_unchecked_mut(index) };
                *to_update = to_insert;
                let gap = Block::Gap(gap_size - size);
                self.blocks.try_insert(index + 1, gap).ok()?;
                self.cleanup();
                Some(index)
            }
            Some(Gap { index, .. }) => {
                // crate::debug!("Found a gap of exact same size! inserting");
                // SAFETY: `index` is always within `blocks` because it results
                // from `first_gap_of_size` which only returns indices from
                // existing elements.
                let to_update = unsafe { self.blocks.get_unchecked_mut(index) };
                *to_update = to_insert;
                Some(index)
            }
            None => {
                self.blocks.try_push(to_insert).ok()?;
                Some(self.blocks.len() - 1)
            }
        }
    }
    /// Insert `Id` of given `size`, returning the offset from start of insert position.
    ///
    /// If there is not enough room to fit `size`, then return `None`.
    pub fn insert_sized(&mut self, id: Id, size: u16) -> Option<u16> {
        let already_existing = self.offset_of(id);
        if already_existing.is_some() {
            return already_existing;
        }
        // crate::debug!("Try to insert an object of size {size}");
        let insert_index = self.replace_gap(id, size)?;
        let offset = self.blocks.iter().take(insert_index).map(Block::size).sum();
        (offset + size <= self.full_size).then_some(offset)
    }
    // /// Remove `id`, not merging adjascent gaps.
    // ///
    // /// You are free to call `remove_no_cleanup` consecutively as many times as
    // /// you want, but before calling [`Self::insert_sized`], you should then call
    // /// [`Self::cleanup`].
    // pub fn remove_no_cleanup(&mut self, id: Id) {
    //     if let Some(to_remove) = self.blocks.iter_mut().find(|block| block.has_id(&id)) {
    //         *to_remove = Block::Gap(to_remove.size());
    //     }
    // }
    /// Remove `id`, if `Id` was allocated, return `true`
    pub fn remove(&mut self, id: Id) -> bool {
        if let Some(to_remove) = self.blocks.iter_mut().find(|block| block.has_id(&id)) {
            *to_remove = Block::Gap(to_remove.size());
        } else {
            return false;
        }
        // TODO: something more performant
        self.cleanup();
        true
    }
    /// Whether one of the blocks has given id.
    fn offset_of(&self, id: Id) -> Option<u16> {
        let mut id_found = false;
        let is_not_id = |elem: &&Block<Id>| {
            let is_id = matches!(elem, &Block::Full(elem_id, _) if elem_id == &id);
            if is_id {
                id_found = true;
            }
            !is_id
        };
        let computed = self
            .blocks
            .iter()
            .take_while(is_not_id)
            .fold(0, |acc, elem| acc + elem.size());
        id_found.then_some(computed)
    }
    /// Replace an existing allocation, changing it's id from `old` to `new`.
    ///
    /// Also checks that `new` and `old` both have an allocation of given `size`,
    /// returns `None` if either `old` is not allocated or not of given `size`.
    pub fn replace_id(&mut self, old: Id, new: Id, size: u16) -> Option<u16> {
        let expected = Block::Full(old, size);
        let mut offset = 0;
        let find = |elem: &&mut Block<Id>| {
            let is_elem = *elem == &expected;
            if !is_elem {
                offset += elem.size();
            }
            is_elem
        };
        let found = self.blocks.iter_mut().find(find)?;
        if let Block::Full(old, _) = found {
            *old = new;
        }
        Some(offset)
    }
    /// Merges adjacent gaps and remove [`Block::Gap`] at the end of `self`.
    ///
    /// If using [`Self::remove_no_cleanup`], you must call this method before
    /// calling [`Self::insert_sized`].
    ///
    /// If you never use [`Self::remove_no_cleanup`] this method does nothing.
    pub fn cleanup(&mut self) {
        // Since we don't expect to run `cleanup` much often, we allow ourselves
        // to be less efficient than ideal. A perfect world would allow use to
        // either:
        // 1. Have a `filter` or `filter_map` or `retain` that allow to hold of
        //    a previous &mut Gap.0, keep the first Gap of a series of gaps,
        //    add to this &mut Gap.0 each new gaps in the series and remove
        //    all but the first gap of series of gaps
        // 2. Peek forward, keep track of an interal state of gap size,
        //    incremented at each conscecutive gaps, reset when meeting a full
        //    block. Remove all but the last gap in a series, updating the last
        //    gap to the internal increment.
        //
        // In this implementation, it is split in two loops because I found it
        // impossible to reconciliate both methods with the rust borrow system.
        // The first loop redefines each Gap's `size` as the sum of all
        // subsequent adjacent gap's size. This implies that the first gap has
        // the total size of the series of adjacent gaps.
        let mut cur_gap = 0;
        self.blocks.iter_mut().rev().for_each(|elem| match elem {
            Block::Full(..) => cur_gap = 0,
            Block::Gap(size) => {
                cur_gap += *size;
                *size = cur_gap;
            }
        });
        // Then it removes all Gaps that are not the first in a series.
        let mut previous_was_gap = false;
        self.blocks.retain(|elem| {
            let current_is_gap = matches!(elem, Block::Gap(_));
            let should_remove = previous_was_gap && current_is_gap;
            previous_was_gap = current_is_gap;
            !should_remove
        });
        if matches!(self.blocks.last(), Some(Block::Gap(_))) {
            // self.blocks.pop().expect("We just tested last is Some");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gap() {
        let mut blocks = Blocks::<u8, 128>::new(128);
        assert_eq!(Some(0), blocks.insert_sized(1, 3));
        assert_eq!(Some(3), blocks.insert_sized(2, 20));
    }
    #[test]
    fn test_reinsertion() {
        let mut blocks = Blocks::<u8, 128>::new(128);
        blocks.insert_sized(1, 3);
        blocks.insert_sized(2, 2);
        blocks.insert_sized(3, 8);
        blocks.remove(2);
        assert_eq!(Some(3), blocks.insert_sized(4, 1));
        assert_eq!(Some(3 + 1), blocks.insert_sized(5, 1));
    }
    #[test]
    fn test_reinsertion_merging() {
        let mut blocks = Blocks::<u8, 128>::new(128);
        blocks.insert_sized(1, 3);
        blocks.insert_sized(2, 1);
        blocks.insert_sized(3, 1);
        blocks.insert_sized(4, 8);
        blocks.remove(2);
        blocks.remove(3);
        assert_eq!(Some(3), blocks.insert_sized(5, 2));
    }
    #[test]
    fn test_fat_block() {
        let mut blocks = Blocks::<u8, 128>::new(128);
        blocks.insert_sized(1, 1);
        blocks.insert_sized(2, 1);
        blocks.insert_sized(3, 8);
        blocks.remove(2);
        assert_eq!(Some(1 + 1 + 8), blocks.insert_sized(4, 3));
    }
    #[test]
    fn test_cleanup_single_block_end() {
        let mut blocks = Blocks::<u8, 128>::new(128);
        blocks.insert_sized(1, 1);
        blocks.insert_sized(2, 1);
        blocks.remove(2);
        assert_eq!(Some(1), blocks.insert_sized(3, 10));
    }
    #[test]
    fn test_cleanup_multiple_block_end() {
        let mut blocks = Blocks::<u8, 128>::new(128);
        blocks.insert_sized(1, 1);
        blocks.insert_sized(2, 1);
        blocks.insert_sized(3, 1);
        blocks.remove(2);
        blocks.remove(3);
        assert_eq!(Some(1), blocks.insert_sized(4, 1));
    }
}
