//! What the host sends the pick job: show the Save dialog, show the folder dialog, or goodbye.

use super::wire::NAME;
use super::{FileKind, put_path, read_path};
use crate::code::{Code, Kind};
use crate::error::IpcError;
use crate::frame::{Head, write_head};
use crate::rules::{PathRoot, file_name, rooted_path, typed};
use crate::wire::{Payload, Put, Sink, Size};
use std::borrow::Cow;
use std::fmt;
use std::io::{Read, Write};

/// A message from the host to the pick job; it borrows its paths when written and owns them when read. Every ask but
/// `Bye` carries its dialog's id, which the answer repeats.
#[derive(Clone, PartialEq, Eq)]
pub enum PickAsk<'a> {
    /// Show the Save dialog at `folder`, proposing `name` as `kind`; the answer is the file chosen.
    Save {
        /// The dialog's id.
        id: u32,
        /// The folder it opens at, a plain path.
        folder: Cow<'a, str>,
        /// The name it proposes: no control character, separator, mark Windows refuses or device name.
        name: Cow<'a, str>,
        /// The only kind it offers.
        kind: FileKind,
    },
    /// Show the folder dialog at `at`; the answer is the folder chosen.
    Folder {
        /// The dialog's id.
        id: u32,
        /// The folder it opens at, a plain path.
        at: Cow<'a, str>,
    },
    /// End: the pick job exits after this frame.
    Bye,
}

/// The kinds a pick ask may be.
pub(crate) const KINDS: &[Kind] = &[Kind::PickSave, Kind::PickFolder, Kind::PickBye];

impl PickAsk<'_> {
    /// Reads one ask from the host.
    ///
    /// # Errors
    /// Any `IpcError`, `Closed` when the host ended between frames; the stream is out of step after any other.
    pub fn read<R: Read + ?Sized>(r: &mut R) -> Result<PickAsk<'static>, IpcError> {
        let head = Head::read(r, KINDS)?;
        let (id, mut p) = (head.id, Payload::new(r, head.len));
        let ask = match head.kind {
            Kind::PickSave => read_save(&mut p, id)?,
            Kind::PickFolder => PickAsk::Folder {
                id,
                at: read_path(&mut p)?,
            },
            Kind::PickBye if id == 0 => PickAsk::Bye,
            Kind::PickBye => return Err(IpcError::Inconsistent),
            _ => return Err(IpcError::BadKind),
        };
        p.end()?;
        Ok(ask)
    }

    /// Writes this ask as one frame, then flushes.
    ///
    /// # Errors
    /// `BadField`, `TooLong` or `Inconsistent`, before any byte is written, for an ask the job would refuse; `Io`
    /// when the stream fails.
    pub fn write<W: Write + ?Sized>(&self, w: &mut W) -> Result<(), IpcError> {
        let mut size = Size::default();
        self.put(&mut size)?;
        write_head(w, self.kind(), self.id(), size.0)?;
        self.put(&mut Put(w))?;
        w.flush()?;
        Ok(())
    }

    /// Runs this ask's payload through `s`.
    fn put(&self, s: &mut dyn Sink) -> Result<(), IpcError> {
        match self {
            Self::Save {
                folder, name, kind, ..
            } => {
                s.byte(kind.code())?;
                put_path(s, folder)?;
                s.field(NAME, name.as_bytes())?;
                file_name(name)
            }
            Self::Folder { at, .. } => put_path(s, at),
            Self::Bye => Ok(()),
        }
    }

    const fn kind(&self) -> Kind {
        match self {
            Self::Save { .. } => Kind::PickSave,
            Self::Folder { .. } => Kind::PickFolder,
            Self::Bye => Kind::PickBye,
        }
    }

    /// The dialog this asks for; goodbye's is 0.
    #[must_use]
    pub const fn id(&self) -> u32 {
        match self {
            Self::Save { id, .. } | Self::Folder { id, .. } => *id,
            Self::Bye => 0,
        }
    }

    /// Refuses `answer` unless it is an absolute path in this system's style (`PathRoot::NATIVE`) of what this asks:
    /// a file of its kind for a Save, a folder for a folder ask. The job checks before it answers and the host again
    /// before anything is written there.
    ///
    /// # Errors
    /// `BadField` for any other answer, and for every answer to goodbye.
    pub fn check(&self, answer: &str) -> Result<(), IpcError> {
        self.check_in(PathRoot::NATIVE, answer)
    }

    /// `check` in `root`'s style, so both styles are tested on every system.
    pub(crate) fn check_in(&self, root: PathRoot, answer: &str) -> Result<(), IpcError> {
        match self {
            Self::Save { kind, .. } => {
                rooted_path(root, answer).and_then(|()| typed(answer, kind.ext()))
            }
            Self::Folder { .. } => rooted_path(root, answer),
            Self::Bye => Err(IpcError::BadField),
        }
    }
}

/// Reads a Save ask's kind, folder and name.
fn read_save<R: Read + ?Sized>(
    p: &mut Payload<'_, R>,
    id: u32,
) -> Result<PickAsk<'static>, IpcError> {
    let kind = FileKind::from_code(p.byte()?).ok_or(IpcError::BadField)?;
    let folder = read_path(p)?;
    let name = p.text(NAME)?;
    file_name(&name)?;
    Ok(PickAsk::Save {
        id,
        folder,
        name: Cow::Owned(name),
        kind,
    })
}

impl fmt::Debug for PickAsk<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = self.id();
        match self {
            Self::Save { kind, .. } => f
                .debug_struct("Save")
                .field("id", &id)
                .field("kind", kind)
                .finish_non_exhaustive(),
            Self::Folder { .. } => f
                .debug_struct("Folder")
                .field("id", &id)
                .finish_non_exhaustive(),
            Self::Bye => f.write_str("Bye"),
        }
    }
}
