//! The vault frames' fields: each length or count goes before its bytes, little-endian, in the width its cap needs.
//! A writer runs its fields through `Size`, which checks them, before `Put` writes any byte.

use crate::error::IpcError;
use crate::frame::{read_array, read_exact, read_tail, read_wiped};
use crate::limits::{
    MAX_BOX_BYTES, MAX_ENTRIES, MAX_ENTRIES_LEN, MAX_FIELD_BYTES, MAX_FIELDS, MAX_KEEPASS_BYTES,
    MAX_LOCKED_BYTES, MAX_PART_NAME, MAX_PARTS, MAX_PASSWORD_BYTES, MAX_RECIPIENT_BYTES,
};
use std::io::{Read, Write};
use zeroize::Zeroizing;

/// Bytes in a one-byte field: a work factor, a yes or no, or a length or count that one byte holds.
pub(crate) const ONE: usize = size_of::<u8>();
/// Bytes in a two-byte length or count.
const TWO: usize = size_of::<u16>();
/// Bytes in a four-byte length, the widest.
const FOUR: usize = size_of::<u32>();

/// A field: the width of the length or count before it, and the range that length or count may take.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Spec {
    /// Bytes in its length or count.
    pub width: usize,
    /// The shortest length or fewest items.
    pub min: usize,
    /// The longest length or most items.
    pub max: usize,
}

impl Spec {
    /// A field whose `max` fits `width` bytes; anything wider fails the build.
    const fn new(width: usize, min: usize, max: usize) -> Self {
        assert!(width <= FOUR && (max as u64) < 1 << (8 * width as u64) && min <= max);
        Self { width, min, max }
    }

    /// Its fewest bytes on the wire, the length included.
    pub(crate) const fn least(self) -> usize {
        self.width + self.min
    }

    /// Its most bytes on the wire, the length included.
    pub(crate) const fn most(self) -> usize {
        self.width + self.max
    }

    /// Refuses `n` past `max` as `TooLong` and short of `min` as `Inconsistent`.
    fn check(self, n: usize) -> Result<(), IpcError> {
        if n > self.max {
            Err(IpcError::TooLong)
        } else if n < self.min {
            Err(IpcError::Inconsistent)
        } else {
            Ok(())
        }
    }
}

/// A password: UTF-8, possibly empty.
pub(crate) const PASSWORD: Spec = Spec::new(TWO, 0, MAX_PASSWORD_BYTES);
/// An open padlock: an `age` recipient's text.
pub(crate) const RECIPIENT: Spec = Spec::new(ONE, 1, MAX_RECIPIENT_BYTES);
/// A locked identity.
pub(crate) const LOCKED: Spec = Spec::new(TWO, 1, MAX_LOCKED_BYTES);
/// A box's bytes.
pub(crate) const BYTES: Spec = Spec::new(FOUR, 1, MAX_BOX_BYTES);
/// A `KeePass` file's bytes.
pub(crate) const KEEPASS: Spec = Spec::new(FOUR, 1, MAX_KEEPASS_BYTES);
/// The count of a box's parts; a box holds at least one.
pub(crate) const PARTS: Spec = Spec::new(ONE, 1, MAX_PARTS);
/// A part's name, one of `BOX_PARTS`.
pub(crate) const NAME: Spec = Spec::new(ONE, 1, MAX_PART_NAME);
/// A part's plain bytes; all the parts together hold `MAX_BOX_BYTES` at most.
pub(crate) const PART: Spec = Spec::new(FOUR, 0, MAX_BOX_BYTES);
/// The count of `KeePass` entries.
pub(crate) const ENTRIES: Spec = Spec::new(TWO, 0, MAX_ENTRIES);
/// One field of an entry, as UTF-8.
pub(crate) const FIELD: Spec = Spec::new(TWO, 0, MAX_FIELD_BYTES);
/// The count of an entry's custom fields.
pub(crate) const CUSTOM: Spec = Spec::new(ONE, 0, MAX_FIELDS);
/// A count of what a `KeePass` import dropped.
pub(crate) const DROPPED: Spec = Spec::new(FOUR, 0, u32::MAX as usize);

