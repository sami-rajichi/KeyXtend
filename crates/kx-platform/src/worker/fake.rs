//! A scripted spawner and worker for the hosts' tests, for any job its `Faked` side knows: each ask takes the next
//! act, whose frames go to a stream's notes once it took them, and every call is kept in order. It never sleeps: a
//! silent worker is late at once, and the clock moves only when a test moves it.

mod fetch;
mod media;
mod net;
mod vault;
pub mod voice;

pub use fetch::FetchAct;
pub use media::Act;
pub use vault::VaultAct;
pub use voice::VoiceAct;

use super::{Frames, Heard, Launch, SpawnError, Spawner, Split, Worker};
use kx_ipc::{CLEAN_EXIT, FetchAsk, IpcError, NetAsk, VaultAsk, VoiceAsk};
use kx_module_api::{Clock, Mono};
use std::collections::VecDeque;
use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

/// A frame as a streamed worker sends it, or why it could not be read.
pub type Frame<F> = Result<<F as Frames>::Reply, IpcError>;

/// How a job's asks and acts look to the script.
pub trait Faked: Frames {
    /// What the worker does with an ask.
    type Act: Clone + Debug + Send + 'static;

    /// What it does with no act queued.
    fn usual() -> Self::Act;

    /// The ask's id and the call it is kept as; `None` for goodbye.
    fn note(ask: Self::Ask) -> Option<(u32, Call)>;

    /// What a worker doing `act` answers to ask `id`.
    fn answer(act: Self::Act, id: u32) -> Heard<Self::Reply>;

    /// The frames a streamed worker doing `act` sends for ask `id`, in order; a silent one sends none.
    fn play(act: Self::Act, id: u32) -> Vec<Frame<Self>> {
        match Self::answer(act, id) {
            Heard::Frame(frame) => vec![frame],
            Heard::Late => Vec::new(),
        }
    }
}

/// What the host did to a worker, in order, with the waits it gave; a vault, voice, fetch or net ask is kept whole,
/// as its `Debug` hides its values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Call {
    Draw { id: u32, side: u32 },
    Dib { id: u32, max_bytes: u32 },
    Vault(VaultAsk<'static>),
    Voice(VoiceAsk<'static>),
    Fetch(FetchAsk<'static>),
    Net(NetAsk<'static>),
    Bye,
    Reply { within: Duration },
    End,
    Code { within: Duration },
}

/// What the script holds and saw.
struct Log<F: Faked> {
    acts: VecDeque<F::Act>,
    /// Starts still to refuse.
    refuse: usize,
    /// Every start asked, refused or not.
    tries: usize,
    launches: Vec<Launch>,
    calls: Vec<Call>,
    /// How the live worker quit, if it did; a worker that quit reads as closed.
    exit: Option<u32>,
    /// The id of the ask that waits for its reply.
    asked: Option<u32>,
    /// Hears each call, for a test on another thread.
    tell: Option<Sender<Call>>,
    /// The live worker's notes, once a stream took them; dropping it closes them.
    tap: Option<Sender<Frame<F>>>,
    /// The most memory each worker's job took, as the system would tell it.
    peak: Option<u32>,
}

/// A spawner of scripted workers of job `F`; clones share one script.
pub struct Script<F: Faked>(Arc<Mutex<Log<F>>>);

impl<F: Faked> Default for Script<F> {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(Log {
            acts: VecDeque::new(),
            refuse: 0,
            tries: 0,
            launches: Vec::new(),
            calls: Vec::new(),
            exit: None,
            asked: None,
            tell: None,
            tap: None,
            peak: None,
        })))
    }
}

impl<F: Faked> Clone for Script<F> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<F: Faked> Script<F> {
    fn log(&self) -> MutexGuard<'_, Log<F>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Queues what a later ask does.
    pub fn act(&self, act: F::Act) {
        self.log().acts.push_back(act);
    }

    /// Makes the live worker quit with `code`, as if it died between asks; its notes close.
    pub fn quit(&self, code: u32) {
        let mut log = self.log();
        log.exit = Some(code);
        log.tap = None;
    }

