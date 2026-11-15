//! A stream's worker and its rules: started by a fresh ask, each watched id kept by its notes, ended when silent past
//! a wait or for a frame its job cannot read or a note on an id it never took, held back by a doubling backoff, and
//! stopped when idle.

use super::watch::{Wait, Watches};
use super::{Split, Streams, Told, Unsent};
use crate::worker::backoff::{Backoff, ms};
use crate::worker::{Launch, Lost, SpawnError, Spawner, Times, Worker, bye, exit_note};
use kx_ipc::IpcError;
use kx_module_api::{Clock, Mono};
use std::io;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::time::Duration;

/// Hears what the stream tells.
pub(super) type Tell<R> = Box<dyn FnMut(Told<R>) + Send>;
/// Hands worker `nth`'s notes to a reader thread of their own.
pub(super) type Pump<R> =
    Box<dyn FnMut(u64, Receiver<Result<R, IpcError>>) -> Result<(), SpawnError> + Send>;

/// Why a streamed worker was ended.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Fault {
    /// A watched id stayed silent past its wait.
    Late,
    /// A frame its job cannot read: an unknown kind, one past its size, or a bad field.
    BadFrame,
    /// A note on an id never asked of this worker.
    Stray,
    /// It closed its output while an id was watched.
    Ended,
    /// It closed its output with nothing watched, so no ask of ours failed.
    Quit,
}

/// The worker of one stream of job `F`.
pub(super) struct Flow<S: Spawner<F, Worker: Split<F>>, F: Streams> {
    spawner: Arc<S>,
    clock: Arc<dyn Clock>,
    times: Times,
    launch: Launch,
    tell: Tell<F::Reply>,
    pump: Pump<F::Reply>,
    worker: Option<S::Worker>,
    /// Counts the workers started, so a frame of an ended one is dropped.
    nth: u64,
    /// The first id asked of the live worker.
    base: u32,
    /// The next fresh id; ids start at 1.
    next: u32,
    watches: Watches,
    /// When the worker last spoke or was asked.
    used: Mono,
    backoff: Backoff,
}

impl<S: Spawner<F, Worker: Split<F>>, F: Streams> Flow<S, F> {
    /// A flow with no worker yet; the first fresh ask starts one as `launch` says, and `pump` reads its notes.
    pub(super) fn new(
        spawner: Arc<S>,
        clock: Arc<dyn Clock>,
        times: Times,
        launch: Launch,
        tell: Tell<F::Reply>,
        pump: Pump<F::Reply>,
    ) -> Self {
        Self {
            spawner,
            clock,
            times,
            launch,
            tell,
            pump,
            worker: None,
            nth: 0,
            base: 1,
            next: 1,
            watches: Watches::default(),
            used: Mono::default(),
            backoff: Backoff::new(times),
        }
    }

    /// Sends the ask `make` builds from a fresh id, the worker started first if need be, and watches that id as
    /// `wait` says; an ask built with another id is refused.
    pub(super) fn begin(
        &mut self,
        make: impl FnOnce(u32) -> F::Ask,
        wait: Wait,
    ) -> Result<u32, Unsent> {
        self.idle();
        let id = self.next;
        let ask = make(id);
        if F::ask_id(&ask) != id {
            return Err(Unsent::Refused(IpcError::BadField));
        }
        refuse::<F>(&ask)?;
        self.start()?;
        self.next = id.checked_add(1).unwrap_or(1);
        self.send(ask, wait);
        Ok(id)
    }

    /// Sends `ask` on a run the live worker took, and watches its id as `wait` says.
    pub(super) fn ask(&mut self, ask: F::Ask, wait: Wait) -> Result<(), Unsent> {
        refuse::<F>(&ask)?;
        self.clear_quit();
        if !self.owns(F::ask_id(&ask)) {
            return Err(Lost::Gone.into());
        }
        self.send(ask, wait);
        Ok(())
    }

    /// Takes a frame worker `nth` sent, as its reader took it `at`: a note on a run it took is told and keeps its
    /// id's watch, and a worker late by then, a stray note or anything else ends the worker.
    pub(super) fn heard(&mut self, nth: u64, frame: Result<F::Reply, IpcError>, at: Mono) {
        if nth != self.nth || self.worker.is_none() {
            return;
        }
        if self.watches.late(at) {
            return self.lose(Fault::Late);
        }
        match frame {
            Ok(note) if self.owns(F::id(&note)) => self.note(note),
            Ok(_) => self.lose(Fault::Stray),
            Err(IpcError::Closed) => self.lose(self.closed()),
            Err(_) => self.lose(Fault::BadFrame),
        }
    }

    /// Ends a worker silent past a wait, and stops one idle for its time.
    pub(super) fn tick(&mut self) {
        if self.watches.late(self.clock.now()) {
            self.lose(Fault::Late);
        } else {
            self.idle();
        }
    }

    /// How long the stream may wait before its next `tick`; `None` without a worker.
    pub(super) fn wake_in(&self) -> Option<Duration> {
        self.worker.as_ref()?;
        let idle = || self.used.saturating_add(self.times.idle());
        let at = self.watches.due().unwrap_or_else(idle);
        Some(at.saturating_since(self.clock.now()))
    }

