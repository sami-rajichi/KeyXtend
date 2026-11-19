//! The job's loop over pipes: a dictation sent and told in order, then goodbye; no note holds the key or the sound; a
//! clean end and a broken frame; a request that panics told as `Gone` while the loop goes on; and a panic on an ask told
//! as `Gone` on that ask and on every run still live.

use super::bench::{WAIT, fault, part, send, text};
use super::*;
use crate::net::job::Job;
use crate::net::{Event, Notes, act, run};
use kx_ipc::{IpcError, NetAsk, NetFault, NetNote};
use kx_platform_net::{Call, Hop, Http, NetError, Reply};
use std::io::{self, Cursor, Read, Write};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

/// Input fed by the test, one frame at a time; it ends when the test drops its sender.
struct Feed(Receiver<Vec<u8>>, Vec<u8>);

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
struct Tap(Sender<Vec<u8>>);

impl Write for Tap {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let _ = self.0.send(buf.to_vec());
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// A server whose every call panics.
struct Boom;

impl Http for Boom {
    fn send(&self, _: &Hop<'_>) -> Result<Reply, NetError> {
        Err(NetError::Net)
    }

    fn deliver(&self, _: &Call<'_>) -> Result<Reply, NetError> {
        panic!("the test server panics");
    }
}

/// Notes whose first `tell` panics, as a broken writer might; later notes are kept.
struct Shaky(bool, Vec<NetNote>);

impl Notes for Shaky {
    fn tell(&mut self, note: NetNote) -> Result<(), IpcError> {
        if !self.0 {
            self.0 = true;
            panic!("the test's notes panic once");
        }
        self.1.push(note);
        Ok(())
    }
}

fn frame(ask: &NetAsk<'_>) -> Vec<u8> {
    let mut out = Vec::new();
    ask.write(&mut out).unwrap();
    out
}

fn notes(mut bytes: &[u8]) -> Vec<NetNote> {
    std::iter::from_fn(|| NetNote::read(&mut bytes).ok()).collect()
}

/// The loop over `http`, fed by the returned sender; its bytes come on the returned receiver.
fn start(http: Arc<dyn Http>) -> (Sender<Vec<u8>>, Receiver<Vec<u8>>, thread::JoinHandle<()>) {
    let ((ask_tx, ask_rx), (out_tx, out_rx)) = (mpsc::channel(), mpsc::channel());
    let clock: Arc<dyn Clock> = Arc::new(Still::default());
    let job = thread::spawn(move || {
        let end = run(
            Feed(ask_rx, Vec::new()),
            &mut Tap(out_tx),
            Kit { http, clock },
        );
        assert_eq!(end, Ok(()));
    });
    (ask_tx, out_rx, job)
}

/// The bytes the loop writes on `rx` until it has told `n` notes.
fn until(rx: &Receiver<Vec<u8>>, n: usize) -> Vec<u8> {
    let mut bytes = Vec::new();
    while notes(&bytes).len() < n {
        bytes.extend(rx.recv_timeout(WAIT).unwrap());
    }
    bytes
}

#[test]
fn the_loop_sends_a_dictation_tells_it_in_order_then_ends_at_goodbye() {
    let http = Arc::new(FakeHttp::new([said("Bonjour"), said("à tous")]));
    let (tx, rx, job) = start(http.clone());
    tx.send(frame(&send(1, call(), run_of(&[2, 2], 1, 1, 20))))
        .unwrap();
    tx.send(frame(&part(1, 0, sound(2, 1)))).unwrap();
    tx.send(frame(&part(1, 1, sound(2, 2)))).unwrap();
    let want = [
        text(1, 0, "Bonjour"),
        text(1, 1, "à tous"),
        NetNote::Done { id: 1 },
    ];
    assert_eq!(notes(&until(&rx, 3)), want);
    tx.send(frame(&NetAsk::Bye)).unwrap();
    job.join().unwrap();
    assert_eq!(http.called().len(), 2);
}

#[test]
fn no_note_holds_the_key_or_the_sound() {
    let http = Arc::new(FakeHttp::new([reply(401, KEY_BAD)]));
    let (tx, rx, job) = start(http);
    let sound = b"\xA5\x5A\xA5\x5A".to_vec();
    tx.send(frame(&send(2, call(), run_of(&[4], 1, 1, 20))))
        .unwrap();
    tx.send(frame(&part(2, 0, sound.clone()))).unwrap();
    let bytes = until(&rx, 1);
    assert_eq!(notes(&bytes), [fault(2, NetFault::Key)]);
    assert!(!bytes.windows(KEY.len()).any(|w| w == KEY));
    assert!(!bytes.windows(sound.len()).any(|w| w == sound.as_slice()));
    drop(tx);
    job.join().unwrap();
}

#[test]
fn the_end_of_input_ends_cleanly_and_a_broken_frame_with_its_error() {
    let kit = || {
        let clock: Arc<dyn Clock> = Arc::new(Still::default());
        Kit {
            http: Arc::new(FakeHttp::new([])),
            clock,
        }
    };
    let mut out = Vec::new();
    assert_eq!(run(Cursor::new(Vec::new()), &mut out, kit()), Ok(()));
    let broken = run(Cursor::new(b"not a frame at all".to_vec()), &mut out, kit());
    assert!(broken.is_err());
    assert!(out.is_empty());
}

#[test]
fn a_request_that_panics_is_gone_and_the_loop_goes_on() {
    let (tx, rx, job) = start(Arc::new(Boom));
    tx.send(frame(&send(3, call(), run_of(&[2], 3, 1, 20))))
        .unwrap();
    tx.send(frame(&part(3, 0, sound(2, 1)))).unwrap();
    assert_eq!(notes(&until(&rx, 1)), [fault(3, NetFault::Gone)]);
    tx.send(frame(&NetAsk::Bye)).unwrap();
    job.join().unwrap();
}

#[test]
fn a_panic_on_an_ask_tells_gone_on_it_and_on_every_live_run() {
    let kit = Kit {
        http: Arc::new(FakeHttp::new([])),
        clock: Arc::new(Still::default()),
    };
    let (mut job, mut out) = (Job::new(kit, mpsc::channel().0), Shaky(false, Vec::new()));
    for id in [1, 2] {
        let ask = send(id, call(), run_of(&[2], 1, 1, 20));
        assert!(act(&mut job, Event::Ask(Box::new(ask)), &mut out).is_ok());
    }
    let gemini = send(3, call_to(URL, NetShape::Gemini), run_of(&[2], 1, 1, 20));
    assert!(act(&mut job, Event::Ask(Box::new(gemini)), &mut out).is_ok());
    let (asked, live) = out.1.split_first().unwrap();
    assert_eq!(asked, &fault(3, NetFault::Gone));
    let mut live = live.to_vec();
    live.sort_by_key(NetNote::id);
    assert_eq!(live, [fault(1, NetFault::Gone), fault(2, NetFault::Gone)]);
    assert!(job.live().is_empty());
}
