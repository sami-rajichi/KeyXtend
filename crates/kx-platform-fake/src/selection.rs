//! A fake selection watch: the test scripts the focused box, its selection's text and the clipboard, and the fake
//! counts every call, so a test can prove no text was read before a click.

use crate::lock;
use kx_module_api::ports::{Screen, SelConfig, SelError, SelKind, SelRun, SelSpot, SelectionWatch};
use run::FakeSelRun;
use std::fmt;
use std::sync::{Arc, Mutex};

/// The callback a watch tells on each change.
type Tell = Box<dyn Fn() + Send>;

/// How many calls the watch and its runs took, failed looks and reads too; a refused watch and a read in a `Closed` box
/// are not counted.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct SelCalls {
    /// Watches started.
    pub watches: usize,
    /// Runs dropped.
    pub stops: usize,
    /// `look` calls.
    pub looks: usize,
    /// `still` calls.
    pub stills: usize,
    /// `read` calls that reached a box which may be read; one in a `Closed` box is answered at once and not counted.
    pub reads: usize,
    /// `clip_holds` calls.
    pub clips: usize,
}

/// What the watch did, and what the test set for it.
struct Log {
    spot: SelSpot,
    /// The selection's text, which reads give while the spot holds a selection.
    text: String,
    /// The scripted `still` answer; `None` follows the rule.
    still: Option<bool>,
    clip: bool,
    watch_fail: Option<SelError>,
    /// Fails every look until healed.
    look_fail: Option<SelError>,
    /// Fails every read of a box that may be read until healed.
    read_fail: Option<SelError>,
    config: Option<SelConfig>,
    calls: SelCalls,
}

impl Default for Log {
    fn default() -> Self {
        let spot = SelSpot {
            box_id: 0,
            kind: SelKind::Closed,
            selected: false,
            first: None,
            last: None,
            caret: None,
            screen: Screen::default(),
        };
        Self {
            spot,
            text: String::new(),
            still: None,
            clip: false,
            watch_fail: None,
            look_fail: None,
            read_fail: None,
            config: None,
            calls: SelCalls::default(),
        }
    }
}

/// The log, and the live watch's callback, kept apart so a change tells with the log unlocked.
#[derive(Default)]
struct Shared {
    tell: Mutex<Option<Tell>>,
    log: Mutex<Log>,
}

/// A selection watch shared by its clones: focus starts on no box, a `Closed` one with id 0, and the fake gives each
/// new box a fresh id.
///
/// The scripted text, `still` answer, clipboard and failures stay across boxes; a second watch replaces the first's
/// `tell`, which runs under the fake's lock, so it must not change the spot or drop its run.
#[derive(Clone, Default)]
pub struct FakeSelectionWatch {
    shared: Arc<Shared>,
}

impl FakeSelectionWatch {
    /// A watch on no box with nothing scripted.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Moves focus to a new box that looks as `spot` says, under a fresh id in place of `spot.box_id`, and tells the
    /// live watch at once on this thread.
    pub fn focus(&self, spot: SelSpot) {
        let mut log = lock(&self.shared.log);
        let box_id = log.spot.box_id + 1;
        log.spot = SelSpot { box_id, ..spot };
        drop(log);
        self.tell();
    }

    /// Makes the focused box look as `spot` says, keeping its id in place of `spot.box_id`, and tells at once.
    pub fn change(&self, spot: SelSpot) {
        self.change_unheard(spot);
        self.tell();
    }

    /// Changes the focused box as `change` does but tells nothing, as an app that raises no event; looks find it.
    pub fn change_unheard(&self, spot: SelSpot) {
        let mut log = lock(&self.shared.log);
        log.spot = SelSpot {
            box_id: log.spot.box_id,
            ..spot
        };
    }

    /// The focused box's spot now; its id is 0 before any focus.
    #[must_use]
    pub fn spot(&self) -> SelSpot {
        lock(&self.shared.log).spot
    }

    /// Makes the selection hold `text`, which a read gives cut to its cap while the spot holds a selection.
    pub fn set_text(&self, text: &str) {
        text.clone_into(&mut lock(&self.shared.log).text);
    }

    /// Makes `still` answer `answer` for any box, or follow its rule again with `None`: true only for the focused box
    /// while its spot holds a selection.
    pub fn set_still(&self, answer: Option<bool>) {
        lock(&self.shared.log).still = answer;
    }

    /// Makes `clip_holds` answer `holds`; the clipboard starts empty.
    pub fn set_clip(&self, holds: bool) {
        lock(&self.shared.log).clip = holds;
    }

    /// Refuses the next watch with `error`; a debug build stops on one `SelectionWatch::watch` never returns.
    pub fn fail_watch(&self, error: SelError) {
        let ported = matches!(error, SelError::Unavailable | SelError::Refused);
        debug_assert!(ported, "the port never returns {error:?} from a watch");
        lock(&self.shared.log).watch_fail = Some(error);
    }

    /// Fails every look with `error` until healed; a debug build stops on `Closed`, which a look never returns.
    pub fn fail_looks(&self, error: SelError) {
        open_error(error);
        lock(&self.shared.log).look_fail = Some(error);
    }

    /// Fails every read of a box that may be read with `error` until healed; `Closed` comes only from a `Closed` box.
    pub fn fail_reads(&self, error: SelError) {
        open_error(error);
        lock(&self.shared.log).read_fail = Some(error);
    }

    /// Lets looks and reads through again, as an app does once it is no longer busy.
    pub fn heal(&self) {
        let mut log = lock(&self.shared.log);
        log.look_fail = None;
        log.read_fail = None;
    }

    /// Whether a watch is held, from its start until the module drops it.
    #[must_use]
    pub fn watching(&self) -> bool {
        let calls = lock(&self.shared.log).calls;
        calls.watches > calls.stops
    }

    /// How many calls the watch and its runs took.
    #[must_use]
    pub fn calls(&self) -> SelCalls {
        lock(&self.shared.log).calls
    }

    /// The config the latest watch took, kept after it stops.
    #[must_use]
    pub fn config(&self) -> Option<SelConfig> {
        lock(&self.shared.log).config
    }

    /// Tells the live watch, if any, with the log unlocked.
    fn tell(&self) {
        if let Some(tell) = lock(&self.shared.tell).as_ref() {
            tell();
        }
    }
}

/// Stops a debug build on an error a look or a failed read never returns: `Closed` comes only from a `Closed` box.
fn open_error(error: SelError) {
    let ported = error != SelError::Closed;
    debug_assert!(ported, "a look or a failed read never returns {error:?}");
}

impl fmt::Debug for FakeSelectionWatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let log = lock(&self.shared.log);
        f.debug_struct("FakeSelectionWatch")
            .field("spot", &log.spot)
            .field("watching", &(log.calls.watches > log.calls.stops))
            .finish_non_exhaustive()
    }
}

impl SelectionWatch for FakeSelectionWatch {
    fn watch(&self, cfg: SelConfig, tell: Tell) -> Result<Box<dyn SelRun>, SelError> {
        let mut log = lock(&self.shared.log);
        if let Some(error) = log.watch_fail.take() {
            return Err(error);
        }
        log.config = Some(cfg);
        log.calls.watches += 1;
        drop(log);
        *lock(&self.shared.tell) = Some(tell);
        let shared = Arc::clone(&self.shared);
        Ok(Box::new(FakeSelRun { shared }))
    }
}

mod run;

#[cfg(test)]
mod tests;