/// A padlock: the recipient, then the locked identity.
pub(crate) const PADLOCK_LEAST: usize = RECIPIENT.least() + LOCKED.least();
pub(crate) const PADLOCK_MOST: usize = RECIPIENT.most() + LOCKED.most();
/// A padlock and a part list: each known part's name and length, and all the parts' bytes together.
pub(crate) const SEAL_LEAST: usize = PADLOCK_LEAST + PARTS.width + NAME.least() + PART.least();
pub(crate) const SEAL_MOST: usize =
    PADLOCK_MOST + PARTS.width + MAX_PARTS * (NAME.most() + PART.width) + MAX_BOX_BYTES;
/// A work factor, then a password.
pub(crate) const MAKE_LEAST: usize = ONE + PASSWORD.least();
pub(crate) const MAKE_MOST: usize = ONE + PASSWORD.most();
/// A password, then a box's bytes.
pub(crate) const OPEN_LEAST: usize = PASSWORD.least() + BYTES.least();
pub(crate) const OPEN_MOST: usize = PASSWORD.most() + BYTES.most();
/// A password, then a `KeePass` file's bytes.
pub(crate) const KEEPASS_LEAST: usize = PASSWORD.least() + KEEPASS.least();
pub(crate) const KEEPASS_MOST: usize = PASSWORD.most() + KEEPASS.most();
/// A password, then a locked identity.
pub(crate) const CHECK_LEAST: usize = PASSWORD.least() + LOCKED.least();
pub(crate) const CHECK_MOST: usize = PASSWORD.most() + LOCKED.most();
/// A `KeePass` reply: the dropped entries and fields, then the count and the entries, within `MAX_ENTRIES_LEN`.
pub(crate) const ENTRIES_LEAST: usize = 2 * DROPPED.width + ENTRIES.width;
pub(crate) const ENTRIES_MOST: usize = MAX_ENTRIES_LEN;

// Every vault payload fits the header's `u32` length.
const _: () = assert!(SEAL_MOST <= u32::MAX as usize);
const _: () = assert!(OPEN_MOST <= u32::MAX as usize);

/// Where a frame's fields go: `Size`, which checks and counts them, or `Put`, which writes them.
pub(crate) trait Sink {
    /// Takes one byte as it is.
    fn byte(&mut self, b: u8) -> Result<(), IpcError>;
    /// Takes the count `n` before a run of items, refusing one outside `spec`.
    fn count(&mut self, spec: Spec, n: usize) -> Result<(), IpcError>;
    /// Takes `bytes` after their length, refusing a length outside `spec`.
    fn field(&mut self, spec: Spec, bytes: &[u8]) -> Result<(), IpcError>;
}

/// Counts a payload's bytes, refusing what a reader would.
#[derive(Default)]
pub(crate) struct Size(pub usize);

impl Size {
    fn add(&mut self, n: usize) -> Result<(), IpcError> {
        self.0 = self.0.checked_add(n).ok_or(IpcError::TooLong)?;
        Ok(())
    }
}

impl Sink for Size {
    fn byte(&mut self, _: u8) -> Result<(), IpcError> {
        self.add(ONE)
    }

    fn count(&mut self, spec: Spec, n: usize) -> Result<(), IpcError> {
        spec.check(n)?;
        self.add(spec.width)
    }

    fn field(&mut self, spec: Spec, bytes: &[u8]) -> Result<(), IpcError> {
        spec.check(bytes.len())?;
        self.add(spec.width)?;
        self.add(bytes.len())
    }
}

/// Writes a payload's fields once `Size` has checked them.
pub(crate) struct Put<'w, W: ?Sized>(pub &'w mut W);

