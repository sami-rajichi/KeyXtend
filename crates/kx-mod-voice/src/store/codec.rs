//! The store's byte primitives: a mark and a version, little-endian numbers, flags, and texts after a u64 length. A
//! write counts its length first, so the wiped buffer never grows and leaves a copy behind; every read checks what is
//! left before it slices, so random bytes give `Bad`, never a panic or a large allocation.

use std::fmt;
use std::str;

use kx_module_api::Zeroizing;
use kx_module_api::ports::Date;

/// Why bytes did not read.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Bad {
    /// A newer version of the app wrote them, so this one leaves them alone.
    Newer,
    /// They are cut, changed or another form.
    Damaged,
}

/// The bytes that name a form, first in each file.
pub(crate) type Mark = [u8; 4];

/// Writes a byte form front to back, or only counts its length.
pub(crate) struct Writer {
    out: Option<Zeroizing<Vec<u8>>>,
    len: usize,
}

/// The bytes `write` writes, in one wiped buffer sized first.
pub(crate) fn encode(write: impl Fn(&mut Writer)) -> Zeroizing<Vec<u8>> {
    let mut count = Writer { out: None, len: 0 };
    write(&mut count);
    let out = Some(Zeroizing::new(Vec::with_capacity(count.len)));
    let mut writer = Writer { out, len: 0 };
    write(&mut writer);
    writer.out.unwrap_or_default()
}

impl Writer {
    fn put(&mut self, bytes: &[u8]) {
        self.len = self.len.saturating_add(bytes.len());
        if let Some(out) = &mut self.out {
            out.extend_from_slice(bytes);
        }
    }

    /// The form's mark and version.
    pub(crate) fn head(&mut self, mark: Mark, version: u8) {
        self.put(&mark);
        self.u8(version);
    }

    pub(crate) fn u8(&mut self, n: u8) {
        self.put(&[n]);
    }

    pub(crate) fn u16(&mut self, n: u16) {
        self.put(&n.to_le_bytes());
    }

    pub(crate) fn u32(&mut self, n: u32) {
        self.put(&n.to_le_bytes());
    }

    pub(crate) fn u64(&mut self, n: u64) {
        self.put(&n.to_le_bytes());
    }

    pub(crate) fn i64(&mut self, n: i64) {
        self.put(&n.to_le_bytes());
    }

    pub(crate) fn i16(&mut self, n: i16) {
        self.put(&n.to_le_bytes());
    }

    /// A flag as one byte, 0 or 1.
    pub(crate) fn flag(&mut self, on: bool) {
        self.u8(u8::from(on));
    }

    /// A count or a length, as a u64.
    pub(crate) fn count(&mut self, n: usize) {
        self.u64(u64::try_from(n).unwrap_or(u64::MAX));
    }

    /// `text`'s UTF-8 after its length.
    pub(crate) fn text(&mut self, text: &str) {
        self.count(text.len());
        self.put(text.as_bytes());
    }

    /// A date as its year, month and day.
    pub(crate) fn date(&mut self, date: Date) {
        self.u16(date.year());
        self.u8(date.month());
        self.u8(date.day());
    }
}

/// Reads a byte form front to back.
pub(crate) struct Reader<'b>(&'b [u8]);

impl<'b> Reader<'b> {
    pub(crate) fn new(bytes: &'b [u8]) -> Self {
        Self(bytes)
    }

    /// The form's head: another mark is `Damaged`, a later version `Newer`, an earlier one `Damaged`.
    pub(crate) fn head(&mut self, mark: Mark, version: u8) -> Result<(), Bad> {
        if self.take(mark.len())? != mark.as_slice() {
            return Err(Bad::Damaged);
        }
        match self.u8()? {
            v if v == version => Ok(()),
            v if v > version => Err(Bad::Newer),
            _ => Err(Bad::Damaged),
        }
    }

