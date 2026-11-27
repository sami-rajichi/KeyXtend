//! A session's worker and the rules it lives by: started on demand, one ask at a time within the deadline, ended when
//! late or lying and held back by a doubling backoff, and stopped when idle.

use super::backoff::{Backoff, ms};
use super::{Frames, Heard, Launch, Lost, Spawner, Times, Worker, bye, exit_note};
use kx_ipc::IpcError;
use kx_module_api::{Clock, Mono};
use std::sync::Arc;
use std::time::Duration;

/// Why a worker was ended.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Fault {
    /// No reply within the deadline.
    Late,
    /// A reply that names another ask.
    Stray,
    /// A reply of another kind than asked, or one that does not fit the ask.
    BadReply,
    /// A frame that could not be read.
    BadFrame,
    /// The worker closed its output.
    Ended,
}

/// The worker of one session of job `F`.
pub(crate) struct Host<S: Spawner<F>, F: Frames> {
    spawner: Arc<S>,
    clock: Arc<dyn Clock>,
    times: Times,
    launch: Launch,
    worker: Option<S::Worker>,
    /// When the worker last answered.
    used: Mono,
    backoff: Backoff,
    /// The next ask's id; ids start at 1.
    next: u32,
    /// Its job shows system dialogs, so the worker is granted the foreground before each ask.
    front: bool,
}

impl<S: Spawner<F>, F: Frames> Host<S, F> {
    /// A host with no worker yet; the first ask starts one as `launch` says.
    pub(crate) fn new(
        spawner: Arc<S>,
        clock: Arc<dyn Clock>,
        times: Times,
        launch: Launch,
    ) -> Self {
        Self {
            spawner,
            clock,
            times,
            launch,
            worker: None,
            used: Mono::default(),
            backoff: Backoff::new(times),
            next: 1,
            front: false,
        }
    }

    /// The same host, whose worker is granted the foreground before each ask, as a job that shows dialogs needs.
    pub(crate) const fn fronted(mut self) -> Self {
        self.front = true;
        self
    }

    /// Sends the ask `make` builds with the next id, the worker started first if need be, and reads its reply with
    /// `take` within the deadline; `take` gives `None` for a reply that does not fit the ask, which ends the worker.
    /// An ask `make` refuses starts nothing.
    pub(crate) fn call<T, E: From<Lost>>(
        &mut self,
        make: impl FnOnce(u32) -> Result<F::Ask, E>,
        take: impl FnOnce(F::Reply) -> Option<Result<T, E>>,
    ) -> Result<T, E> {
        self.idle();
        let id = self.next;
        let ask = make(id)?;
        self.next = id.checked_add(1).unwrap_or(1);
        let (deadline, front) = (ms(self.times.deadline_ms), self.front);
        let worker = self.start()?;
        if front && !worker.grant_front() {
            tracing::debug!(job = F::JOB, "the system refused a worker the foreground");
        }
        worker.send(ask);
        let heard = worker.reply(deadline);
        self.judge(id, heard, take)
    }

    /// How long the session may wait before the idle check; `None` without a worker.
    pub(crate) fn wake_in(&self) -> Option<Duration> {
        let since = self.clock.now().saturating_since(self.used);
        self.worker
            .as_ref()
            .map(|_| self.idle_time().saturating_sub(since))
    }

    /// Starts the next worker as `times` and `launch` say; a worker started otherwise says goodbye first.
    pub(crate) fn relaunch(&mut self, times: Times, launch: Launch) {
        if (self.times, &self.launch) != (times, &launch) {
            self.stop();
            self.backoff.retime(times);
            (self.times, self.launch) = (times, launch);
        }
    }

    /// Stops the worker once it has been idle for its time.
    pub(crate) fn idle(&mut self) {
        if self.wake_in() == Some(Duration::ZERO) {
            self.stop();
        }
    }

    /// The live worker, started now unless the backoff still runs; one that quit since the last ask is replaced.
    /// `Down` while it rests or when it cannot start.
    fn start(&mut self) -> Result<&mut S::Worker, Lost> {
        self.clear_quit();
        let now = self.clock.now();
        if self.worker.is_none() {
            if self.backoff.resting(now) {
                return Err(self.backoff.down(now));
            }
            match self.spawner.spawn(&self.launch) {
                Ok(worker) => self.worker = Some(worker),
                Err(e) => {
                    tracing::warn!(job = F::JOB, error = %e, "a worker did not start");
                    self.backoff.hold(now);
                }
            }
        }
        let down = self.backoff.down(now);
        self.worker.as_mut().ok_or(down)
    }

    /// Ends a worker that quit since the last ask, with no backoff, since no ask of ours failed.
    fn clear_quit(&mut self) {
        let Some(worker) = self.worker.as_mut() else {
            return;
        };
        if let Some(code) = worker.code(Duration::ZERO) {
            worker.end();
            self.worker = None;
            let note = exit_note(Some(code));
            tracing::warn!(job = F::JOB, note, "a worker quit between asks");
        }
    }

    /// Checks the reply to ask `id`, whose answer `take` reads when it fits; a worker that lies or is late is ended.
    fn judge<T, E: From<Lost>>(
        &mut self,
        id: u32,
        heard: Heard<F::Reply>,
        take: impl FnOnce(F::Reply) -> Option<Result<T, E>>,
    ) -> Result<T, E> {
        let reply = match heard {
            Heard::Frame(Ok(reply)) if F::id(&reply) == id => reply,
            Heard::Frame(Ok(_)) => return Err(self.fail(Fault::Stray).into()),
            Heard::Late => return Err(self.fail(Fault::Late).into()),
            Heard::Frame(Err(IpcError::Closed)) => return Err(self.fail(Fault::Ended).into()),
            Heard::Frame(Err(_)) => return Err(self.fail(Fault::BadFrame).into()),
        };
        let Some(answer) = take(reply) else {
            return Err(self.fail(Fault::BadReply).into());
        };
        self.served();
        answer
    }

    /// Notes an answer: the idle time starts again and the backoff goes back to its first wait.
    fn served(&mut self) {
        self.used = self.clock.now();
        self.backoff.reset();
    }

    /// Ends the worker for `fault` and starts the backoff; a late worker is `Late`, any other `Gone`.
    fn fail(&mut self, fault: Fault) -> Lost {
        if let Some(mut worker) = self.worker.take() {
            let code = (fault == Fault::Ended).then(|| worker.code(ms(self.times.deadline_ms)));
            worker.end();
            let note = code.map(exit_note);
            tracing::warn!(job = F::JOB, ?fault, ?note, "a worker failed and was ended");
        }
        self.backoff.hold(self.clock.now());
        if fault == Fault::Late {
            Lost::Late
        } else {
            Lost::Gone
        }
    }

    /// Says goodbye to the worker, waits the deadline at most for it to quit, then ends it.
    fn stop(&mut self) {
        if let Some(mut worker) = self.worker.take() {
            bye::<F, _>(&mut worker, ms(self.times.deadline_ms));
        }
    }

    /// How long the worker may sit idle.
    fn idle_time(&self) -> Duration {
        self.times.idle()
    }
}

impl<S: Spawner<F>, F: Frames> Drop for Host<S, F> {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests;
