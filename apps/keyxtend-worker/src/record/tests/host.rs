//! The host's side of the job in tests: asks written as frames, notes read back from the pipe, and a barrier that shows
//! every ask before it was taken.

use super::super::run;
use super::{NONE, WAIT};
use crate::stream::pipes::{Feed, Tap};
use kx_ipc::{IpcError, RecordAsk, RecordFault, RecordNote};
use kx_platform_record::fake::FakeStudio;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

/// The host's side of a job running on its own thread: it writes asks and reads notes. Dropping it opens the studio's
/// clock and ends the input, so a failed test never leaves the job waiting.
pub(super) struct Host {
    studio: FakeStudio,
    feed: Option<Sender<Vec<u8>>>,
    out: Receiver<Vec<u8>>,
    bytes: Vec<u8>,
    job: Option<JoinHandle<Result<(), IpcError>>>,
}

impl Host {
    /// Starts the job over `studio`.
    pub(super) fn start(studio: &FakeStudio) -> Self {
        let (feed, input) = mpsc::channel();
        let (tap, out) = mpsc::channel();
        let job = studio.clone();
        let job = thread::spawn(move || run(Feed(input, Vec::new()), &mut Tap(tap), job));
        Self {
            studio: studio.clone(),
            feed: Some(feed),
            out,
            bytes: Vec::new(),
            job: Some(job),
        }
    }

    /// Writes `ask` as one frame.
    pub(super) fn send(&self, ask: &RecordAsk<'_>) {
        let mut frame = Vec::new();
        ask.write(&mut frame).unwrap();
        self.feed.as_ref().unwrap().send(frame).unwrap();
    }

    /// The next note, within `WAIT`.
    pub(super) fn next(&mut self) -> RecordNote {
        loop {
            let mut rest = &self.bytes[..];
            if let Ok(note) = RecordNote::read(&mut rest) {
                let used = self.bytes.len() - rest.len();
                self.bytes.drain(..used);
                return note;
            }
            let more = self.out.recv_timeout(WAIT).expect("a note within the wait");
            self.bytes.extend(more);
        }
    }

    /// The notes up to the answer to a switch on no run, so every ask written before was taken.
    pub(super) fn barrier(&mut self) -> Vec<RecordNote> {
        self.send(&RecordAsk::Mic { id: NONE, on: true });
        let answer = RecordNote::Fault {
            id: NONE,
            fault: RecordFault::Failed,
        };
        let mut told = Vec::new();
        loop {
            match self.next() {
                note if note == answer => return told,
                note => told.push(note),
            }
        }
    }

    /// Ends the input, or says goodbye first, and hands back how the job ended and every note it wrote after.
    pub(super) fn finish(mut self, bye: bool) -> (Result<(), IpcError>, Vec<RecordNote>) {
        if bye {
            self.send(&RecordAsk::Bye);
        }
        self.feed = None;
        let ended = self.job.take().unwrap().join().unwrap();
        self.bytes.extend(self.out.try_iter().flatten());
        let mut rest = &self.bytes[..];
        let told = std::iter::from_fn(|| RecordNote::read(&mut rest).ok()).collect();
        (ended, told)
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        self.studio.free();
        self.feed = None;
    }
}
