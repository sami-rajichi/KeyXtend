//! A scripted spawner and worker for the hosts' tests, for any job its `Faked` side knows: each ask takes the next
//! act, and every call is kept in order. It never sleeps: a silent worker is late at once, and the clock moves only
//! when a test moves it.

mod media;
mod vault;

pub use media::Act;
pub use vault::VaultAct;

use super::{Frames, Heard, Launch, SpawnError, Spawner, Worker};
use kx_ipc::{CLEAN_EXIT, IpcError, VaultAsk};
use kx_module_api::{Clock, Mono};
use std::collections::VecDeque;
use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

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
}

/// What the host did to a worker, in order, with the waits it gave; a vault ask is kept whole, as its `Debug` hides
/// its values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Call {
    Draw { id: u32, side: u32 },
    Dib { id: u32, max_bytes: u32 },
    Vault(VaultAsk<'static>),
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

    /// Makes the live worker quit with `code`, as if it died between asks.
    pub fn quit(&self, code: u32) {
        self.log().exit = Some(code);
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
        } else {
            self.0.log().exit = Some(u32::from(CLEAN_EXIT));
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
        self.0.record(Call::End);
    }

    fn code(&mut self, within: Duration) -> Option<u32> {
        self.0.record(Call::Code { within });
        self.0.log().exit
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