    /// The next `n` bytes.
    fn take(&mut self, n: usize) -> Result<&'b [u8], Bad> {
        let (head, tail) = self.0.split_at_checked(n).ok_or(Bad::Damaged)?;
        self.0 = tail;
        Ok(head)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], Bad> {
        self.take(N)?.try_into().map_err(|_| Bad::Damaged)
    }

    pub(crate) fn u8(&mut self) -> Result<u8, Bad> {
        self.array().map(u8::from_le_bytes)
    }

    pub(crate) fn u16(&mut self) -> Result<u16, Bad> {
        self.array().map(u16::from_le_bytes)
    }

    pub(crate) fn u32(&mut self) -> Result<u32, Bad> {
        self.array().map(u32::from_le_bytes)
    }

    pub(crate) fn u64(&mut self) -> Result<u64, Bad> {
        self.array().map(u64::from_le_bytes)
    }

    pub(crate) fn i64(&mut self) -> Result<i64, Bad> {
        self.array().map(i64::from_le_bytes)
    }

    pub(crate) fn i16(&mut self) -> Result<i16, Bad> {
        self.array().map(i16::from_le_bytes)
    }

    /// The next flag; a byte other than 0 or 1 is `Damaged`.
    pub(crate) fn flag(&mut self) -> Result<bool, Bad> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Bad::Damaged),
        }
    }

    /// The next count or length; never more than the bytes left, as each counted item takes a byte at least.
    pub(crate) fn count(&mut self) -> Result<usize, Bad> {
        let n = usize::try_from(self.u64()?).map_err(|_| Bad::Damaged)?;
        if n > self.0.len() {
            Err(Bad::Damaged)
        } else {
            Ok(n)
        }
    }

    /// The next UTF-8 text after its length.
    pub(crate) fn text(&mut self) -> Result<&'b str, Bad> {
        let n = self.count()?;
        str::from_utf8(self.take(n)?).map_err(|_| Bad::Damaged)
    }

    /// The next date; a day that does not exist is `Damaged`.
    pub(crate) fn date(&mut self) -> Result<Date, Bad> {
        let (year, month, day) = (self.u16()?, self.u8()?, self.u8()?);
        Date::new(year, month, day).ok_or(Bad::Damaged)
    }

    /// `Ok` when every byte was read, else `Damaged`.
    pub(crate) fn end(&self) -> Result<(), Bad> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(Bad::Damaged)
        }
    }
}

/// Tells the length written, never the bytes.
impl fmt::Debug for Writer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("Writer");
        s.field("len", &self.len).finish_non_exhaustive()
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

#[cfg(test)]
mod tests {
    use super::*;

    const MARK: Mark = *b"TEST";

    #[test]
    fn numbers_flags_texts_and_dates_round_trip_in_a_buffer_sized_first() {
        let day = Date::new(2026, 11, 14).unwrap();
        let bytes = encode(|w| {
            w.head(MARK, 1);
            w.u16(300);
            w.u32(70_000);
            w.u64(u64::MAX);
            w.i64(-5);
            w.i16(-60);
            w.flag(true);
            w.text("مرحبا");
            w.date(day);
        });
        assert_eq!(bytes.len(), bytes.capacity());
        let mut r = Reader::new(&bytes);
        assert_eq!(r.head(MARK, 1), Ok(()));
        assert_eq!(
            (r.u16(), r.u32(), r.u64()),
            (Ok(300), Ok(70_000), Ok(u64::MAX))
        );
        assert_eq!((r.i64(), r.i16(), r.flag()), (Ok(-5), Ok(-60), Ok(true)));
        assert_eq!((r.text(), r.date()), (Ok("مرحبا"), Ok(day)));
        assert_eq!(r.end(), Ok(()));
        assert_eq!(r.u8(), Err(Bad::Damaged));
    }

    #[test]
    fn another_mark_an_older_or_newer_version_and_bad_values_are_refused() {
        let head = |mark: &Mark, v: u8| encode(|w| w.head(*mark, v));
        assert_eq!(
            Reader::new(&head(b"ELSE", 1)).head(MARK, 1),
            Err(Bad::Damaged)
        );
        assert_eq!(Reader::new(&head(&MARK, 2)).head(MARK, 1), Err(Bad::Newer));
        assert_eq!(
            Reader::new(&head(&MARK, 0)).head(MARK, 1),
            Err(Bad::Damaged)
        );
        let long = encode(|w| {
            w.u64(3);
            w.u8(9);
        });
        assert_eq!(
            Reader::new(&long).text(),
            Err(Bad::Damaged),
            "3 announced, 1 left"
        );
        assert_eq!(Reader::new(&[2]).flag(), Err(Bad::Damaged));
        let bad_day = encode(|w| {
            w.u16(2026);
            w.u8(2);
            w.u8(30);
        });
        assert_eq!(Reader::new(&bad_day).date(), Err(Bad::Damaged));
        assert_eq!(Reader::new(&[1]).end(), Err(Bad::Damaged));
    }

    #[test]
    fn debug_tells_lengths_never_bytes() {
        let mut w = Writer { out: None, len: 0 };
        w.text("pw");
        assert_eq!(format!("{w:?}"), "Writer { len: 10, .. }");
        assert_eq!(format!("{:?}", Reader::new(b"pw")), "Reader { left: 2 }");
    }
}
