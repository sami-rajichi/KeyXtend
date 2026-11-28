//! A stream on its own thread: notes told in order between asks, a bad note read by the real pipe reader ending the
//! worker, a silent worker ended once the clock passes its wait, a stopped or dropped stream saying goodbye, and a
//! `tell` that calls its own stream getting `Gone`. Nothing sleeps: tests wait on channels.

use super::*;
use crate::worker::fake::voice::{devices, done, level, list, open, piece, shut, start, stop};
use crate::worker::fake::{Call, Hands, Script, VoiceAct};
use crate::worker::{Child, Piped, SpawnError, VoiceFrames};
use kx_ipc::{FetchNote, VOICE_JOB, VoiceNote};
use std::io::{self, Cursor};
use std::sync::mpsc::Receiver;
use std::sync::{Mutex, PoisonError};
use std::time::Duration;

mod exit;
mod load;
mod post;

/// How long a test waits for the stream's thread; a wait, never a sleep.
const LONG: Duration = Duration::from_secs(10);
/// A test worker's memory cap, in MB.
const MEMORY_MB: u32 = 64;
/// The stream's thread name in the tests.
const THREAD: &str = "kx-test-stream";
/// Test values; the real ones come from each port's config.
const TIMES: Times = Times {
    deadline_ms: 2_000,
    idle_s: 30,
    restart_ms: 1_000,
    restart_max_ms: 4_000,
};
/// How long a test watch waits for a note.
const WAIT: Duration = Duration::from_millis(500);
/// The first id a stream hands out.
const FIRST: u32 = 1;
/// A level note's payload: one byte.
const LEVEL_LEN: usize = size_of::<u8>();

/// What a stream told, with the name of the thread that told it.
type Heard = Receiver<(Option<String>, Told<VoiceNote>)>;
/// A stream a test shares with its own `tell`.
type Shared = Arc<Mutex<Option<Stream<Script<VoiceFrames>, VoiceFrames>>>>;

/// How a test stream starts its worker.
fn launch() -> Launch {
    Launch {
        args: vec![VOICE_JOB.to_owned()],
        memory_mb: MEMORY_MB,
    }
}

/// A voice stream over `spawner` on the clock `clock`, and what it tells.
fn streamed<S>(spawner: S, clock: &Hands) -> (Stream<S, VoiceFrames>, Heard)
where
    S: Spawner<VoiceFrames, Worker: Split<VoiceFrames>>,
{
    let (said, heard) = mpsc::channel();
    let tell = move |told| {
        let name = thread::current().name().map(str::to_owned);
        drop(said.send((name, told)));
    };
    let (spawner, clock) = (Arc::new(spawner), Arc::new(clock.clone()));
    let stream = Stream::open(THREAD, spawner, clock, TIMES, launch(), tell);
    (stream, heard)
}

/// The next `n` things told, each on the stream's thread.
fn next(heard: &Heard, n: usize) -> Vec<Told<VoiceNote>> {
    let one = |_| {
        let (name, told) = heard.recv_timeout(LONG).expect("told in time");
        assert_eq!(name.as_deref(), Some(THREAD));
        told
    };
    (0..n).map(one).collect()
}

/// Each note as told.
fn told(list: &[VoiceNote]) -> Vec<Told<VoiceNote>> {
    list.iter().cloned().map(Told::Note).collect()
}

/// Waits until the script's workers were ended `n` times.
fn ended(called: &Receiver<Call>, n: usize) {
    for _ in 0..n {
        while called.recv_timeout(LONG).expect("the worker is ended") != Call::End {}
    }
}

/// A child that says on a channel each time it is ended.
struct Kid(Sender<()>);

impl Child for Kid {
    fn end(&mut self) {
        let _ = self.0.send(());
    }

    fn code(&mut self, _: Duration) -> Option<u32> {
        None
    }
}

/// Starts one piped worker, whose output is the bytes it holds; no other.
struct Canned(Mutex<Option<(Vec<u8>, Kid)>>);

impl Spawner<VoiceFrames> for Canned {
    type Worker = Piped<Kid, VoiceFrames>;

    fn spawn(&self, _: &Launch) -> Result<Self::Worker, SpawnError> {
        let held = self.0.lock().unwrap_or_else(PoisonError::into_inner).take();
        let (bytes, kid) = held.ok_or(SpawnError::Missing)?;
        Piped::new(io::sink(), Cursor::new(bytes), kid)
    }
}

/// `note`'s frame.
fn frame(note: &VoiceNote) -> Vec<u8> {
    let mut out = Vec::new();
    note.write(&mut out).unwrap();
    out
}

/// A frame of a kind voice notes never take: the fetch job's.
fn unknown() -> Vec<u8> {
    let mut out = Vec::new();
    FetchNote::Done { id: FIRST }.write(&mut out).unwrap();
    out
}

/// A level whose header claims one byte more than a level may take; the length is the header's last field.
fn oversized() -> Vec<u8> {
    let mut out = frame(&level(FIRST));
    let head = out.len() - LEVEL_LEN;
    let len = u32::try_from(LEVEL_LEN + 1).unwrap();
    out[head - size_of::<u32>()..head].copy_from_slice(&len.to_le_bytes());
    out
}

