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

    /// At most `max_words` words before the cursor, read within `FieldConfig::read_ms`; `None` when the app does not
    /// tell, or the box is a password or unknown box.
    ///
    /// # Errors
    /// `TimedOut` when the read takes too long, `Refused` when the app refuses, `Unavailable` when nothing can read.
    fn before_cursor(&mut self, max_words: u8) -> Result<Option<Redacted<String>>, FieldError>;

    /// Whether the text just before the cursor ends with `text`; `None` when it cannot be read.
    fn ends_with(&mut self, text: &Redacted<String>) -> Option<bool>;
}

/// Watches the focused box in other apps; the platform provides it.
pub trait FieldWatch: Send + Sync {
    /// Starts a watch as `cfg` says; `tell` hears each focus change, on any thread.
    ///
    /// # Errors
    /// `Unavailable` when the platform cannot watch, `Refused` when the OS refuses.
    fn watch(
        &self,
        cfg: FieldConfig,
        tell: Box<dyn Fn(FieldChange) + Send>,
    ) -> Result<Box<dyn FieldRun>, FieldError>;
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
