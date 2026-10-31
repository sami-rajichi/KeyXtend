//! A fake pictures service: each draw or DIB takes the answer the test queued for its kind, the latest repeating, and
//! the fake records the session's config and counts the asks. It never sleeps: a hung worker is a queued `Timeout`.

use crate::lock;
use kx_module_api::Redacted;
use kx_module_api::ports::{
    DibAsk, PictureAsk, PictureConfig, PictureError, PictureRun, Pictures, Pixels,
};
use std::collections::VecDeque;
use std::fmt;
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::time::Duration;

/// The answers queued for one kind of ask: the next in turn, else the latest again, else `Gone`.
struct Answers<T> {
    queued: VecDeque<Result<T, PictureError>>,
    last: Option<Result<T, PictureError>>,
}

impl<T> Default for Answers<T> {
    fn default() -> Self {
        Self {
            queued: VecDeque::new(),
            last: None,
        }
    }
}

impl<T: Clone> Answers<T> {
    /// The next queued answer, else the latest, else `Gone`.
    fn next(&mut self) -> Result<T, PictureError> {
        if let Some(next) = self.queued.pop_front() {
            self.last = Some(next);
        }
        self.last.clone().unwrap_or(Err(PictureError::Gone))
    }
}

/// What the sessions did, and what the test set for them.
#[derive(Default)]
struct Log {
    draws: Answers<Pixels>,
    dibs: Answers<Vec<u8>>,
    last_ask: Option<PictureAsk>,
    last_dib: Option<DibAsk>,
    config: Option<PictureConfig>,
    opens: usize,
    stops: usize,
    asks: usize,
    dib_asks: usize,
}

/// The log, and the signal each ask gives, for a test waiting on another thread.
#[derive(Default)]
struct Shared {
    log: Mutex<Log>,
    asked: Condvar,
}

/// Pictures shared by their clones: each draw or DIB of any session takes the next answer queued for its kind, the
/// latest repeating once they run out, and `Gone` before any.
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
        lock(&self.shared.log).draws.queued.push_back(drawn);
    }

    /// Queues `made` for a later DIB, given back as it is, even past the ask's cap.
    pub fn queue_dib(&self, made: Result<Vec<u8>, PictureError>) {
        lock(&self.shared.log).dibs.queued.push_back(made);
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

    /// How many DIBs were asked, failed ones too.
    #[must_use]
    pub fn dib_asks(&self) -> usize {
        lock(&self.shared.log).dib_asks
    }

    /// The latest draw's ask, failed or not.
    #[must_use]
    pub fn last_ask(&self) -> Option<PictureAsk> {
        lock(&self.shared.log).last_ask.clone()
    }

    /// The latest DIB's ask, failed or not.
    #[must_use]
    pub fn last_dib(&self) -> Option<DibAsk> {
        lock(&self.shared.log).last_dib.clone()
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
            .field("dib_asks", &log.dib_asks)
            .field("queued", &log.draws.queued.len())
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
    /// Pixels past the ask's side are `Gone`, as bad pixels are.
    fn draw(&mut self, ask: PictureAsk) -> Result<Pixels, PictureError> {
        let mut log = lock(&self.shared.log);
        log.asks += 1;
        let side = ask.side;
        log.last_ask = Some(ask);
        self.shared.asked.notify_all();
        let drawn = log.draws.next()?;
        let fits = drawn.width().max(drawn.height()) <= side;
        fits.then_some(drawn).ok_or(PictureError::Gone)
    }

    fn dib(&mut self, ask: DibAsk) -> Result<Redacted<Vec<u8>>, PictureError> {
        let mut log = lock(&self.shared.log);
        log.dib_asks += 1;
        log.last_dib = Some(ask);
        self.shared.asked.notify_all();
        log.dibs.next().map(Redacted::new)
    }
}

impl Drop for FakePictureRun {
    fn drop(&mut self) {
        lock(&self.shared.log).stops += 1;
    }
}

#[cfg(test)]
mod tests;
