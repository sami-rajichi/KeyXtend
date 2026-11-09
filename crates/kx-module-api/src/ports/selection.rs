//! The selection port (ADR-0041): what kind of box has focus, whether its selection holds text, and where its lines and
//! caret are, told on events and never on a timer, with no text. Only Translate's one read at a click takes text, which
//! stays in `Redacted`.

mod board;
mod events;

pub use board::{Board, BoardPlace, BoardPlaceAsk};
pub use events::{ChipAct, ChipsView, Edit, EditAsk, EditDone, Legend, SelChip, StripSpot};

use super::{Rect, Screen};
use crate::{Capability, Redacted, ServiceId, ServiceKey};
use std::fmt;
use thiserror::Error;

listed! {
    /// What kind of box has focus, which sets the chips it shows; the adapter's tables say which control is which.
    pub enum SelKind ("Every kind.") {
        /// A box the user types into: Copy, Cut, Paste and Translate.
        Editable,
        /// Text that cannot be edited, such as a web page: Copy and Translate.
        ReadOnly,
        /// A terminal: Copy, Paste and Translate, with keys that never stop a program.
        Terminal,
        /// A password box, a box of unknown kind, one of our windows, or no box: no chips and no read.
        Closed,
    }
}

/// How the platform watches the box; the module fills it from its settings.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SelConfig {
    /// How long the watch waits after an event before it tells, so a burst tells once and the app has settled, in ms.
    pub settle_ms: u32,
    /// The longest one look or read may take, in ms.
    pub read_ms: u32,
    /// The most changes told in one second, so at most this many looks follow; later ones are told once, after.
    pub looks_per_s: u16,
}

/// Whether box ids `a` and `b` name the same box; a box with no id (0) is never the same, as two cannot be told apart.
#[must_use]
pub fn same_box(a: u64, b: u64) -> bool {
    a != 0 && a == b
}

/// What one look at the focused box found, with no text; its `Debug` hides the boxes.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct SelSpot {
    /// Names the box, as `FieldChange::box_id` does.
    pub box_id: u64,
    /// What kind of box it is.
    pub kind: SelKind,
    /// Its selection holds text; false at a bare caret.
    pub selected: bool,
    /// The selection's first line box, in physical pixels; `None` with no selection or when the app does not tell.
    pub first: Option<Rect>,
    /// The selection's last line box: the first again for a selection on one line.
    pub last: Option<Rect>,
    /// The caret's box; `None` when the app does not tell.
    pub caret: Option<Rect>,
    /// The screen that holds the selection or the caret: its work area and its DPI.
    pub screen: Screen,
}

/// Shows the box's id, kind and flag and the screen, and only whether each box is known.
impl fmt::Debug for SelSpot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SelSpot")
            .field("box_id", &self.box_id)
            .field("kind", &self.kind)
            .field("selected", &self.selected)
            .field("first", &self.first.map(Redacted::new))
            .field("last", &self.last.map(Redacted::new))
            .field("caret", &self.caret.map(Redacted::new))
            .field("screen", &self.screen)
            .finish()
    }
}

/// Why the box could not be watched, looked at or read; no variant holds text.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum SelError {
    /// The platform cannot watch selections.
    #[error("the system cannot tell the focused box's selection")]
    Unavailable,
    /// A look or a read took longer than `SelConfig::read_ms`.
    #[error("looking at the focused box took too long")]
    TimedOut,
    /// The OS or the app refused.
    #[error("the app refused to tell its selection")]
    Refused,
    /// The box is `Closed`, so it is never read.
    #[error("the focused box may not be read")]
    Closed,
}

/// A running watch on the platform's side. Dropping it stops the watch.
pub trait SelRun: Send {
    /// The focused box now, looked at within `SelConfig::read_ms`, with no text.
    ///
    /// # Errors
    /// `TimedOut` when the app takes too long, `Refused` when it refuses, `Unavailable` when nothing can look.
    fn look(&mut self) -> Result<SelSpot, SelError>;

    /// True when the last answered look found focus in one of our own windows, so it gave the box kept from before;
    /// no key may go there.
    fn ours(&self) -> bool;

    /// True when box `box_id` still has focus and its selection still holds text; false on any doubt, so nothing is sent.
    fn still(&mut self, box_id: u64) -> bool;

    /// Box `box_id`'s selected text from its start, at most `max_chars` characters, read once within
    /// `SelConfig::read_ms`; only Translate's click asks, for the box its `still` just confirmed.
    ///
    /// # Errors
    /// `Closed` in a `Closed` box or when the focused box is not `box_id`; `TimedOut`, `Refused` or `Unavailable` as for
    /// `look`.
    fn read(&mut self, box_id: u64, max_chars: u16) -> Result<Redacted<String>, SelError>;

    /// True when the clipboard holds any format; it reads none of them.
    fn clip_holds(&self) -> bool;
}

/// Watches the focused box's selection in other apps; the platform provides it.
pub trait SelectionWatch: Send + Sync {
    /// Starts a watch as `cfg` says; `tell` hears each focus or selection change, once settled, on any thread.
    ///
    /// # Errors
    /// `Unavailable` when the platform cannot watch, `Refused` when the OS refuses.
    fn watch(
        &self,
        cfg: SelConfig,
        tell: Box<dyn Fn() + Send>,
    ) -> Result<Box<dyn SelRun>, SelError>;
}

/// The selection watch service; it needs `ReadFocusedText` (ADR-0041 decision 3).
pub struct SelectionWatchKey;

impl ServiceKey for SelectionWatchKey {
    type Api = dyn SelectionWatch;
    const ID: ServiceId = ServiceId::new("selection-watch");
    const CAPABILITY: Option<Capability> = Some(Capability::ReadFocusedText);
}

#[cfg(test)]
mod tests;
