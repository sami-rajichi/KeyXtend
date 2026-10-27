//! The lines of a bzip2 stream, as the Tatoeba and Wikipedia readers walk them, read into one reused buffer.

use std::io::{BufRead, BufReader, Read};

use bzip2::read::MultiBzDecoder;

/// What ends a line.
const LINE_END: u8 = b'\n';

/// A bzip2 stream read one line at a time.
pub(crate) struct Lines<R: Read> {
    /// The decoded stream.
    reader: BufReader<MultiBzDecoder<R>>,
    /// The last line read, its end included.
    buf: Vec<u8>,
}

impl<R: Read> Lines<R> {
    /// The lines of the bzip2 stream `reader`.
    pub(crate) fn new(reader: R) -> Self {
        Self {
            reader: BufReader::new(MultiBzDecoder::new(reader)),
            buf: Vec::new(),
        }
    }

    /// The next line, its end included; `None` at the stream's end.
    pub(crate) fn line(&mut self) -> Result<Option<&[u8]>, String> {
        self.buf.clear();
        let read = self.reader.read_until(LINE_END, &mut self.buf);
        let n = read.map_err(|e| e.to_string())?;
        Ok((n > 0).then_some(self.buf.as_slice()))
    }
}
