//! A fake pictures service: each draw takes the answer the test queued, the latest repeating, and the fake records the
//! session's config and counts the asks. It never sleeps: a hung worker is a queued `Timeout`.

use crate::lock;
use kx_module_api::ports::{PictureAsk, PictureConfig, PictureError, PictureRun, Pictures, Pixels};
use std::collections::VecDeque;
use std::fmt;
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::time::Duration;

/// What one draw returns.
type Drawn = Result<Pixels, PictureError>;

/// What the sessions did, and what the test set for them.
#[derive(Default)]
struct Log {
    /// The answers still to come, oldest first.
    queued: VecDeque<Drawn>,
    /// The latest answer given, which repeats once the queue is empty.
    last: Option<Drawn>,
    last_ask: Option<PictureAsk>,
    config: Option<PictureConfig>,
    opens: usize,
    stops: usize,
    asks: usize,
}

impl Log {
    /// The next queued answer, else the latest, else `Gone`; pixels past `side` are `Gone`, as bad pixels are.
    fn answer(&mut self, side: u32) -> Drawn {
        if let Some(next) = self.queued.pop_front() {
            self.last = Some(next);
        }
        let drawn = self.last.clone().unwrap_or(Err(PictureError::Gone))?;
        let fits = drawn.width().max(drawn.height()) <= side;
        fits.then_some(drawn).ok_or(PictureError::Gone)
    }
}

/// The log, and the signal each ask gives, for a test waiting on another thread.
#[derive(Default)]
struct Shared {
    log: Mutex<Log>,
    asked: Condvar,
}

/// Pictures shared by their clones: each draw of any session takes the next queued answer, the latest repeating once
/// they run out, and `Gone` before any.
#[derive(Clone, Default)]
pub struct FakePictures {
    shared: Arc<Shared>,
}

impl FakePictures {
    /// Pictures with no session and nothing queued.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Queues `drawn` for a later draw, after those already queued: pixels, or any `PictureError`.
    pub fn queue(&self, drawn: Result<Pixels, PictureError>) {
        lock(&self.shared.log).queued.push_back(drawn);
    }

    /// How many sessions opened.
    #[must_use]
    pub fn opens(&self) -> usize {
        lock(&self.shared.log).opens
    }

    /// Whether a session is held, from its open until the module drops it.
    #[must_use]
    pub fn running(&self) -> bool {
        let log = lock(&self.shared.log);
        log.opens > log.stops
    }

    /// The config the latest session took, kept after it stops.
    #[must_use]
    pub fn config(&self) -> Option<PictureConfig> {
        lock(&self.shared.log).config
    }

    /// How many draws were asked, failed ones too.
    #[must_use]
    pub fn asks(&self) -> usize {
        lock(&self.shared.log).asks
    }

    /// The latest draw's ask, failed or not.
    #[must_use]
    pub fn last_ask(&self) -> Option<PictureAsk> {
        lock(&self.shared.log).last_ask.clone()
    }

    /// Waits up to `within` until `count` draws were asked, so a test never sleeps; true when they were.
    #[must_use]
    pub fn wait_asks(&self, count: usize, within: Duration) -> bool {
        let log = lock(&self.shared.log);
        let waited = self
            .shared
            .asked
            .wait_timeout_while(log, within, |l| l.asks < count);
        let (log, _) = waited.unwrap_or_else(PoisonError::into_inner);
        log.asks >= count
    }
}

/// Shows the counts, never an image's bytes.
impl fmt::Debug for FakePictures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let log = lock(&self.shared.log);
        f.debug_struct("FakePictures")
            .field("opens", &log.opens)
            .field("asks", &log.asks)
            .field("queued", &log.queued.len())
            .finish_non_exhaustive()
    }
}

impl Pictures for FakePictures {
    fn open(&self, cfg: PictureConfig) -> Box<dyn PictureRun> {
        let mut log = lock(&self.shared.log);
        log.config = Some(cfg);
        log.opens += 1;
        let shared = Arc::clone(&self.shared);
        Box::new(FakePictureRun { shared })
    }
}

/// A live fake session; dropping it stops the worker.
struct FakePictureRun {
    shared: Arc<Shared>,
}

impl PictureRun for FakePictureRun {
    fn draw(&mut self, ask: PictureAsk) -> Result<Pixels, PictureError> {
        let mut log = lock(&self.shared.log);
        log.asks += 1;
        let side = ask.side;
        log.last_ask = Some(ask);
        self.shared.asked.notify_all();
        log.answer(side)
    }
}

impl Drop for FakePictureRun {
    fn drop(&mut self) {
        lock(&self.shared.log).stops += 1;
    }
}

#[cfg(test)]
mod tests;
