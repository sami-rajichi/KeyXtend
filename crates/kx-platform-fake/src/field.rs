//! A fake field watch: the test scripts the focused box and what it holds, and the fake counts every read.

use crate::lock;
use kx_module_api::Redacted;
use kx_module_api::ports::{
    FieldChange, FieldConfig, FieldError, FieldKind, FieldRead, FieldRun, FieldWatch, last_words,
};
use kx_platform::field::after_first;
use std::fmt;
use std::sync::{Arc, Mutex};

/// The callback a watch tells on each focus change.
type Tell = Box<dyn Fn(FieldChange) + Send>;

/// What the watch did, and what the test set for it.
struct Log {
    kind: FieldKind,
    box_id: u64,
    /// The kind of a box focus moved to unheard, which reads see until the next told move.
    unheard: Option<FieldKind>,
    before: Option<String>,
    ends: Option<bool>,
    /// Fails every read until focus moves to a new box.
    read_fail: Option<FieldError>,
    watch_fail: Option<FieldError>,
    config: Option<FieldConfig>,
    before_calls: usize,
    ends_calls: usize,
    /// The text the latest `ends_with` asked about.
    last_ends: Option<Redacted<String>>,
    watches: usize,
    stops: usize,
}

impl Default for Log {
    fn default() -> Self {
        Self {
            kind: FieldKind::None,
            box_id: 0,
            unheard: None,
            before: None,
            ends: None,
            read_fail: None,
            watch_fail: None,
            config: None,
            before_calls: 0,
            ends_calls: 0,
            last_ends: None,
            watches: 0,
            stops: 0,
        }
    }
}

impl Log {
    /// Whether a read may reach the box: only a box told as a text box is asked, as on the real adapter.
    fn readable(&self) -> bool {
        self.kind.reads()
    }

    /// The kind a read finds: the box focus moved to unheard, else the one told.
    fn seen(&self) -> FieldKind {
        self.unheard.unwrap_or(self.kind)
    }
}

/// The log, and the live watch's callback, kept apart so `focus` tells with the log unlocked.
#[derive(Default)]
struct Shared {
    tell: Mutex<Option<Tell>>,
    log: Mutex<Log>,
}

/// A field watch shared by its clones: focus starts on no box, and only a text box is read, the scripted text cut to the
/// read cap and the asked words.
///
/// The scripted text and `ends_with` answer stay across focus changes; a read failure lasts only for its box. A `tell`
/// runs under the fake's lock, so it must not call `focus` or drop its run, and a second watch replaces the first's.
#[derive(Clone, Default)]
pub struct FakeFieldWatch {
    shared: Arc<Shared>,
}

impl FakeFieldWatch {
    /// A watch with focus on no box and nothing scripted.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Moves focus to a new box of `kind`, with a fresh `box_id`, and tells the live watch at once on this thread.
    pub fn focus(&self, kind: FieldKind) {
        let change = {
            let mut log = lock(&self.shared.log);
            log.kind = kind;
            log.box_id += 1;
            log.unheard = None;
            log.read_fail = None;
            FieldChange {
                kind,
                box_id: log.box_id,
            }
        };
        if let Some(tell) = lock(&self.shared.tell).as_ref() {
            tell(change);
        }
    }

    /// Moves focus to a box of `kind` that raises no focus event: `kind` and the module keep the box told last, and
    /// reads find the new one, as the real adapter checks the kind afresh at each read.
    pub fn move_unheard(&self, kind: FieldKind) {
        lock(&self.shared.log).unheard = Some(kind);
    }

    /// Makes a text box hold `text` before its cursor, or not tell with `None`.
    pub fn set_before(&self, text: Option<&str>) {
        lock(&self.shared.log).before = text.map(str::to_owned);
    }

    /// Makes `ends_with` in a text box answer `ends`, whatever text it asks about.
    pub fn set_ends(&self, ends: Option<bool>) {
        lock(&self.shared.log).ends = ends;
    }

    /// Refuses the next watch with `error`; a debug build stops on one `FieldWatch::watch` never returns.
    pub fn fail_watch(&self, error: FieldError) {
        let ported = matches!(error, FieldError::Unavailable | FieldError::Refused);
        debug_assert!(ported, "the port never returns {error:?} from a watch");
        lock(&self.shared.log).watch_fail = Some(error);
    }

    /// Fails every read of the focused box with `error`, `ends_with` answering `None`, until focus moves.
    pub fn fail_reads(&self, error: FieldError) {
        lock(&self.shared.log).read_fail = Some(error);
    }

