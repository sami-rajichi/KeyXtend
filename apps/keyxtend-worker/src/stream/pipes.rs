//! Test pipes for the streaming jobs' loops: input the test feeds one frame at a time, and output that hands each write
//! to the test.

use std::io::{self, Read, Write};
use std::sync::mpsc::{Receiver, Sender};

/// Input fed by the test, one frame at a time; it ends when the test drops its sender.
pub struct Feed(pub Receiver<Vec<u8>>, pub Vec<u8>);

impl Read for Feed {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.1.is_empty() {
            match self.0.recv() {
                Ok(bytes) => self.1 = bytes,
                Err(_) => return Ok(0),
            }
        }
        let n = buf.len().min(self.1.len());
        buf[..n].copy_from_slice(&self.1[..n]);
        self.1.drain(..n);
        Ok(n)
    }
}

/// Output that hands each write to the test.
pub struct Tap(pub Sender<Vec<u8>>);

impl Write for Tap {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let _ = self.0.send(buf.to_vec());
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
