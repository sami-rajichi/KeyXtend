//! The store's byte primitives: little-endian numbers, flags, and texts and byte runs after a u64 length. Every read
//! checks what is left before it slices, so random bytes give `Bad`, never a panic or a large allocation.

use std::fmt;
use std::path::PathBuf;
use std::str;

/// Bytes that do not read as this version's form.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Bad;

/// Writes a byte form front to back.
#[derive(Default)]
pub(crate) struct Writer(Vec<u8>);

impl Writer {
    /// A u8.
    pub(crate) fn u8(&mut self, n: u8) {
        self.0.push(n);
    }

    /// A little-endian u16.
    pub(crate) fn u16(&mut self, n: u16) {
        self.0.extend(n.to_le_bytes());
    }

    /// A little-endian u32.
    pub(crate) fn u32(&mut self, n: u32) {
        self.0.extend(n.to_le_bytes());
    }

    /// A little-endian u64.
    pub(crate) fn u64(&mut self, n: u64) {
        self.0.extend(n.to_le_bytes());
    }

    /// A little-endian i64.
    pub(crate) fn i64(&mut self, n: i64) {
        self.0.extend(n.to_le_bytes());
    }

    /// A little-endian i16.
    pub(crate) fn i16(&mut self, n: i16) {
        self.0.extend(n.to_le_bytes());
    }

    /// A flag as one byte, 0 or 1.
    pub(crate) fn flag(&mut self, on: bool) {
        self.u8(u8::from(on));
    }

    /// A count or a length, as a u64.
    pub(crate) fn count(&mut self, n: usize) {
        self.u64(u64::try_from(n).unwrap_or(u64::MAX));
    }

    /// `bytes` after their length.
    pub(crate) fn bytes(&mut self, bytes: &[u8]) {
        self.count(bytes.len());
        self.0.extend_from_slice(bytes);
    }

    /// `text`'s UTF-8 after its length.
    pub(crate) fn text(&mut self, text: &str) {
        self.bytes(text.as_bytes());
    }

    /// `paths` after their count, each as UTF-8; a name that is not Unicode keeps a replacement character.
    pub(crate) fn paths(&mut self, paths: &[PathBuf]) {
        self.count(paths.len());
        for path in paths {
            self.text(&path.to_string_lossy());
        }
    }

    /// The bytes written.
    pub(crate) fn done(self) -> Vec<u8> {
        self.0
    }
}

/// Reads a byte form front to back.
pub(crate) struct Reader<'b>(&'b [u8]);

/// Tells the length written, never the bytes.
impl fmt::Debug for Writer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Writer")
            .field("len", &self.0.len())
            .finish()
    }
}

/// Tells the length left, never the bytes.
impl fmt::Debug for Reader<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Reader")
            .field("left", &self.0.len())
            .finish()
    }
}

impl<'b> Reader<'b> {
    /// A reader at the start of `bytes`.
    pub(crate) fn new(bytes: &'b [u8]) -> Self {
        Self(bytes)
    }

    /// The next `n` bytes.
    pub(crate) fn take(&mut self, n: usize) -> Result<&'b [u8], Bad> {
        let (head, tail) = self.0.split_at_checked(n).ok_or(Bad)?;
        self.0 = tail;
        Ok(head)
    }

    /// The next `N` bytes.
    fn array<const N: usize>(&mut self) -> Result<[u8; N], Bad> {
        self.take(N)?.try_into().map_err(|_| Bad)
    }

    /// The next u8.
    pub(crate) fn u8(&mut self) -> Result<u8, Bad> {
        self.array().map(u8::from_le_bytes)
    }

    /// The next little-endian u16.
    pub(crate) fn u16(&mut self) -> Result<u16, Bad> {
        self.array().map(u16::from_le_bytes)
    }

    /// The next little-endian u32.
    pub(crate) fn u32(&mut self) -> Result<u32, Bad> {
        self.array().map(u32::from_le_bytes)
    }

    /// The next little-endian u64.
    pub(crate) fn u64(&mut self) -> Result<u64, Bad> {
        self.array().map(u64::from_le_bytes)
    }

    /// The next little-endian i64.
    pub(crate) fn i64(&mut self) -> Result<i64, Bad> {
        self.array().map(i64::from_le_bytes)
    }

    /// The next little-endian i16.
    pub(crate) fn i16(&mut self) -> Result<i16, Bad> {
        self.array().map(i16::from_le_bytes)
    }

    /// The next flag; a byte other than 0 or 1 is `Bad`.
    pub(crate) fn flag(&mut self) -> Result<bool, Bad> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Bad),
        }
    }

    /// The next count or length; never more than the bytes left, as each counted item takes a byte at least.
    pub(crate) fn count(&mut self) -> Result<usize, Bad> {
        let n = usize::try_from(self.u64()?).map_err(|_| Bad)?;
        if n > self.0.len() { Err(Bad) } else { Ok(n) }
    }

    /// The next bytes after their length.
    pub(crate) fn bytes(&mut self) -> Result<&'b [u8], Bad> {
        let n = self.count()?;
        self.take(n)
    }

    /// The next UTF-8 text after its length.
    pub(crate) fn text(&mut self) -> Result<&'b str, Bad> {
        str::from_utf8(self.bytes()?).map_err(|_| Bad)
    }

    /// The next paths after their count.
    pub(crate) fn paths(&mut self) -> Result<Vec<PathBuf>, Bad> {
        let n = self.count()?;
        (0..n).map(|_| self.text().map(PathBuf::from)).collect()
    }

    /// True when every byte was read.
    pub(crate) fn done(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_flags_and_texts_round_trip() {
        let mut w = Writer::default();
        w.u8(7);
        w.u16(300);
        w.u32(70_000);
        w.u64(u64::MAX);
        w.i64(-5);
        w.i16(-60);
        w.flag(true);
        w.text("école");
        w.bytes(&[1, 2]);
        let bytes = w.done();
        let mut r = Reader::new(&bytes);
        assert_eq!((r.u8(), r.u16(), r.u32()), (Ok(7), Ok(300), Ok(70_000)));
        assert_eq!((r.u64(), r.i64(), r.i16()), (Ok(u64::MAX), Ok(-5), Ok(-60)));
        assert_eq!((r.flag(), r.text()), (Ok(true), Ok("école")));
        assert_eq!(r.bytes(), Ok(&[1, 2][..]));
        assert!(r.done());
        assert_eq!(r.u8(), Err(Bad));
    }

    #[test]
    fn a_length_past_the_end_or_a_bad_flag_or_text_is_bad() {
        let mut w = Writer::default();
        w.u64(3);
        w.u8(9);
        let bytes = w.done();
        assert_eq!(Reader::new(&bytes).bytes(), Err(Bad), "3 announced, 1 left");
        assert_eq!(Reader::new(&[2]).flag(), Err(Bad));
        let mut w = Writer::default();
        w.bytes(&[0xff]);
        assert_eq!(Reader::new(&w.done()).text(), Err(Bad));
        let mut r = Reader::new(&[9]);
        assert_eq!(r.take(2), Err(Bad));
        assert_eq!(r.take(1), Ok(&[9][..]), "a refused read takes nothing");
    }

    #[test]
    fn debug_tells_lengths_never_bytes() {
        let mut w = Writer::default();
        w.text("pw");
        assert_eq!(format!("{w:?}"), "Writer { len: 10 }");
        assert_eq!(format!("{:?}", Reader::new(b"pw")), "Reader { left: 2 }");
    }
}