    /// The focused box's id; 0 before any focus.
    #[must_use]
    pub fn box_id(&self) -> u64 {
        lock(&self.shared.log).box_id
    }

    /// Whether a watch is held, from its start until the module drops it.
    #[must_use]
    pub fn watching(&self) -> bool {
        let log = lock(&self.shared.log);
        log.watches > log.stops
    }

    /// How many watches started.
    #[must_use]
    pub fn watches(&self) -> usize {
        lock(&self.shared.log).watches
    }

    /// The config the latest watch took, kept after it stops.
    #[must_use]
    pub fn config(&self) -> Option<FieldConfig> {
        lock(&self.shared.log).config
    }

    /// How many `before_cursor` calls the runs took, unread and failed ones too.
    #[must_use]
    pub fn before_calls(&self) -> usize {
        lock(&self.shared.log).before_calls
    }

    /// How many `ends_with` calls the runs took, unread ones too.
    #[must_use]
    pub fn ends_calls(&self) -> usize {
        lock(&self.shared.log).ends_calls
    }

    /// The text the latest `ends_with` asked about, unread ones too; `None` before any.
    #[must_use]
    pub fn last_ends(&self) -> Option<Redacted<String>> {
        lock(&self.shared.log).last_ends.clone()
    }
}

impl fmt::Debug for FakeFieldWatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let log = lock(&self.shared.log);
        f.debug_struct("FakeFieldWatch")
            .field("kind", &log.kind)
            .field("box_id", &log.box_id)
            .field("watching", &(log.watches > log.stops))
            .finish_non_exhaustive()
    }
}

impl FieldWatch for FakeFieldWatch {
    fn watch(&self, cfg: FieldConfig, tell: Tell) -> Result<Box<dyn FieldRun>, FieldError> {
        let mut log = lock(&self.shared.log);
        if let Some(error) = log.watch_fail.take() {
            return Err(error);
        }
        log.config = Some(cfg);
        log.watches += 1;
        drop(log);
        *lock(&self.shared.tell) = Some(tell);
        let shared = Arc::clone(&self.shared);
        Ok(Box::new(FakeRun { shared, cfg }))
    }
}

/// The end of `text` one read of at most `chars` characters takes: all of it when it fits, else its last `chars`
/// characters without the first word they may split, as on the real adapter.
fn capped(text: &str, chars: usize) -> &str {
    let skip = text.chars().count().saturating_sub(chars);
    if skip == 0 {
        return text;
    }
    let at = text
        .char_indices()
        .nth(skip)
        .map_or(text.len(), |(at, _)| at);
    after_first(&text[at..])
}

/// A live fake watch; dropping it stops the watch and its tells.
struct FakeRun {
    shared: Arc<Shared>,
    /// The watch's config, whose `read_chars` caps each read.
    cfg: FieldConfig,
}

impl FieldRun for FakeRun {
    fn kind(&self) -> FieldKind {
        lock(&self.shared.log).kind
    }

    fn before_cursor(&mut self, max_words: u8) -> Result<FieldRead, FieldError> {
        let mut log = lock(&self.shared.log);
        log.before_calls += 1;
        let told = FieldRead {
            kind: log.kind,
            text: None,
        };
        if !log.readable() {
            return Ok(told);
        }
        if let Some(error) = log.read_fail {
            return Err(error);
        }
        let kind = log.seen();
        let text = log.before.as_deref().filter(|_| kind.reads());
        let text = text
            .map(|t| last_words(capped(t, usize::from(self.cfg.read_chars)), max_words).to_owned());
        Ok(FieldRead {
            kind,
            text: text.map(Redacted::new),
        })
    }

    fn ends_with(&mut self, text: &Redacted<String>) -> Option<bool> {
        let mut log = lock(&self.shared.log);
        log.ends_calls += 1;
        log.last_ends = Some(text.clone());
        let read = log.readable()
            && log.seen().reads()
            && log.read_fail.is_none()
            && self.cfg.fits(text.expose());
        log.ends.filter(|_| read)
    }
}

impl Drop for FakeRun {
    fn drop(&mut self) {
        let mut log = lock(&self.shared.log);
        log.stops += 1;
        let idle = log.watches == log.stops;
        drop(log);
        if idle {
            let gone = lock(&self.shared.tell).take();
            drop(gone);
        }
    }
}

#[cfg(test)]
mod tests;