#[test]
fn notes_arrive_in_order_between_asks_on_the_stream_thread() {
    let script = Script::default();
    let (stream, heard) = streamed(script.clone(), &Hands::default());
    script.act(VoiceAct::Say(vec![open, level, piece]));
    let id = stream.begin(start, Wait::Each(WAIT)).unwrap();
    assert_eq!(next(&heard, 3), told(&[open(id), level(id), piece(id)]));
    script.say(Ok(level(id)));
    assert_eq!(next(&heard, 1), told(&[level(id)]), "a note on its own");
    script.act(VoiceAct::Say(vec![shut, piece, done]));
    let asked = thread::scope(|s| s.spawn(|| stream.ask(stop(id), Wait::Each(WAIT))).join());
    assert_eq!(asked.unwrap(), Ok(()), "an ask from another thread");
    assert_eq!(next(&heard, 3), told(&[shut(id), piece(id), done(id)]));
    let quit = Call::Code {
        within: Duration::ZERO,
    };
    let asks = [Call::Voice(start(id)), quit, Call::Voice(stop(id))];
    assert_eq!(script.calls(), asks, "a quit checked first");
}

#[test]
fn a_note_of_an_unknown_kind_or_past_its_size_ends_the_worker() {
    let read = |bytes: Vec<u8>| VoiceNote::read(&mut Cursor::new(bytes));
    assert_eq!(read(unknown()), Err(IpcError::BadKind), "the fixtures");
    assert_eq!(read(oversized()), Err(IpcError::TooLong));
    for bad in [unknown(), oversized()] {
        let (ends, ended) = mpsc::channel();
        let bytes = [frame(&level(FIRST)), bad].concat();
        let canned = Canned(Mutex::new(Some((bytes, Kid(ends)))));
        let (stream, heard) = streamed(canned, &Hands::default());
        assert_eq!(stream.begin(start, Wait::Each(WAIT)), Ok(FIRST));
        let lost = Told::Lost(Lost::Gone);
        assert_eq!(next(&heard, 2), [Told::Note(level(FIRST)), lost]);
        ended.recv_timeout(LONG).expect("the worker is ended");
    }
}

#[test]
fn a_worker_silent_past_its_wait_is_ended_once_the_clock_passes_it() {
    let (script, clock) = (Script::default(), Hands::default());
    let (calls, called) = mpsc::channel();
    script.tell(calls);
    let (stream, heard) = streamed(script.clone(), &clock);
    stream.begin(list, Wait::Each(WAIT)).unwrap();
    clock.advance(WAIT);
    stream.run(|_| ()).expect("the thread runs");
    assert_eq!(next(&heard, 1), [Told::Lost(Lost::Late)]);
    ended(&called, 1);
    let retry_in = Duration::from_millis(TIMES.restart_ms.into());
    let down = Err(Unsent::Lost(Lost::Down { retry_in }));
    assert_eq!(stream.begin(list, Wait::Each(WAIT)), down);
}

#[test]
fn a_late_worker_is_waited_for_once_ended_before_its_loss_is_told() {
    let (script, clock) = (Script::<VoiceFrames>::default(), Hands::default());
    let (said, heard) = mpsc::channel();
    let seen = script.clone();
    let tell = move |told| {
        if matches!(told, Told::Lost(_)) {
            drop(said.send(seen.calls()));
        }
    };
    let spawner = Arc::new(script.clone());
    let stream = Stream::open(
        THREAD,
        spawner,
        Arc::new(clock.clone()),
        TIMES,
        launch(),
        tell,
    );
    stream.begin(list, Wait::Each(WAIT)).unwrap();
    clock.advance(WAIT);
    stream.run(|_| ()).expect("the thread runs");
    let calls = heard.recv_timeout(LONG).expect("told in time");
    let within = Duration::from_millis(TIMES.deadline_ms.into());
    let last = [Call::End, Call::Code { within }];
    assert!(calls.ends_with(&last), "its files close first: {calls:?}");
}

#[test]
fn a_stopped_or_dropped_stream_says_goodbye_to_its_worker() {
    let script = Script::default();
    let (calls, called) = mpsc::channel();
    script.tell(calls);
    let (stream, heard) = streamed(script.clone(), &Hands::default());
    let within = Duration::from_millis(TIMES.deadline_ms.into());
    let bye = [Call::Bye, Call::Code { within }, Call::End];
    assert_eq!(stream.begin(list, Wait::Off), Ok(FIRST));
    stream.stop();
    assert!(script.calls().ends_with(&bye), "stop waits for the goodbye");
    assert_eq!(
        stream.begin(list, Wait::Off),
        Ok(FIRST + 1),
        "a fresh ask starts another"
    );
    assert_eq!(script.spawns(), 2);
    drop(stream);
    ended(&called, 2);
    assert!(script.calls().ends_with(&bye), "the second worker too");
    assert!(heard.try_recv().is_err(), "a goodbye tells nothing");
}

#[test]
fn a_tell_that_calls_its_own_stream_gets_gone_and_never_hangs() {
    let script = Script::default();
    let (shared, (said, heard)) = (Shared::default(), mpsc::channel());
    let own = Arc::clone(&shared);
    let tell = move |_| {
        let held = own.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(stream) = held.as_ref() {
            let begun = stream.begin(list, Wait::Off);
            stream.stop();
            let _ = said.send(begun);
        }
    };
    let (spawner, clock) = (Arc::new(script.clone()), Arc::new(Hands::default()));
    let stream = Stream::open(THREAD, spawner, clock, TIMES, launch(), tell);
    script.act(VoiceAct::Say(vec![devices]));
    let mut slot = shared.lock().unwrap_or_else(PoisonError::into_inner);
    assert_eq!(slot.insert(stream).begin(list, Wait::Off), Ok(FIRST));
    drop(slot);
    let gone = Err(Unsent::Lost(Lost::Gone));
    assert_eq!(heard.recv_timeout(LONG), Ok(gone), "no deadlock");
    assert_eq!(script.calls(), [Call::Voice(list(FIRST))], "nor a stop");
    let stream = shared.lock().unwrap_or_else(PoisonError::into_inner).take();
    drop(stream);
}
