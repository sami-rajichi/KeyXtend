//! A session's state: its worker, when it last answered, and the backoff before the next start.

use super::{Heard, Launch, Spawner, Worker, exit_note};
use kx_ipc::{Ask, CLEAN_EXIT, Codec, IpcError, MAX_SIDE, Reply, Why};
use kx_module_api::ports::{ImageCodec, PictureAsk, PictureConfig, PictureError, Pixels};
use kx_module_api::{Clock, Mono};
use std::borrow::Cow;
use std::io;
use std::sync::Arc;
use std::time::Duration;

/// Why a worker was ended.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Fault {
    /// No reply within the deadline.
    Late,
    /// A reply that names another draw.
    Stray,
    /// Pixels that do not fit what was asked.
    BadPixels,
    /// A frame that could not be read.
    BadFrame,
    /// The worker closed its output.
    Ended,
}

/// The worker of one session and the rules it lives by.
pub(super) struct Host<S: Spawner> {
    spawner: Arc<S>,
    clock: Arc<dyn Clock>,
    cfg: PictureConfig,
    worker: Option<S::Worker>,
    /// When the worker last answered.
    used: Mono,
    /// No start before this point.
    calm: Mono,
    /// The backoff after the next failure.
    pause: Duration,
    /// The next draw's id; ids start at 1.
    next: u32,
}

impl<S: Spawner> Host<S> {
    /// A host with no worker yet; the first draw starts one.
    pub(super) fn new(spawner: Arc<S>, clock: Arc<dyn Clock>, cfg: PictureConfig) -> Self {
        Self {
            spawner,
            clock,
            cfg,
            worker: None,
            used: Mono::default(),
            calm: Mono::default(),
            pause: first_pause(&cfg),
            next: 1,
        }
    }

    /// Draws `ask` on the worker, started first if need be: one draw at a time, within the deadline.
    pub(super) fn draw(&mut self, ask: PictureAsk) -> Result<Pixels, PictureError> {
        self.idle();
        let (id, side) = (self.next, ask.side.min(MAX_SIDE));
        let ask = Ask::Draw {
            id,
            codec: codec(ask.codec),
            side,
            bytes: Cow::Owned(ask.bytes.into_inner()),
        };
        fits(&ask)?;
        self.next = id.checked_add(1).unwrap_or(1);
        let deadline = ms(self.cfg.deadline_ms);
        let worker = self.start()?;
        worker.send(ask);
        let heard = worker.reply(deadline);
        self.judge(id, side, heard)
    }

    /// How long the session may wait before the idle check; `None` without a worker.
    pub(super) fn wake_in(&self) -> Option<Duration> {
        let since = self.clock.now().saturating_since(self.used);
        self.worker
            .as_ref()
            .map(|_| self.idle_time().saturating_sub(since))
    }

    /// Stops the worker once it has been idle for the config's time.
    pub(super) fn idle(&mut self) {
        if self.wake_in() == Some(Duration::ZERO) {
            self.stop();
        }
    }

    /// The live worker, started now unless the backoff still runs; one that quit since the last draw is replaced.
    fn start(&mut self) -> Result<&mut S::Worker, PictureError> {
        self.clear_quit();
        if self.worker.is_none() {
            let now = self.clock.now();
            if now < self.calm {
                return Err(PictureError::Gone);
            }
            match self.spawner.spawn(&Launch::media(&self.cfg)) {
                Ok(worker) => self.worker = Some(worker),
                Err(e) => {
                    tracing::warn!(error = %e, "the picture worker did not start");
                    self.back_off(now);
                }
            }
        }
        self.worker.as_mut().ok_or(PictureError::Gone)
    }

    /// Ends a worker that quit since the last draw, with no backoff, since no draw of ours failed.
    fn clear_quit(&mut self) {
        let Some(worker) = self.worker.as_mut() else {
            return;
        };
        if let Some(code) = worker.code(Duration::ZERO) {
            worker.end();
            self.worker = None;
            let note = exit_note(Some(code));
            tracing::warn!(note, "the picture worker quit between draws");
        }
    }