    /// Sends `frame` from the live streamed worker on its own, with no ask.
    pub fn say(&self, frame: Frame<F>) {
        if let Some(tap) = &self.log().tap {
            let _ = tap.send(frame);
        }
    }

    /// Refuses the next `n` starts.
    pub fn refuse(&self, n: usize) {
        self.log().refuse = n;
    }

    /// Starts asked, refused or not.
    pub fn tries(&self) -> usize {
        self.log().tries
    }

    /// Workers started.
    pub fn spawns(&self) -> usize {
        self.log().launches.len()
    }

    /// How each worker was started.
    pub fn launches(&self) -> Vec<Launch> {
        self.log().launches.clone()
    }

    /// Every call so far.
    pub fn calls(&self) -> Vec<Call> {
        self.log().calls.clone()
    }

    /// Makes each worker's job tell `mb` as its peak memory.
    pub fn peak(&self, mb: Option<u32>) {
        self.log().peak = mb;
    }

    /// Sends each later call to `to`.
    pub fn tell(&self, to: Sender<Call>) {
        self.log().tell = Some(to);
    }

    fn record(&self, call: Call) {
        let mut log = self.log();
        if let Some(tell) = &log.tell {
            let _ = tell.send(call.clone());
        }
        log.calls.push(call);
    }

    /// Sends the next act's frames for ask `id` to a streamed worker's notes; a worker that is not streamed waits for
    /// its `reply` instead.
    fn play(&self, id: u32) {
        let mut log = self.log();
        let Some(tap) = log.tap.clone() else {
            return;
        };
        let act = log.acts.pop_front().unwrap_or_else(F::usual);
        drop(log);
        for frame in F::play(act, id) {
            let _ = tap.send(frame);
        }
    }
}

impl<F: Faked> Spawner<F> for Script<F> {
    type Worker = FakeWorker<F>;

    fn spawn(&self, launch: &Launch) -> Result<FakeWorker<F>, SpawnError> {
        let mut log = self.log();
        log.tries += 1;
        if log.refuse > 0 {
            log.refuse -= 1;
            return Err(SpawnError::Missing);
        }
        log.launches.push(launch.clone());
        log.exit = None;
        log.tap = None;
        Ok(FakeWorker(self.clone()))
    }
}

/// One scripted worker.
pub struct FakeWorker<F: Faked>(Script<F>);

impl<F: Faked> Worker<F> for FakeWorker<F> {
    fn send(&mut self, ask: F::Ask) {
        if let Some((id, call)) = F::note(ask) {
            self.0.log().asked = Some(id);
            self.0.record(call);
            self.0.play(id);
        } else {
            self.0.quit(u32::from(CLEAN_EXIT));
            self.0.record(Call::Bye);
        }
    }

    fn reply(&mut self, within: Duration) -> Heard<F::Reply> {
        self.0.record(Call::Reply { within });
        let mut log = self.0.log();
        let id = log.asked.take().unwrap_or_default();
        if log.exit.is_some() {
            return Heard::Frame(Err(IpcError::Closed));
        }
        let act = log.acts.pop_front().unwrap_or_else(F::usual);
        F::answer(act, id)
    }

    fn end(&mut self) {
        self.0.log().tap = None;
        self.0.record(Call::End);
    }

    fn code(&mut self, within: Duration) -> Option<u32> {
        self.0.record(Call::Code { within });
        self.0.log().exit
    }

    fn peak_mb(&mut self) -> Option<u32> {
        self.0.log().peak
    }
}

impl<F: Faked> Split<F> for FakeWorker<F> {
    fn notes(&mut self) -> Receiver<Frame<F>> {
        let (tap, notes) = mpsc::channel();
        self.0.log().tap = Some(tap);
        notes
    }
}

/// A clock that moves only when the test says; clones share it.
#[derive(Clone, Default)]
pub struct Hands(Arc<AtomicU64>);

impl Hands {
    /// Moves the time on by `by`.
    pub fn advance(&self, by: Duration) {
        let us = u64::try_from(by.as_micros()).unwrap_or(u64::MAX);
        self.0.fetch_add(us, Ordering::SeqCst);
    }
}

impl Clock for Hands {
    fn now(&self) -> Mono {
        Mono::from_us(self.0.load(Ordering::SeqCst))
    }
}
