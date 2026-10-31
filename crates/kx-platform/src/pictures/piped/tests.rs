//! A piped worker over in-memory streams and anonymous pipes: replies in turn, a broken frame, a silent worker that is
//! late without a wait, asks written in turn, and the child's end and code.

use super::*;
use kx_ipc::{Codec, IpcError, Reply, Why};
use std::borrow::Cow;
use std::io::{self, Cursor};
use std::sync::{Arc, Mutex, PoisonError};

/// How long a test waits for a thread; a wait, never a sleep.
const WAIT: Duration = Duration::from_secs(10);
/// The exit code the test child gives.
const CODE: u32 = 7;

/// A child that keeps its calls; clones share them.
#[derive(Clone, Default)]
struct Kid(Arc<Mutex<Vec<&'static str>>>);

impl Kid {
    fn calls(&self) -> Vec<&'static str> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn push(&self, call: &'static str) {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(call);
    }
}

impl Child for Kid {
    fn end(&mut self) {
        self.push("end");
    }

    fn code(&mut self, _: Duration) -> Option<u32> {
        self.push("code");
        Some(CODE)
    }
}

fn pixels(id: u32) -> Reply {
    Reply::Pixels {
        id,
        width: 1,
        height: 1,
        rgba: vec![1, 2, 3, 4],
    }
}

fn failed(id: u32) -> Reply {
    Reply::Failed {
        id,
        why: Why::TooBig,
    }
}

fn frames(replies: &[Reply]) -> Vec<u8> {
    let mut out = Vec::new();
    for reply in replies {
        reply.write(&mut out).unwrap();
    }
    out
}

fn draw(id: u32) -> Ask<'static> {
    Ask::Draw {
        id,
        codec: Codec::Png,
        side: 8,
        bytes: Cow::Owned(vec![9; 3]),
    }
}

#[test]
fn replies_come_in_turn_and_the_end_reads_closed() {
    let out = Cursor::new(frames(&[pixels(1), failed(2)]));
    let mut worker = Piped::new(io::sink(), out, Kid::default()).unwrap();
    assert_eq!(worker.reply(WAIT), Heard::Frame(Ok(pixels(1))));
    assert_eq!(worker.reply(WAIT), Heard::Frame(Ok(failed(2))));
    let closed = Heard::Frame(Err(IpcError::Closed));
    assert_eq!(worker.reply(WAIT), closed);
    assert_eq!(worker.reply(WAIT), closed, "and it stays closed");
}

#[test]
fn a_broken_frame_is_heard_as_its_error() {
    let out = Cursor::new(b"not a frame at all".to_vec());
    let mut worker = Piped::new(io::sink(), out, Kid::default()).unwrap();
    assert_eq!(worker.reply(WAIT), Heard::Frame(Err(IpcError::BadMagic)));
}

#[test]
fn a_silent_worker_is_late_without_a_wait() {
    let (out, theirs) = io::pipe().unwrap();
    let mut worker = Piped::new(io::sink(), out, Kid::default()).unwrap();
    assert_eq!(worker.reply(Duration::ZERO), Heard::Late);
    drop(theirs);
    assert_eq!(worker.reply(WAIT), Heard::Frame(Err(IpcError::Closed)));
}

#[test]
fn asks_are_written_in_turn_and_a_drop_closes_the_input() {
    let (mut theirs, input) = io::pipe().unwrap();
    let mut worker = Piped::new(input, io::empty(), Kid::default()).unwrap();
    worker.send(draw(1));
    worker.send(Ask::Bye);
    assert_eq!(Ask::read(&mut theirs), Ok(draw(1)));
    assert_eq!(Ask::read(&mut theirs), Ok(Ask::Bye));
    drop(worker);
    assert_eq!(Ask::read(&mut theirs), Err(IpcError::Closed));
}

#[test]
fn end_and_code_reach_the_child() {
    let kid = Kid::default();
    let mut worker = Piped::new(io::sink(), io::empty(), kid.clone()).unwrap();
    worker.end();
    assert_eq!(worker.code(WAIT), Some(CODE));
    assert_eq!(kid.calls(), ["end", "code"]);
    assert_eq!(worker.child().calls(), kid.calls());
}
