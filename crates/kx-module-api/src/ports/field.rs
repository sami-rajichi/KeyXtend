//! The focused box (ADR-0033): its kind on each focus change, and at most a few words before its cursor, which only
//! `ReadFocusedText` reaches. Every text read stays in `Redacted`.

use crate::{Capability, Redacted, ServiceId, ServiceKey};
use thiserror::Error;

/// What kind of box has focus.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum FieldKind {
    /// A box for plain text.
    Text,
    /// A password box.
    Password,
    /// A box whose kind the app does not tell.
    Unknown,
    /// No box has focus.
    None,
}

impl FieldKind {
    /// True when a box of this kind may be read: only a text box is, on every platform.
    #[must_use]
    pub fn reads(self) -> bool {
        self == Self::Text
    }
}

/// Told on each focus change; a new `box_id` means a new box.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FieldChange {
    /// The kind of the box with focus now.
    pub kind: FieldKind,
    /// Names the box, so a module notices a new one.
    pub box_id: u64,
}

/// How the platform reads the box; the module fills it from its settings.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FieldConfig {
    /// The longest one read may take, in ms.
    pub read_ms: u32,
    /// The longest text one read takes before the cursor, in characters; a longer one is cut to whole words.
    pub read_chars: u16,
}

impl FieldConfig {
    /// True when `text` holds at most `read_chars` characters, so one read can check that the box ends with it.
    #[must_use]
    pub fn fits(&self, text: &str) -> bool {
        text.chars().count() <= usize::from(self.read_chars)
    }
}

/// What one read of the focused box found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldRead {
    /// The kind of the box the platform saw at the read, or told last when nothing was read.
    pub kind: FieldKind,
    /// At most the asked words before the cursor; `None` when the app does not tell, or the box is no text box.
    pub text: Option<Redacted<String>>,
}

/// Why the box could not be watched or read.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum FieldError {
    /// The platform cannot watch or read boxes.
    #[error("the system cannot read the focused box")]
    Unavailable,
    /// A read took longer than `FieldConfig::read_ms`.
    #[error("reading the focused box took too long")]
    TimedOut,
    /// The OS or the app refused.
    #[error("the app refused to tell what its box holds")]
    Refused,
}

/// A running watch on the platform's side. Dropping it stops the watch.
pub trait FieldRun: Send {
    /// The kind of the box with focus now.
    fn kind(&self) -> FieldKind;

    /// At most `max_words` words before the cursor, read within `FieldConfig::read_ms`, with the kind of box the read
    /// saw; no text when the app does not tell, or the box is a password or unknown box.
    ///
    /// # Errors
    /// `TimedOut` when the read takes too long, `Refused` when the app refuses, `Unavailable` when nothing can read.
    fn before_cursor(&mut self, max_words: u8) -> Result<FieldRead, FieldError>;

    /// Whether the text just before the cursor ends with `text`; `None` when it cannot be read.
    fn ends_with(&mut self, text: &Redacted<String>) -> Option<bool>;
}

/// Watches the focused box in other apps; the platform provides it.
pub trait FieldWatch: Send + Sync {
    /// Starts a watch as `cfg` says; `tell` hears each focus change, on any thread. It runs under the run's lock, so a
    /// caller must not drop the run while holding a lock that its `tell` also takes.
    ///
    /// # Errors
    /// `Unavailable` when the platform cannot watch, `Refused` when the OS refuses.
    fn watch(
        &self,
        cfg: FieldConfig,
        tell: Box<dyn Fn(FieldChange) + Send>,
    ) -> Result<Box<dyn FieldRun>, FieldError>;
}

/// The end of `text` from the start of its last `max` words, with what follows them; empty when it keeps none.
/// Every field watch cuts with it, so the fake and the platforms agree.
#[must_use]
pub fn last_words(text: &str, max: u8) -> &str {
    let mut starts = Vec::new();
    let mut gap = true;
    for (at, c) in text.char_indices() {
        if gap && !c.is_whitespace() {
            starts.push(at);
        }
        gap = c.is_whitespace();
    }
    let skip = starts.len().saturating_sub(usize::from(max));
    let from = starts.get(skip).copied().unwrap_or(text.len());
    &text[from..]
}

/// The field watch service; it needs `ReadFocusedText`.
pub struct FieldWatchKey;

impl ServiceKey for FieldWatchKey {
    type Api = dyn FieldWatch;
    const ID: ServiceId = ServiceId::new("field-watch");
    const CAPABILITY: Option<Capability> = Some(Capability::ReadFocusedText);
}

#[cfg(test)]
mod tests;