    /// Checks the reply to draw `id` of longest side `side`; a worker that lies or is late is ended.
    fn judge(&mut self, id: u32, side: u32, heard: Heard) -> Result<Pixels, PictureError> {
        match heard {
            Heard::Late => Err(self.fail(Fault::Late)),
            Heard::Frame(Ok(reply)) if reply.id() != id => Err(self.fail(Fault::Stray)),
            Heard::Frame(Ok(Reply::Pixels {
                width,
                height,
                rgba,
                ..
            })) => match Pixels::new(width, height, rgba, side) {
                Some(pixels) => {
                    self.served();
                    Ok(pixels)
                }
                None => Err(self.fail(Fault::BadPixels)),
            },
            Heard::Frame(Ok(Reply::Failed { why, .. })) => {
                self.served();
                Err(refusal(why))
            }
            Heard::Frame(Err(IpcError::Closed)) => Err(self.fail(Fault::Ended)),
            Heard::Frame(Err(_)) => Err(self.fail(Fault::BadFrame)),
        }
    }

    /// Notes a good answer: the idle time starts again and the backoff goes back to its first wait.
    fn served(&mut self) {
        self.used = self.clock.now();
        self.pause = first_pause(&self.cfg);
    }

    /// Ends the worker for `fault` and starts the backoff; a late worker is a timeout, any other is gone.
    fn fail(&mut self, fault: Fault) -> PictureError {
        if let Some(mut worker) = self.worker.take() {
            let code = (fault == Fault::Ended).then(|| worker.code(ms(self.cfg.deadline_ms)));
            worker.end();
            let note = code.map(exit_note);
            tracing::warn!(?fault, ?note, "the picture worker failed and was ended");
        }
        self.back_off(self.clock.now());
        if fault == Fault::Late {
            PictureError::Timeout
        } else {
            PictureError::Gone
        }
    }

    /// Holds the next start back by the backoff, which then doubles up to its cap.
    fn back_off(&mut self, now: Mono) {
        self.calm = now.saturating_add(self.pause);
        let cap = ms(self.cfg.restart_max_ms);
        self.pause = self.pause.saturating_mul(2).min(cap);
    }

    /// Says goodbye to the worker, waits the deadline at most for it to quit, then ends it.
    fn stop(&mut self) {
        let Some(mut worker) = self.worker.take() else {
            return;
        };
        worker.send(Ask::Bye);
        let code = worker.code(ms(self.cfg.deadline_ms));
        worker.end();
        if code != Some(u32::from(CLEAN_EXIT)) {
            let note = exit_note(code);
            tracing::warn!(note, "the picture worker did not quit cleanly");
        }
    }

    /// How long the worker may sit idle.
    fn idle_time(&self) -> Duration {
        Duration::from_secs(u64::from(self.cfg.idle_s))
    }
}

impl<S: Spawner> Drop for Host<S> {
    fn drop(&mut self) {
        self.stop();
    }
}

/// The backoff's first wait, never past its cap.
fn first_pause(cfg: &PictureConfig) -> Duration {
    ms(cfg.restart_ms).min(ms(cfg.restart_max_ms))
}

fn ms(n: u32) -> Duration {
    Duration::from_millis(u64::from(n))
}

/// Refuses an ask the worker would refuse, by `kx-ipc`'s own writer checks, before any worker starts.
fn fits(ask: &Ask<'_>) -> Result<(), PictureError> {
    ask.write(&mut io::sink()).map_err(|e| match e {
        IpcError::TooLong => PictureError::TooBig,
        _ => PictureError::Refused,
    })
}

/// The frame's code for `codec`.
const fn codec(codec: ImageCodec) -> Codec {
    match codec {
        ImageCodec::Png => Codec::Png,
        ImageCodec::Dib => Codec::Dib,
    }
}

/// The port's error for the worker's reason; a failure inside the worker leaves that image undrawn.
const fn refusal(why: Why) -> PictureError {
    match why {
        Why::TooBig => PictureError::TooBig,
        Why::Refused | Why::Internal => PictureError::Refused,
    }
}

#[cfg(test)]
mod tests;
