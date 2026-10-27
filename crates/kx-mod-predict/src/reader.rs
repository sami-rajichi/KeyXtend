//! A front-to-back reader over a byte form, shared by the learned words' and the emails' files: every read checks what
//! is left, and bytes that end early give the caller's own error.

/// Reads `bytes` front to back; `short` is the error for bytes that end early.
pub(crate) struct Reader<'b, E> {
    /// The bytes not read yet.
    bytes: &'b [u8],
    /// The error past the end.
    short: E,
}

impl<'b, E: Copy> Reader<'b, E> {
    /// A reader at the start of `bytes`, giving `short` past their end.
    pub(crate) fn new(bytes: &'b [u8], short: E) -> Self {
        Self { bytes, short }
    }

    /// The next `n` bytes.
    pub(crate) fn take(&mut self, n: usize) -> Result<&'b [u8], E> {
        let (head, tail) = self.bytes.split_at_checked(n).ok_or(self.short)?;
        self.bytes = tail;
        Ok(head)
    }

    /// The next `N` bytes.
    pub(crate) fn array<const N: usize>(&mut self) -> Result<[u8; N], E> {
        self.take(N)?.try_into().map_err(|_| self.short)
    }

    /// The next u8.
    pub(crate) fn u8(&mut self) -> Result<u8, E> {
        self.array().map(u8::from_le_bytes)
    }

    /// The next little-endian u16.
    pub(crate) fn u16(&mut self) -> Result<u16, E> {
        self.array().map(u16::from_le_bytes)
    }

    /// The next little-endian u32.
    pub(crate) fn u32(&mut self) -> Result<u32, E> {
        self.array().map(u32::from_le_bytes)
    }

    /// True when every byte was read.
    pub(crate) fn done(&self) -> bool {
        self.bytes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The tests' error for bytes that end early.
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    struct Short;

    #[test]
    fn reads_little_endian_numbers_and_refuses_past_the_end() {
        let mut r = Reader::new(&[1, 2, 0, 3, 0, 0, 0], Short);
        assert_eq!((r.u8(), r.u16(), r.u32()), (Ok(1), Ok(2), Ok(3)));
        assert!(r.done());
        assert_eq!(r.u8(), Err(Short));
        let mut r = Reader::new(&[9], Short);
        assert_eq!(r.take(2), Err(Short));
        assert_eq!(r.take(1), Ok(&[9][..]), "a refused read takes nothing");
    }
}
