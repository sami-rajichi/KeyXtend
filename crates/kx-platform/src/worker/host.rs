//! A session's worker and the rules it lives by: started on demand, one ask at a time within the deadline, ended when
//! late or lying and held back by a doubling backoff, and stopped when idle.

use super::{Frames, Heard, Launch, Lost, Spawner, Times, Worker, exit_note};
use kx_ipc::{CLEAN_EXIT, IpcError};
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
    /// No start before this point.
    calm: Mono,
    /// The backoff after the next failure.
    pause: Duration,
    /// The next ask's id; ids start at 1.
    next: u32,
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
            calm: Mono::default(),
            pause: first_pause(times),
            next: 1,
        }
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
        let deadline = ms(self.times.deadline_ms);
        let worker = self.start()?;
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
            if now < self.calm {
                return Err(self.down(now));
            }
            match self.spawner.spawn(&self.launch) {
                Ok(worker) => self.worker = Some(worker),
                Err(e) => {
                    tracing::warn!(job = F::JOB, error = %e, "a worker did not start");
                    self.back_off(now);
                }
            }
        }
        let down = self.down(now);
        self.worker.as_mut().ok_or(down)
    }

    /// The refusal of an ask while no worker runs: how long until one may start again.
    fn down(&self, now: Mono) -> Lost {
        Lost::Down {
            retry_in: self.calm.saturating_since(now),
        }
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
        self.pause = first_pause(self.times);
    }

    /// Ends the worker for `fault` and starts the backoff; a late worker is `Late`, any other `Gone`.
    fn fail(&mut self, fault: Fault) -> Lost {
        if let Some(mut worker) = self.worker.take() {
            let code = (fault == Fault::Ended).then(|| worker.code(ms(self.times.deadline_ms)));
            worker.end();
            let note = code.map(exit_note);
            tracing::warn!(job = F::JOB, ?fault, ?note, "a worker failed and was ended");
        }
        self.back_off(self.clock.now());
        if fault == Fault::Late {
            Lost::Late
        } else {
            Lost::Gone
        }
    }

    /// Holds the next start back by the backoff, which then doubles up to its cap.
    fn back_off(&mut self, now: Mono) {
        self.calm = now.saturating_add(self.pause);
        let cap = ms(self.times.restart_max_ms);
        self.pause = self.pause.saturating_mul(2).min(cap);
    }

    /// Says goodbye to the worker, waits the deadline at most for it to quit, then ends it.
    fn stop(&mut self) {
        let Some(mut worker) = self.worker.take() else {
            return;
        };
        worker.send(F::bye());
        let code = worker.code(ms(self.times.deadline_ms));
        worker.end();
        if code != Some(u32::from(CLEAN_EXIT)) {
            let note = exit_note(code);
            tracing::warn!(job = F::JOB, note, "a worker did not quit cleanly");
        }
    }

    /// How long the worker may sit idle.
    fn idle_time(&self) -> Duration {
        Duration::from_secs(u64::from(self.times.idle_s))
    }
}

impl<S: Spawner<F>, F: Frames> Drop for Host<S, F> {
    fn drop(&mut self) {
        self.stop();
    }
}

/// The backoff's first wait, never past its cap.
fn first_pause(times: Times) -> Duration {
    ms(times.restart_ms).min(ms(times.restart_max_ms))
}

fn ms(n: u32) -> Duration {
    Duration::from_millis(u64::from(n))
}

#[cfg(test)]
mod tests;
