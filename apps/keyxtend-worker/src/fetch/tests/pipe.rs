//! The job's loop over pipes: a download told in order and goodbye, a host gone mid-download, a clean end, a broken
//! frame, a panic told as an internal fault, and a note that fails its own check sent as one.

use super::super::get::MIN_STEP;
use super::super::{Event, Flow, Pipe, act, run};
use super::*;
use kx_platform_net::Body;
use std::io::{self, Cursor, Read, Write};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

/// The longest the test waits for the loop's next write before it fails; no test sleeps.
const WAIT: Duration = Duration::from_secs(20);

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

/// An `Http` whose every request panics.
struct Boom;

impl Http for Boom {
    fn send(&self, _: &Hop<'_>) -> Result<Reply, NetError> {
        panic!("the test client panics");
    }

    fn deliver(&self, _: &kx_platform_net::Call<'_>) -> Result<Reply, NetError> {
        panic!("the fetch job sends no call");
    }
}

/// The size of the file `Drip` serves: far more reads than a test takes.
const DRIP_SIZE: u64 = 1024 * MIN_STEP;

/// A server whose body gives one byte per read, so its download never ends within a test.
struct Drip;

impl Http for Drip {
    fn send(&self, _: &Hop<'_>) -> Result<Reply, NetError> {
        let len = usize::try_from(DRIP_SIZE).unwrap();
        let body = Box::new(Drip);
        Ok(Reply {
            head: Head::whole(len),
            body,
        })
    }

    fn deliver(&self, _: &kx_platform_net::Call<'_>) -> Result<Reply, NetError> {
        Err(NetError::Net)
    }
}

impl Body for Drip {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, NetError> {
        buf.first_mut().map_or(Ok(0), |b| {
            *b = 0;
            Ok(1)
        })
    }
}

fn frame(ask: &FetchAsk<'_>) -> Vec<u8> {
    let mut out = Vec::new();
    ask.write(&mut out).unwrap();
    out
}

fn notes(mut bytes: &[u8]) -> Vec<FetchNote> {
    std::iter::from_fn(|| FetchNote::read(&mut bytes).ok()).collect()
}

/// The notes the loop writes on `rx` up to its first end note.
fn until_end(rx: &Receiver<Vec<u8>>) -> Vec<FetchNote> {
    let mut bytes = Vec::new();
    loop {
        bytes.extend(rx.recv_timeout(WAIT).unwrap());
        let told = notes(&bytes);
        if told
            .last()
            .is_some_and(|n| !matches!(n, FetchNote::Progress { .. }))
        {
            return told;
        }
    }
}

#[test]
fn the_loop_downloads_in_order_then_ends_at_goodbye() {
    let (dir, server) = (Dir::new(), Server::new([whole(FILE)]));
    let ((ask_tx, ask_rx), (out_tx, out_rx)) = (mpsc::channel(), mpsc::channel());
    let http: Box<dyn Http> = Box::new(server.clone());
    let job = thread::spawn(move || run(Feed(ask_rx, Vec::new()), &mut Tap(out_tx), http));
    ask_tx.send(frame(&get(1, &dir))).unwrap();
    assert_eq!(
        until_end(&out_rx),
        [progress(1, 0), progress(1, 40), done(1)]
    );
    ask_tx.send(frame(&FetchAsk::Bye)).unwrap();
    assert_eq!(job.join().unwrap(), Ok(()));
    assert_eq!(dir.files(), (Some(FILE.to_vec()), None));
}

#[test]
fn a_host_that_goes_away_mid_download_ends_the_loop_and_names_nothing() {
    let dir = Dir::new();
    let mut ask = get(1, &dir);
    if let FetchAsk::Get { size, .. } = &mut ask {
        *size = DRIP_SIZE;
    }
    let ((ask_tx, ask_rx), (out_tx, out_rx)) = (mpsc::channel(), mpsc::channel());
    let job =
        thread::spawn(move || run(Feed(ask_rx, Vec::new()), &mut Tap(out_tx), Box::new(Drip)));
    ask_tx.send(frame(&ask)).unwrap();
    let mut bytes = Vec::new();
    while notes(&bytes).is_empty() {
        bytes.extend(out_rx.recv_timeout(WAIT).unwrap());
    }
    let total = DRIP_SIZE;
    assert_eq!(
        notes(&bytes),
        [FetchNote::Progress {
            id: 1,
            got: 0,
            total
        }]
    );
    drop(ask_tx);
    assert_eq!(job.join().unwrap(), Ok(()));
    assert_eq!(dir.files().0, None);
}

#[test]
fn the_end_of_input_ends_the_loop_cleanly_and_a_broken_frame_with_its_error() {
    let server = Server::new([]);
    let mut out = Vec::new();
    assert_eq!(
        run(Cursor::new(Vec::new()), &mut out, Box::new(server.clone())),
        Ok(())
    );
    let broken = run(
        Cursor::new(b"not a frame at all".to_vec()),
        &mut out,
        Box::new(server),
    );
    assert!(broken.is_err());
    assert!(out.is_empty());
}

#[test]
fn a_panic_inside_is_an_internal_fault_on_the_download_and_the_loop_goes_on() {
    let dir = Dir::new();
    let (mut job, mut out) = (Job::new(Box::new(Boom)), Vec::new());
    let started = act(&mut job, Some(Event::Ask(get(1, &dir))), &mut out);
    assert!(matches!(started, Ok(Flow::Go)));
    assert!(matches!(act(&mut job, None, &mut out), Ok(Flow::Go)));
    assert_eq!(out, [progress(1, 0), fault(1, FetchFault::Internal)]);
    assert!(!job.busy());
    assert_eq!(dir.files(), (None, None), "its empty part goes");
    let bye = act(&mut job, Some(Event::Ask(FetchAsk::Bye)), &mut out);
    assert!(matches!(bye, Ok(Flow::Exit(Ok(())))));
}

#[test]
fn a_note_that_fails_its_own_check_is_sent_as_an_internal_fault() {
    let mut bytes = Vec::new();
    let over = FetchNote::Progress {
        id: 4,
        got: 5,
        total: 4,
    };
    Pipe(&mut bytes).tell(over).unwrap();
    assert_eq!(notes(&bytes), [fault(4, FetchFault::Internal)]);
}
