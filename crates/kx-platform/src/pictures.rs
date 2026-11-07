//! `Pictures` over the media worker (ADR-0004, ADR-0034 decision 5): each session's `kx-pictures` thread runs the
//! neutral worker host on the media job, one draw or DIB at a time, and checks each reply. An adapter only starts it.

mod host;

use crate::worker::{Host, Launch, MediaFrames, Session, Spawner, Times};
use kx_ipc::{MAX_PIXELS_FLAG, MEDIA_JOB};
use kx_module_api::ports::{
    DibAsk, PictureAsk, PictureConfig, PictureError, PictureRun, Pictures, Pixels,
};
use kx_module_api::{Clock, Redacted};
use std::sync::Arc;

/// The session thread's name.
const THREAD: &str = "kx-pictures";

impl Launch {
    /// The media job as `cfg` says: `media --max-pixels <n>`, with n at least 1.
    #[must_use]
    pub fn media(cfg: &PictureConfig) -> Self {
        let max = cfg.max_pixels.max(1).to_string();
        Self {
            args: vec![MEDIA_JOB.to_owned(), MAX_PIXELS_FLAG.to_owned(), max],
            memory_mb: cfg.memory_mb,
        }
    }
}

/// The host's times from `cfg`.
const fn times(cfg: &PictureConfig) -> Times {
    Times {
        deadline_ms: cfg.deadline_ms,
        idle_s: cfg.idle_s,
        restart_ms: cfg.restart_ms,
        restart_max_ms: cfg.restart_max_ms,
    }
}

/// Pictures drawn by the worker `S` starts, one `kx-pictures` thread per session.
pub struct WorkerPictures<S> {
    spawner: Arc<S>,
    clock: Arc<dyn Clock>,
}

impl<S: Spawner<MediaFrames>> WorkerPictures<S> {
    /// Pictures over `spawner`, whose sessions read the time from `clock`.
    #[must_use]
    pub fn new(spawner: S, clock: Arc<dyn Clock>) -> Self {
        Self {
            spawner: Arc::new(spawner),
            clock,
        }
    }
}

impl<S: Spawner<MediaFrames>> Pictures for WorkerPictures<S> {
    fn open(&self, cfg: PictureConfig) -> Box<dyn PictureRun> {
        let (spawner, clock) = (Arc::clone(&self.spawner), Arc::clone(&self.clock));
        let host = Host::new(spawner, clock, times(&cfg), Launch::media(&cfg));
        Box::new(Run(Session::open(THREAD, host)))
    }
}

/// A session; dropping it ends its thread, which stops the worker. Without the thread every ask is `Gone`.
struct Run<S: Spawner<MediaFrames>>(Session<S, MediaFrames>);

impl<S: Spawner<MediaFrames>> PictureRun for Run<S> {
    fn draw(&mut self, ask: PictureAsk) -> Result<Pixels, PictureError> {
        let drawn = self.0.run(move |host| host.draw(ask));
        drawn.unwrap_or(Err(PictureError::Gone))
    }

    fn dib(&mut self, ask: DibAsk) -> Result<Redacted<Vec<u8>>, PictureError> {
        let made = self.0.run(move |host| host.dib(ask));
        made.unwrap_or(Err(PictureError::Gone))
    }
}

#[cfg(test)]
mod tests;
