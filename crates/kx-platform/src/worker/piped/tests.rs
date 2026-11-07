//! A piped worker over in-memory streams and anonymous pipes: replies in turn, a broken frame, a silent worker that is
//! late without a wait, asks written in turn, pipe threads named by their job, and the child's end and code.

use super::*;
use crate::worker::{MediaFrames, VaultFrames};
use kx_ipc::{Ask, Codec, IpcError, Reply, VaultAsk, Why};
use std::borrow::Cow;
use std::collections::HashSet;
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

/// A piped worker on the media frames.
fn media<W, R>(input: W, output: R, kid: Kid) -> Result<Piped<Kid, MediaFrames>, SpawnError>
where
    W: Write + Send + 'static,
    R: Read + Send + 'static,
{
    Piped::new(input, output, kid)
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
    let mut worker = media(io::sink(), out, Kid::default()).unwrap();
    assert_eq!(worker.reply(WAIT), Heard::Frame(Ok(pixels(1))));
    assert_eq!(worker.reply(WAIT), Heard::Frame(Ok(failed(2))));
    let closed = Heard::Frame(Err(IpcError::Closed));
    assert_eq!(worker.reply(WAIT), closed);
    assert_eq!(worker.reply(WAIT), closed, "and it stays closed");
}

#[test]
fn a_broken_frame_is_heard_as_its_error() {
    let out = Cursor::new(b"not a frame at all".to_vec());
    let mut worker = media(io::sink(), out, Kid::default()).unwrap();
    assert_eq!(worker.reply(WAIT), Heard::Frame(Err(IpcError::BadMagic)));
}

#[test]
fn a_silent_worker_is_late_without_a_wait() {
    let (out, theirs) = io::pipe().unwrap();
    let mut worker = media(io::sink(), out, Kid::default()).unwrap();
    assert_eq!(worker.reply(Duration::ZERO), Heard::Late);
    drop(theirs);
    assert_eq!(worker.reply(WAIT), Heard::Frame(Err(IpcError::Closed)));
}

#[test]
fn asks_are_written_in_turn_and_a_drop_closes_the_input() {
    let (mut theirs, input) = io::pipe().unwrap();
    let mut worker = media(input, io::empty(), Kid::default()).unwrap();
    worker.send(draw(1));
    worker.send(Ask::Bye);
    assert_eq!(Ask::read(&mut theirs), Ok(draw(1)));
    assert_eq!(Ask::read(&mut theirs), Ok(Ask::Bye));
    drop(worker);
    assert_eq!(Ask::read(&mut theirs), Err(IpcError::Closed));
}

/// A stream that tells which thread used it: it reads nothing and takes every write.
struct Spy(Sender<String>);

impl Spy {
    fn tell(&self) {
        let name = thread::current().name().unwrap_or_default().to_owned();
        let _ = self.0.send(name);
    }
}

impl Read for Spy {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        self.tell();
        Ok(0)
    }
}

impl Write for Spy {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.tell();
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// The names of the threads that use the spies of a worker made by `make`, once `send` has run.
fn thread_names<W>(make: impl FnOnce(Spy, Spy) -> W, send: impl FnOnce(&mut W)) -> HashSet<String> {
    let (tell, told) = mpsc::channel();
    let mut worker = make(Spy(tell.clone()), Spy(tell));
    send(&mut worker);
    let mut names = HashSet::new();
    while names.len() < 2 {
        let name = told
            .recv_timeout(WAIT)
            .expect("both threads use their pipe");
        names.insert(name);
    }
    names
}

#[test]
fn each_pipe_thread_names_its_job() {
    let media = thread_names(
        |input, output| media(input, output, Kid::default()).unwrap(),
        |w| w.send(Ask::Bye),
    );
    let vault = thread_names(
        |input, output| Piped::<_, VaultFrames>::new(input, output, Kid::default()).unwrap(),
        |w| w.send(VaultAsk::Bye),
    );
    let names = |n: [&str; 2]| n.map(str::to_owned).into();
    assert_eq!(media, names(["kx-media-in", "kx-media-out"]));
    assert_eq!(vault, names(["kx-vault-in", "kx-vault-out"]));
}

#[test]
fn end_and_code_reach_the_child() {
    let kid = Kid::default();
    let mut worker = media(io::sink(), io::empty(), kid.clone()).unwrap();
    worker.end();
    assert_eq!(worker.code(WAIT), Some(CODE));
    assert_eq!(kid.calls(), ["end", "code"]);
    assert_eq!(worker.child().calls(), kid.calls());
}
