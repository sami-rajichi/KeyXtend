//! A fake clipboard: the test pushes each copy, skip and clear to the live watch, scripts what is at each copied path
//! and the text a take reads, and the fake records the watch, its config and the takes. Its writer records every write.

use crate::lock;
use kx_module_api::ports::{
    ClipConfig, ClipError, ClipNote, ClipRun, ClipWait, Clipboard, FileFact, probe_ok,
};
use kx_module_api::{Redacted, Secret, Zeroizing};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::time::Duration;

pub use writer::FakeClipboardWriter;

/// The callback a watch tells each note.
type Tell = Box<dyn Fn(ClipNote) + Send>;

/// What the watch did, and what the test set for it.
#[derive(Default)]
struct Log {
    config: Option<ClipConfig>,
    watch_fail: Option<ClipError>,
    files: HashMap<PathBuf, FileFact>,
    watches: usize,
    /// The number of the watch whose tell is live: the latest started.
    current: usize,
    stops: usize,
    checks: usize,
    /// The text the clipboard holds now, the next take's failure, how many takes were asked and the last one's cap.
    text: Option<Zeroizing<String>>,
    take_fail: Option<ClipError>,
    takes: usize,
    take_cap: Option<usize>,
}

impl Log {
    /// Whether a watch is held.
    fn watching(&self) -> bool {
        self.watches > self.stops
    }
}

/// The log, the live watch's callback, kept apart so `push` tells with the log unlocked, and the signal a watch gives.
#[derive(Default)]
struct Shared {
    tell: Mutex<Option<Tell>>,
    log: Mutex<Log>,
    watched: Condvar,
}

/// A clipboard shared by its clones: a pushed note reaches the live watch at once, on the test's thread, and is lost
/// with no watch; a path nobody scripted is `Gone`.
///
/// A `tell` runs under the fake's lock, so it must not call the fake or drop its run; a second watch replaces the
/// first's tell, and only the latest watch's drop ends the tells.
#[derive(Clone, Default)]
pub struct FakeClipboard {
    shared: Arc<Shared>,
}

impl FakeClipboard {
    /// A clipboard with no watch and no file.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Tells `note` to the live watch now, on this thread; with no watch it is lost, as a copy nobody watched.
    pub fn push(&self, note: ClipNote) {
        if let Some(tell) = lock(&self.shared.tell).as_ref() {
            tell(note);
        }
    }

    /// Refuses the next watch with `error`; a debug build stops on one `Clipboard::watch` never returns.
    pub fn fail_watch(&self, error: ClipError) {
        let ported = error == ClipError::Refused;
        debug_assert!(ported, "the port never returns {error:?} from a watch");
        lock(&self.shared.log).watch_fail = Some(error);
    }

    /// Makes `files` find `fact` at `path` from now on; a path `probe_ok` refuses still answers unchecked.
    pub fn set_file(&self, path: impl Into<PathBuf>, fact: FileFact) {
        lock(&self.shared.log).files.insert(path.into(), fact);
    }

    /// Whether a watch is held, from its start until the module drops it.
    #[must_use]
    pub fn watching(&self) -> bool {
        lock(&self.shared.log).watching()
    }

    /// Waits up to `within` until a watch is held, so a test never sleeps; true when one is.
    #[must_use]
    pub fn wait_watching(&self, within: Duration) -> bool {
        let log = lock(&self.shared.log);
        let waited = self
            .shared
            .watched
            .wait_timeout_while(log, within, |l| !l.watching());
        let (log, _) = waited.unwrap_or_else(PoisonError::into_inner);
        log.watching()
    }

    /// How many watches started.
    #[must_use]
    pub fn watches(&self) -> usize {
        lock(&self.shared.log).watches
    }

    /// The config the latest watch took, kept after it stops.
    #[must_use]
    pub fn config(&self) -> Option<ClipConfig> {
        lock(&self.shared.log).config
    }

    /// How many `files` calls were asked.
    #[must_use]
    pub fn checks(&self) -> usize {
        lock(&self.shared.log).checks
    }

    /// Makes the clipboard hold `text` from now on, or no text, whatever the pushed notes say.
    pub fn set_text(&self, text: Option<&str>) {
        lock(&self.shared.log).text = text.map(|t| Zeroizing::new(t.to_owned()));
    }

    /// Refuses the next take with `error`.
    pub fn fail_text(&self, error: ClipError) {
        lock(&self.shared.log).take_fail = Some(error);
    }

    /// How many takes were asked.
    #[must_use]
    pub fn takes(&self) -> usize {
        lock(&self.shared.log).takes
    }

    /// The cap the last take asked with.
    #[must_use]
    pub fn take_cap(&self) -> Option<usize> {
        lock(&self.shared.log).take_cap
    }
}

/// Shows the counts, never a path.
impl fmt::Debug for FakeClipboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let log = lock(&self.shared.log);
        f.debug_struct("FakeClipboard")
            .field("watching", &log.watching())
            .field("watches", &log.watches)
            .field("files", &log.files.len())
            .finish_non_exhaustive()
    }
}

impl Clipboard for FakeClipboard {
    fn watch(&self, cfg: ClipConfig, tell: Tell) -> Result<Box<dyn ClipRun>, ClipError> {
        let mut log = lock(&self.shared.log);
        if let Some(error) = log.watch_fail.take() {
            return Err(error);
        }
        // The tell is in place before a waiting test hears of the watch, so its first push lands.
        *lock(&self.shared.tell) = Some(tell);
        log.config = Some(cfg);
        log.watches += 1;
        log.current = log.watches;
        self.shared.watched.notify_all();
        let shared = Arc::clone(&self.shared);
        let id = log.current;
        Ok(Box::new(FakeClipRun { shared, id }))
    }

    /// Answers as scripted, an unknown path as gone, and a path `probe_ok` refuses as unchecked, as the port says.
    fn files(&self, paths: &Redacted<Vec<PathBuf>>) -> Vec<FileFact> {
        let mut log = lock(&self.shared.log);
        log.checks += 1;
        let scripted = |p: &PathBuf| log.files.get(p).copied().unwrap_or(FileFact::Gone);
        let fact = |p: &PathBuf| {
            if probe_ok(p) {
                scripted(p)
            } else {
                FileFact::Unchecked
            }
        };
        paths.expose().iter().map(fact).collect()
    }

    /// Answers the set text, a copy of it, or `TooLong` past `max_chars` as the platform does; a scripted failure
    /// first.
    fn text(&self, _: ClipWait, max_chars: usize) -> Result<Option<Secret>, ClipError> {
        let mut log = lock(&self.shared.log);
        (log.takes, log.take_cap) = (log.takes + 1, Some(max_chars));
        if let Some(error) = log.take_fail.take() {
            return Err(error);
        }
        match &log.text {
            Some(t) if t.chars().count() > max_chars => Err(ClipError::TooLong),
            held => Ok(held.clone().map(Redacted::new)),
        }
    }
}

/// A live fake watch; dropping it stops the watch, and ends the tells when it is the latest.
struct FakeClipRun {
    shared: Arc<Shared>,
    /// The watch's number, counted from 1 in start order.
    id: usize,
}

impl ClipRun for FakeClipRun {}

impl Drop for FakeClipRun {
    fn drop(&mut self) {
        let mut log = lock(&self.shared.log);
        log.stops += 1;
        let gone = (log.current == self.id).then(|| lock(&self.shared.tell).take());
        drop(log);
        drop(gone);
    }
}

mod writer;

#[cfg(test)]
mod tests;
