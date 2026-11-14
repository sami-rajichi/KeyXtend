//! Sound in buffers wiped when dropped or cleared. A buffer grows by copying into a fresh one and wiping the old, so a
//! reallocation never leaves an unwiped copy behind; a failed growth is `Memory`, never an abort.

use kx_ipc::VoiceFault;
use std::ops::Deref;
use zeroize::{Zeroize, Zeroizing};

/// Samples or bytes wiped when dropped or cleared.
pub struct Buf<T: Zeroize> {
    items: Zeroizing<Vec<T>>,
    /// The most items it grows room for in one step, unless more are asked for.
    limit: usize,
}

impl<T: Copy + Zeroize> Buf<T> {
    /// An empty buffer that grows without a limit.
    pub fn new() -> Self {
        Self::with_limit(usize::MAX)
    }

    /// An empty buffer whose doubling stops at `limit` items, for a recording capped at that length.
    pub fn with_limit(limit: usize) -> Self {
        Self {
            items: Zeroizing::new(Vec::new()),
            limit,
        }
    }

    /// A buffer holding exactly `items`, made in one allocation.
    ///
    /// # Errors
    /// `Memory` when the room cannot be had.
    pub fn collect(items: impl ExactSizeIterator<Item = T>) -> Result<Self, VoiceFault> {
        let mut buf = Self::new();
        buf.reserve(items.len())?;
        for item in items {
            buf.push(item)?;
        }
        Ok(buf)
    }

    /// Makes room for `more` items, so the next pushes do not grow it.
    ///
    /// # Errors
    /// `Memory` when the room cannot be had.
    pub fn reserve(&mut self, more: usize) -> Result<(), VoiceFault> {
        let need = self.items.len().checked_add(more);
        let need = need.ok_or(VoiceFault::Memory)?;
        if need > self.items.capacity() {
            self.grow(need)?;
        }
        Ok(())
    }

    /// Adds `more` at the end.
    ///
    /// # Errors
    /// `Memory` when it cannot grow.
    pub fn extend(&mut self, more: &[T]) -> Result<(), VoiceFault> {
        self.reserve(more.len())?;
        self.items.extend_from_slice(more);
        Ok(())
    }

    /// Adds one item at the end.
    ///
    /// # Errors
    /// `Memory` when it cannot grow.
    pub fn push(&mut self, item: T) -> Result<(), VoiceFault> {
        self.reserve(1)?;
        self.items.push(item);
        Ok(())
    }

    /// Drops the first `n` items, moving the rest to the front; the freed tail is wiped with the buffer.
    pub fn drain_front(&mut self, n: usize) {
        let n = n.min(self.items.len());
        self.items.drain(..n);
    }

    /// Wipes every item and empties the buffer, keeping its room.
    pub fn clear(&mut self) {
        self.items.zeroize();
    }

    /// Moves the items into a buffer of at least `need` items, doubling up to the limit, and wipes the old one.
    fn grow(&mut self, need: usize) -> Result<(), VoiceFault> {
        let double = self.items.capacity().saturating_mul(2).min(self.limit);
        let mut next = Vec::new();
        next.try_reserve_exact(need.max(double))
            .map_err(|_| VoiceFault::Memory)?;
        next.extend_from_slice(&self.items);
        self.items = Zeroizing::new(next);
        Ok(())
    }
}

impl<T: Copy + Zeroize> Default for Buf<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Zeroize> Deref for Buf<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        &self.items
    }
}

#[cfg(test)]
mod tests;