impl<W: Write + ?Sized> Put<'_, W> {
    /// Writes `n` in `width` bytes.
    fn prefix(&mut self, width: usize, n: usize) -> Result<(), IpcError> {
        let raw = u32::try_from(n)
            .map_err(|_| IpcError::TooLong)?
            .to_le_bytes();
        self.0
            .write_all(raw.get(..width).ok_or(IpcError::TooLong)?)?;
        Ok(())
    }
}

impl<W: Write + ?Sized> Sink for Put<'_, W> {
    fn byte(&mut self, b: u8) -> Result<(), IpcError> {
        self.0.write_all(&[b])?;
        Ok(())
    }

    fn count(&mut self, spec: Spec, n: usize) -> Result<(), IpcError> {
        self.prefix(spec.width, n)
    }

    fn field(&mut self, spec: Spec, bytes: &[u8]) -> Result<(), IpcError> {
        self.prefix(spec.width, bytes.len())?;
        self.0.write_all(bytes)?;
        Ok(())
    }
}

/// A payload being read: every length is checked against its cap and against the bytes left before any is read.
pub(crate) struct Payload<'r, R: ?Sized> {
    r: &'r mut R,
    left: usize,
}

impl<'r, R: Read + ?Sized> Payload<'r, R> {
    /// The `len` payload bytes after a checked header.
    pub(crate) fn new(r: &'r mut R, len: usize) -> Self {
        Self { r, left: len }
    }

    /// Counts `n` bytes as read, `Inconsistent` past the payload's end.
    fn take(&mut self, n: usize) -> Result<(), IpcError> {
        self.left = self.left.checked_sub(n).ok_or(IpcError::Inconsistent)?;
        Ok(())
    }

    /// Reads one byte.
    pub(crate) fn byte(&mut self) -> Result<u8, IpcError> {
        self.take(ONE)?;
        let [b] = read_array::<ONE, _>(self.r)?;
        Ok(b)
    }

    /// Reads a length or count and checks it against `spec`.
    pub(crate) fn count(&mut self, spec: Spec) -> Result<usize, IpcError> {
        self.take(spec.width)?;
        let mut raw = [0; FOUR];
        read_exact(
            self.r,
            raw.get_mut(..spec.width).ok_or(IpcError::Inconsistent)?,
        )?;
        let n = usize::try_from(u32::from_le_bytes(raw)).map_err(|_| IpcError::TooLong)?;
        spec.check(n)?;
        Ok(n)
    }

    /// Reads a field of `spec`.
    pub(crate) fn bytes(&mut self, spec: Spec) -> Result<Vec<u8>, IpcError> {
        let n = self.count(spec)?;
        self.take(n)?;
        read_tail(self.r, n)
    }

    /// Reads a UTF-8 field of `spec`, `BadField` when it is not text.
    pub(crate) fn text(&mut self, spec: Spec) -> Result<String, IpcError> {
        String::from_utf8(self.bytes(spec)?).map_err(|_| IpcError::BadField)
    }

    /// Reads `n` secret bytes whose length is already checked; they are wiped when dropped.
    pub(crate) fn wiped(&mut self, n: usize) -> Result<Zeroizing<Vec<u8>>, IpcError> {
        self.take(n)?;
        read_wiped(self.r, n)
    }

    /// Reads a secret UTF-8 field of `spec`; the bytes are wiped whether or not they are text.
    pub(crate) fn secret(&mut self, spec: Spec) -> Result<Zeroizing<String>, IpcError> {
        let n = self.count(spec)?;
        let mut raw = self.wiped(n)?;
        match String::from_utf8(std::mem::take(&mut *raw)) {
            Ok(text) => Ok(Zeroizing::new(text)),
            Err(e) => {
                drop(Zeroizing::new(e.into_bytes()));
                Err(IpcError::BadField)
            }
        }
    }

    /// Ends the payload, `Inconsistent` when bytes are left.
    pub(crate) fn end(self) -> Result<(), IpcError> {
        if self.left == 0 {
            Ok(())
        } else {
            Err(IpcError::Inconsistent)
        }
    }
}
