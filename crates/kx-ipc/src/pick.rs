//! The pick job's frames (ADR-0056 decision 6): the host asks for the system's Save or folder dialog, and the job
//! answers the path chosen, a Cancel, or that no usable answer came. Paths and names never print, and both sides check
//! an answer with `PickAsk::check` before anything is written there.

pub(crate) mod ask;
pub(crate) mod reply;
pub(crate) mod wire;

use crate::code::Code;
use crate::error::IpcError;
use crate::limits::snip::{MP4_EXT, PNG_EXT};
use crate::rules::plain_path;
use crate::wire::{PATH, Payload, Sink};
use std::borrow::Cow;
use std::io::Read;

/// What a snip is saved as, which the Save dialog offers.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum FileKind {
    /// A picture.
    Png = 1,
    /// A video.
    Mp4 = 2,
}

impl FileKind {
    /// Its extension in lower case.
    #[must_use]
    pub const fn ext(self) -> &'static str {
        match self {
            Self::Png => PNG_EXT,
            Self::Mp4 => MP4_EXT,
        }
    }
}

impl Code for FileKind {
    const ALL: &'static [Self] = &[Self::Png, Self::Mp4];

    fn code(self) -> u8 {
        self as u8
    }
}

/// Sends a plain path: no control character, and no part that names a device.
pub(crate) fn put_path(s: &mut dyn Sink, text: &str) -> Result<(), IpcError> {
    s.field(PATH, text.as_bytes())?;
    plain_path(text)
}

/// Reads a plain path.
pub(crate) fn read_path<R: Read + ?Sized>(
    p: &mut Payload<'_, R>,
) -> Result<Cow<'static, str>, IpcError> {
    let text = p.text(PATH)?;
    plain_path(&text)?;
    Ok(Cow::Owned(text))
}
