//! What the sealed blobs' formats share: a sink that counts or keeps bytes, lengths and texts written after their
//! length, and a reader that checks what is left before each slice, so random bytes give `Bad`, never a panic.

/// The bytes of a flag.
pub(crate) const FLAG: usize = 1;
/// The bytes of a count or a text's length.
pub(crate) const LEN: usize = size_of::<u32>();

/// Bytes that do not read as a blob.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Bad {
    /// Damaged, cut short or foreign.
    Damaged,
    /// Saved by a newer version.
    Newer,
}

/// Where bytes go: a counter, or the bytes themselves.
pub(crate) trait Sink {
    /// Takes `bytes` at the end.
    fn put(&mut self, bytes: &[u8]);
}

/// Counts the bytes a write would take.
pub(crate) struct Count(pub(crate) usize);

impl Sink for Count {
    fn put(&mut self, bytes: &[u8]) {
        self.0 = self.0.saturating_add(bytes.len());
    }
}

impl Sink for Vec<u8> {
    fn put(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }
}

/// Writes `flag` as one byte.
pub(crate) fn put_flag(sink: &mut impl Sink, flag: bool) {
    sink.put(&[u8::from(flag)]);
}

/// Writes `text` after its length.
pub(crate) fn put_text(sink: &mut impl Sink, text: &str) {
    put_bytes(sink, text.as_bytes());
}

/// Writes `bytes` after their length.
pub(crate) fn put_bytes(sink: &mut impl Sink, bytes: &[u8]) {
    put_len(sink, bytes.len());
    sink.put(bytes);
}

/// Writes a count or a length; every one the rules allow fits.
pub(crate) fn put_len(sink: &mut impl Sink, n: usize) {
    sink.put(&u32::try_from(n).unwrap_or(u32::MAX).to_le_bytes());
}

/// Reads front to back, checking what is left before each slice.
pub(crate) struct Reader<'a>(pub(crate) &'a [u8]);

impl<'a> Reader<'a> {
    /// The next `n` bytes.
    pub(crate) fn take(&mut self, n: usize) -> Result<&'a [u8], Bad> {
        let (head, rest) = self.0.split_at_checked(n).ok_or(Bad::Damaged)?;
        self.0 = rest;
        Ok(head)
    }

    /// The next `N` bytes as an array.
    fn array<const N: usize>(&mut self) -> Result<[u8; N], Bad> {
        self.take(N)?.try_into().map_err(|_| Bad::Damaged)
    }

    pub(crate) fn byte(&mut self) -> Result<u8, Bad> {
        Ok(self.array::<1>()?[0])
    }

    pub(crate) fn u64(&mut self) -> Result<u64, Bad> {
        self.array().map(u64::from_le_bytes)
    }

    pub(crate) fn i64(&mut self) -> Result<i64, Bad> {
        self.array().map(i64::from_le_bytes)
    }

    /// A flag: 0 or 1, nothing else.
    pub(crate) fn flag(&mut self) -> Result<bool, Bad> {
        match self.byte()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Bad::Damaged),
        }
    }

    /// A count or a length of at most `most`.
    pub(crate) fn count(&mut self, most: usize) -> Result<usize, Bad> {
        let n = usize::try_from(u32::from_le_bytes(self.array()?)).map_err(|_| Bad::Damaged)?;
        (n <= most).then_some(n).ok_or(Bad::Damaged)
    }

    /// Bytes after their length, of at most `most`.
    pub(crate) fn bytes(&mut self, most: usize) -> Result<&'a [u8], Bad> {
        let n = self.count(most)?;
        self.take(n)
    }

    /// A text of at most `most` bytes.
    pub(crate) fn text(&mut self, most: usize) -> Result<&'a str, Bad> {
        std::str::from_utf8(self.bytes(most)?).map_err(|_| Bad::Damaged)
    }

    /// The mark `mark`, then a version of at most `version`: a newer one is `Newer`, an older one `Damaged`.
    pub(crate) fn head(&mut self, mark: &[u8], version: u8) -> Result<(), Bad> {
        if self.take(mark.len())? != mark {
            return Err(Bad::Damaged);
        }
        match self.byte()? {
            v if v == version => Ok(()),
            v if v > version => Err(Bad::Newer),
            _ => Err(Bad::Damaged),
        }
    }

    /// Nothing is left.
    pub(crate) fn end(&self) -> Result<(), Bad> {
        self.0.is_empty().then_some(()).ok_or(Bad::Damaged)
    }
}
