//! The fields inside a payload, for every job with fields: each length or count goes before its bytes, little-endian, in
//! the width its cap needs. A writer runs its fields through `Size`, which checks them, before `Put` writes any byte.

use crate::error::IpcError;
use crate::frame::{read_array, read_exact, read_tail, read_wiped};
use std::io::{Read, Write};
use zeroize::Zeroizing;

/// Bytes in a one-byte field: a code, a yes or no, or a length or count that one byte holds.
pub(crate) const ONE: usize = size_of::<u8>();
/// Bytes in a two-byte number, length or count.
pub(crate) const TWO: usize = size_of::<u16>();
/// Bytes in a four-byte number or length, the widest length.
pub(crate) const FOUR: usize = size_of::<u32>();
/// Bytes in an eight-byte number.
pub(crate) const EIGHT: usize = size_of::<u64>();

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
    pub(crate) const fn new(width: usize, min: usize, max: usize) -> Self {
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

/// Where a frame's fields go: `Size`, which checks and counts them, or `Put`, which writes them.
pub(crate) trait Sink {
    /// Takes fixed bytes as they are: a code, a number or a fingerprint.
    fn raw(&mut self, bytes: &[u8]) -> Result<(), IpcError>;
    /// Takes the count `n` before a run of items, refusing one outside `spec`.
    fn count(&mut self, spec: Spec, n: usize) -> Result<(), IpcError>;
    /// Takes `bytes` after their length, refusing a length outside `spec`.
    fn field(&mut self, spec: Spec, bytes: &[u8]) -> Result<(), IpcError>;

    /// Takes one byte as it is.
    fn byte(&mut self, b: u8) -> Result<(), IpcError> {
        self.raw(&[b])
    }

    /// Takes a two-byte number.
    fn u16(&mut self, n: u16) -> Result<(), IpcError> {
        self.raw(&n.to_le_bytes())
    }

    /// Takes a four-byte number.
    fn u32(&mut self, n: u32) -> Result<(), IpcError> {
        self.raw(&n.to_le_bytes())
    }

    /// Takes an eight-byte number.
    fn u64(&mut self, n: u64) -> Result<(), IpcError> {
        self.raw(&n.to_le_bytes())
    }
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
    fn raw(&mut self, bytes: &[u8]) -> Result<(), IpcError> {
        self.add(bytes.len())
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
    fn raw(&mut self, bytes: &[u8]) -> Result<(), IpcError> {
        self.0.write_all(bytes)?;
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

    /// Reads exactly `N` fixed bytes.
    pub(crate) fn array<const N: usize>(&mut self) -> Result<[u8; N], IpcError> {
        self.take(N)?;
        read_array::<N, _>(self.r)
    }

    /// Reads one byte.
    pub(crate) fn byte(&mut self) -> Result<u8, IpcError> {
        let [b] = self.array::<ONE>()?;
        Ok(b)
    }

    /// Reads a yes or no, `BadField` for any byte but 0 or 1.
    pub(crate) fn flag(&mut self) -> Result<bool, IpcError> {
        match self.byte()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(IpcError::BadField),
        }
    }

    /// Reads a two-byte number.
    pub(crate) fn u16(&mut self) -> Result<u16, IpcError> {
        Ok(u16::from_le_bytes(self.array()?))
    }

    /// Reads a four-byte number.
    pub(crate) fn u32(&mut self) -> Result<u32, IpcError> {
        Ok(u32::from_le_bytes(self.array()?))
    }

    /// Reads an eight-byte number.
    pub(crate) fn u64(&mut self) -> Result<u64, IpcError> {
        Ok(u64::from_le_bytes(self.array()?))
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
