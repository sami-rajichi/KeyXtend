//! A buffered read whose buffer is wiped after each frame and when dropped; a large read skips it, as `BufReader`'s does.

use kx_module_api::Zeroizing;
use std::io::{self, Read};

/// The buffer's size: `BufReader`'s own default.
const BUF_BYTES: usize = 8 * 1024;

/// `inner`, read through a buffer that is wiped when asked and when dropped.
pub(super) struct Wipe<R> {
    inner: R,
    buf: Zeroizing<Vec<u8>>,
    /// The next byte to hand out.
    at: usize,
    /// The end of the bytes read into `buf`.
    end: usize,
}

impl<R: Read> Wipe<R> {
    /// `inner` behind an empty buffer.
    pub(super) fn new(inner: R) -> Self {
        Self {
            inner,
            buf: Zeroizing::new(vec![0; BUF_BYTES]),
            at: 0,
            end: 0,
        }
    }

    /// Wipes the bytes handed out so far in place, moving those not yet read to the front of the buffer.
    pub(super) fn wipe(&mut self) {
        let left = self.end - self.at;
        self.buf.copy_within(self.at..self.end, 0);
        // A wiping iterator zeroes each byte behind the tail as it drops, with writes the compiler keeps.
        drop(Zeroizing::new(self.buf[left..].iter_mut()));
        (self.at, self.end) = (0, left);
    }
}

impl<R: Read> Read for Wipe<R> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if self.at == self.end {
            if out.len() >= self.buf.len() {
                return self.inner.read(out);
            }
            let n = self.inner.read(&mut self.buf)?;
            (self.at, self.end) = (0, n.min(self.buf.len()));
        }
        let n = out.len().min(self.end - self.at);
        out[..n].copy_from_slice(&self.buf[self.at..self.at + n]);
        self.at += n;
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// A frame's worth of test bytes, under the buffer's size.
    const SMALL: usize = 100;

    #[test]
    fn reads_come_through_whole_and_a_wipe_keeps_what_is_unread() {
        let bytes: Vec<u8> = (1..=u8::MAX).cycle().take(SMALL * 2).collect();
        let mut wipe = Wipe::new(Cursor::new(bytes.clone()));
        let mut first = vec![0; SMALL];
        wipe.read_exact(&mut first).unwrap();
        let held = wipe.buf.as_ptr();
        wipe.wipe();
        assert_eq!(wipe.buf.as_ptr(), held, "the same buffer, wiped in place");
        assert_eq!(
            wipe.buf[..SMALL],
            bytes[SMALL..],
            "the tail moved to the front"
        );
        assert!(
            wipe.buf[SMALL..].iter().all(|&b| b == 0),
            "handed-out bytes are gone"
        );
        let mut rest = Vec::new();
        wipe.read_to_end(&mut rest).unwrap();
        assert_eq!([first, rest].concat(), bytes);
    }

    #[test]
    fn a_wipe_after_the_end_leaves_no_byte() {
        let bytes = vec![7; SMALL];
        let mut wipe = Wipe::new(Cursor::new(bytes.clone()));
        let mut all = Vec::new();
        wipe.read_to_end(&mut all).unwrap();
        assert_eq!(all, bytes);
        wipe.wipe();
        assert!(wipe.buf.iter().all(|&b| b == 0), "nothing is left");
    }

    #[test]
    fn a_large_read_skips_the_buffer() {
        let bytes = vec![7; BUF_BYTES * 2];
        let mut wipe = Wipe::new(Cursor::new(bytes.clone()));
        let mut out = vec![0; BUF_BYTES * 2];
        wipe.read_exact(&mut out).unwrap();
        assert_eq!(out, bytes);
        assert!(
            wipe.buf.iter().all(|&b| b == 0),
            "nothing passed through it"
        );
    }
}
