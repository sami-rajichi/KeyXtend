//! The clipboard ports (ADR-0034 decision 3): `Clipboard` watches copies, which only `ReadClipboard` reaches, and
//! `ClipboardWriter` sets the clipboard, which only `WriteClipboard` reaches. Copied content always stays redacted.

use crate::{Capability, Redacted, ServiceId, ServiceKey};
use std::fmt;
use std::path::PathBuf;
use thiserror::Error;

/// How an image's bytes are coded.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ImageCodec {
    /// PNG.
    Png,
    /// A Windows device-independent bitmap, as `CF_DIB` holds it; macOS adds TIFF in P15.
    Dib,
}

/// One clipboard format; its content always prints redacted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClipPart {
    /// Plain text.
    Text(Redacted<String>),
    /// HTML, as the platform's bytes.
    Html(Redacted<Vec<u8>>),
    /// RTF, as bytes.
    Rtf(Redacted<Vec<u8>>),
    /// An image.
    Image {
        /// How its bytes are coded.
        codec: ImageCodec,
        /// Its bytes.
        bytes: Redacted<Vec<u8>>,
    },
    /// Files and folders by path, never their content.
    Files(Redacted<Vec<PathBuf>>),
}

/// The app that made a copy or a clear; its `Debug` hides both names, as they tell what the user works in.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct AppRef {
    /// The app's stable id; on Windows its exe file name in lower case.
    pub id: String,
    /// The app's name to show.
    pub name: String,
}

/// Shows only that there is an app.
impl fmt::Debug for AppRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppRef")
            .field("id", &Redacted::new(&self.id))
            .field("name", &Redacted::new(&self.name))
            .finish()
    }
}

/// What a watch tells; only a copy carries content, and an app is `None` when the platform cannot tell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClipNote {
    /// A copy, with its formats.
    Copied {
        /// The formats read, the richest first.
        parts: Vec<ClipPart>,
        /// The app that copied.
        app: Option<AppRef>,
    },
    /// A copy with a "leave out of history" marker, left unread.
    Marked {
        /// The app that copied.
        app: Option<AppRef>,
    },
    /// The clipboard was emptied, or now holds only blank text.
    Cleared {
        /// The app that cleared it.
        app: Option<AppRef>,
    },
    /// A copy over `ClipConfig::max_bytes`, left unread.
    TooBig {
        /// The app that copied.
        app: Option<AppRef>,
        /// At least how many bytes the copy holds: the count stops once it passes the cap.
        bytes: u64,
    },
    /// The clipboard stayed locked past `ClipWait::open_ms`, so this copy was missed.
    Busy,
}

/// How long the platform waits for another app to let go of the clipboard; the module fills it from its settings.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ClipWait {
    /// The longest wait, in ms.
    pub open_ms: u32,
    /// How often to try again within that wait, in ms.
    pub poll_ms: u32,
}

/// How the platform reads the clipboard; the module fills it from its settings.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ClipConfig {
    /// The most bytes one copy may hold to be read.
    pub max_bytes: u64,
    /// The wait to open the clipboard.
    pub wait: ClipWait,
}

/// What the platform finds at a copied path.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FileFact {
    /// It is there, this many bytes long.
    Size(u64),
    /// Nothing is there any more.
    Gone,
    /// Its drive or share is out of reach now, so the row stays and an insert skips it.
    Away,
}

/// Why the clipboard could not be watched or set.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum ClipError {
    /// Another app held the clipboard past `ClipWait::open_ms`.
    #[error("another app holds the clipboard")]
    Busy,
    /// The OS refused.
    #[error("the system refused the clipboard")]
    Refused,
    /// A part could not be read or written.
    #[error("the clipboard data could not be read or written")]
    Io,
}

/// A running watch on the platform's side. Dropping it stops the watch.
pub trait ClipRun: Send {}

/// Watches the system clipboard; the platform provides it.
pub trait Clipboard: Send + Sync {
    /// Starts a watch as `cfg` says; `tell` hears each copy, skip and clear, on any thread, and never hears our own
    /// writes, which carry the marker.
    ///
    /// # Errors
    /// `Refused` when the OS refuses to watch.
    fn watch(
        &self,
        cfg: ClipConfig,
        tell: Box<dyn Fn(ClipNote) + Send>,
    ) -> Result<Box<dyn ClipRun>, ClipError>;

    /// What is at each of `paths` now, in the same order.
    fn files(&self, paths: &Redacted<Vec<PathBuf>>) -> Vec<FileFact>;
}

/// Sets the system clipboard; the platform provides it.
pub trait ClipboardWriter: Send + Sync {
    /// Sets the clipboard to `parts` with our "leave out of history" marker; files always go as a copy, never a move.
    ///
    /// # Errors
    /// `Busy` when another app holds the clipboard past `wait.open_ms`, `Refused` when the OS refuses, `Io` when a
    /// part cannot be written.
    fn write(&self, parts: &[ClipPart], wait: ClipWait) -> Result<(), ClipError>;
}

/// The clipboard watch service; it needs `ReadClipboard`.
pub struct ClipboardKey;

impl ServiceKey for ClipboardKey {
    type Api = dyn Clipboard;
    const ID: ServiceId = ServiceId::new("clipboard");
    const CAPABILITY: Option<Capability> = Some(Capability::ReadClipboard);
}

/// The clipboard writer service; it needs `WriteClipboard`.
pub struct ClipboardWriterKey;

impl ServiceKey for ClipboardWriterKey {
    type Api = dyn ClipboardWriter;
    const ID: ServiceId = ServiceId::new("clipboard-writer");
    const CAPABILITY: Option<Capability> = Some(Capability::WriteClipboard);
}

#[cfg(test)]
mod tests;
