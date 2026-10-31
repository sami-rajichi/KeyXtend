//! A scripted spawner and worker for the host's tests: each ask takes the next act, and every call is kept in order.
//! It never sleeps: a silent worker is late at once, and the clock moves only when a test moves it.

use super::{Heard, Launch, SpawnError, Spawner, Worker};
use kx_ipc::{Ask, CLEAN_EXIT, IpcError, RGBA_BYTES, Reply, Why};
use kx_module_api::{Clock, Mono};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

/// What the worker does with the next ask; with none queued it draws one pixel.
#[derive(Clone, Debug)]
pub enum Act {
    /// Pixels `w` by `h` for the ask's id.
    Draw(u32, u32),
    /// A DIB of this many bytes for the ask's id.
    Dib(usize),
    /// A failure for the draw's id.
    Fail(Why),
    /// This reply, whatever the draw's id.
    Raw(Reply),
    /// Nothing within the wait.
    Silent,
    /// A frame that could not be read.
    Broken(IpcError),
}

/// What the host did to a worker, in order, with the waits it gave.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Call {
    Draw { id: u32, side: u32 },
    Dib { id: u32, max_bytes: u32 },
    Bye,
    Reply { within: Duration },
    End,
    Code { within: Duration },
}

/// What the script holds and saw.
#[derive(Default)]
struct Log {
    acts: VecDeque<Act>,
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

/// A spawner of scripted workers; clones share one script.
#[derive(Clone, Default)]
pub struct Script(Arc<Mutex<Log>>);

impl Script {
    fn log(&self) -> MutexGuard<'_, Log> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Queues what a later draw does.
    pub fn act(&self, act: Act) {
        self.log().acts.push_back(act);
    }

    /// Makes the live worker quit with `code`, as if it died between draws.
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

impl Spawner for Script {
    type Worker = FakeWorker;

    fn spawn(&self, launch: &Launch) -> Result<FakeWorker, SpawnError> {
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
pub struct FakeWorker(Script);

impl Worker for FakeWorker {
    fn send(&mut self, ask: Ask<'static>) {
        match ask {
            Ask::Draw { id, side, .. } => {
                self.0.log().asked = Some(id);
                self.0.record(Call::Draw { id, side });
            }
            Ask::Dib { id, max_bytes, .. } => {
                self.0.log().asked = Some(id);
                self.0.record(Call::Dib { id, max_bytes });
            }
            Ask::Bye => {
                self.0.log().exit = Some(u32::from(CLEAN_EXIT));
                self.0.record(Call::Bye);
            }
        }
    }

    fn reply(&mut self, within: Duration) -> Heard {
        self.0.record(Call::Reply { within });
        let mut log = self.0.log();
        let id = log.asked.take().unwrap_or_default();
        if log.exit.is_some() {
            return Heard::Frame(Err(IpcError::Closed));
        }
        match log.acts.pop_front().unwrap_or(Act::Draw(1, 1)) {
            Act::Draw(width, height) => {
                let len = usize::try_from(width * height).unwrap_or_default() * RGBA_BYTES;
                let rgba = vec![0; len];
                Heard::Frame(Ok(Reply::Pixels {
                    id,
                    width,
                    height,
                    rgba,
                }))
            }
            Act::Dib(len) => Heard::Frame(Ok(Reply::Dib {
                id,
                bytes: vec![0; len],
            })),
            Act::Fail(why) => Heard::Frame(Ok(Reply::Failed { id, why })),
            Act::Raw(reply) => Heard::Frame(Ok(reply)),
            Act::Silent => Heard::Late,
            Act::Broken(e) => Heard::Frame(Err(e)),
        }
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