    /// Says goodbye to the worker and drops every watch; the next fresh ask starts another with no backoff.
    pub(super) fn stop(&mut self) {
        self.watches.clear();
        if let Some(mut worker) = self.worker.take() {
            bye::<F, _>(&mut worker, ms(self.times.deadline_ms));
        }
    }

    /// True when run `id` was begun on the live worker.
    fn owns(&self, id: u32) -> bool {
        let asked = self.next.wrapping_sub(self.base);
        self.worker.is_some() && id.wrapping_sub(self.base) < asked
    }

    /// Hands `ask` to the live worker and watches its id as `wait` says.
    fn send(&mut self, ask: F::Ask, wait: Wait) {
        let (id, now) = (F::ask_id(&ask), self.clock.now());
        if let Some(worker) = self.worker.as_mut() {
            worker.send(ask);
        }
        self.used = now;
        self.watches.set(id, wait, now);
    }

    /// Says goodbye to the worker and drops every watch, as `stop`; the next starts as `launch` and `times` say, held
    /// back by the same backoff.
    pub(super) fn relaunch(&mut self, launch: Launch, times: Times) {
        self.stop();
        (self.launch, self.times) = (launch, times);
    }

    /// Tells `note`, with its job's peak memory added where it reports memory; one on a watched id keeps, shifts or
    /// ends its watch and resets the backoff.
    fn note(&mut self, mut note: F::Reply) {
        let now = self.clock.now();
        self.used = now;
        let (id, last, shift) = (F::id(&note), F::last(&note), F::shift(&note));
        if self.watches.heard(id, now, last, shift) {
            self.backoff.reset();
        }
        if let Some(worker) = self.worker.as_mut() {
            F::stamp(&mut note, &mut || worker.peak_mb());
        }
        (self.tell)(Told::Note(note));
    }

    /// Stops a worker that sat idle for its time with nothing watched, and tells it.
    fn idle(&mut self) {
        if self.watches.is_empty() && self.wake_in() == Some(Duration::ZERO) {
            self.stop();
            (self.tell)(Told::Idle);
        }
    }

    /// Why a worker that closed is ended: with nothing watched, no ask of ours failed.
    fn closed(&self) -> Fault {
        if self.watches.is_empty() {
            Fault::Quit
        } else {
            Fault::Ended
        }
    }

    /// Ends a worker that quit though its close is not heard yet, just as its close would.
    fn clear_quit(&mut self) {
        let quit = self.worker.as_mut().and_then(|w| w.code(Duration::ZERO));
        if quit.is_some() {
            self.lose(self.closed());
        }
    }

    /// Starts a worker unless a live one runs; `Down` while the backoff runs or when none starts.
    fn start(&mut self) -> Result<(), Lost> {
        self.clear_quit();
        if self.worker.is_some() {
            return Ok(());
        }
        let now = self.clock.now();
        if self.backoff.resting(now) {
            return Err(self.backoff.down(now));
        }
        match self.spawner.spawn(&self.launch) {
            Ok(worker) => self.hire(worker, now),
            Err(e) => {
                tracing::warn!(job = F::JOB, error = %e, "a streamed worker did not start");
                self.backoff.hold(now);
                Err(self.backoff.down(now))
            }
        }
    }

    /// Takes a started worker once a reader thread has its notes; without one it is ended.
    fn hire(&mut self, mut worker: S::Worker, now: Mono) -> Result<(), Lost> {
        self.nth += 1;
        if (self.pump)(self.nth, worker.notes()).is_err() {
            worker.end();
            self.backoff.hold(now);
            return Err(self.backoff.down(now));
        }
        (self.base, self.used) = (self.next, now);
        self.worker = Some(worker);
        Ok(())
    }

    /// Ends the worker for `fault`, drops every watch and tells it; a worker that quit is held back by no backoff.
    fn lose(&mut self, fault: Fault) {
        self.watches.clear();
        let Some(mut worker) = self.worker.take() else {
            return;
        };
        let closed = matches!(fault, Fault::Ended | Fault::Quit);
        let code = closed.then(|| worker.code(ms(self.times.deadline_ms)));
        worker.end();
        let note = code.map(exit_note);
        tracing::warn!(job = F::JOB, ?fault, ?note, "a streamed worker was ended");
        if fault != Fault::Quit {
            self.backoff.hold(self.clock.now());
        }
        let lost = if fault == Fault::Late {
            Lost::Late
        } else {
            Lost::Gone
        };
        (self.tell)(Told::Lost(lost));
    }
}

impl<S: Spawner<F, Worker: Split<F>>, F: Streams> Drop for Flow<S, F> {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Refuses an ask its frames would refuse, before any worker sees it: a refused ask would end the worker's input.
fn refuse<F: Streams>(ask: &F::Ask) -> Result<(), Unsent> {
    F::write(ask, &mut io::sink()).map_err(Unsent::Refused)
}

#[cfg(test)]
mod tests;
