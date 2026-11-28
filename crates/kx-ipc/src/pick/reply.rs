//! What the pick job answers: the path the user chose, a Cancel, a refused answer, or a dialog that failed.

use super::{put_path, read_path};
use crate::code::Kind;
use crate::error::IpcError;
use crate::frame::{Head, write_head};
use crate::wire::{Payload, Put, Sink, Size};
use std::borrow::Cow;
use std::fmt;
use std::io::{Read, Write};

/// A message from the pick job to the host, answering one dialog; it borrows its path when written and owns it when
/// read.
#[derive(Clone, PartialEq, Eq)]
pub enum PickReply<'a> {
    /// The path the user chose, which passed `PickAsk::check` in the job; the host checks it again before any write.
    Picked {
        /// The dialog this answers.
        id: u32,
        /// The file or folder chosen.
        path: Cow<'a, str>,
    },
    /// The user cancelled the dialog (edge 6).
    Cancelled {
        /// The dialog this answers.
        id: u32,
    },
    /// The user chose a path `PickAsk::check` refuses, such as a network place; the path is never told.
    Refused {
        /// The dialog this answers.
        id: u32,
    },
    /// The dialog could not show, or its answer could not be read.
    Failed {
        /// The dialog this answers.
        id: u32,
        /// The system's error code (an `HRESULT`'s bits), or `NO_CODE`; safe to log, as it names no path.
        code: u32,
    },
}

/// The kinds a pick reply may be.
pub(crate) const KINDS: &[Kind] = &[
    Kind::Picked,
    Kind::PickCancelled,
    Kind::PickRefused,
    Kind::PickFailed,
];

impl PickReply<'_> {
    /// The code of a failure the system gave none for, such as a panic: no `HRESULT` failure is 0.
    pub const NO_CODE: u32 = 0;

    /// Reads one answer from the pick job.
    ///
    /// # Errors
    /// Any `IpcError`, `Closed` when the job ended between frames; the stream is out of step after any other.
    pub fn read<R: Read + ?Sized>(r: &mut R) -> Result<PickReply<'static>, IpcError> {
        let head = Head::read(r, KINDS)?;
        let (id, mut p) = (head.id, Payload::new(r, head.len));
        let reply = match head.kind {
            Kind::Picked => PickReply::Picked {
                id,
                path: read_path(&mut p)?,
            },
            Kind::PickCancelled => PickReply::Cancelled { id },
            Kind::PickRefused => PickReply::Refused { id },
            Kind::PickFailed => PickReply::Failed { id, code: p.u32()? },
            _ => return Err(IpcError::BadKind),
        };
        p.end()?;
        Ok(reply)
    }

    /// Writes this answer as one frame, then flushes.
    ///
    /// # Errors
    /// `BadField`, `TooLong` or `Inconsistent`, before any byte is written, for a path the host would refuse; `Io`
    /// when the stream fails.
    pub fn write<W: Write + ?Sized>(&self, w: &mut W) -> Result<(), IpcError> {
        let mut size = Size::default();
        self.put(&mut size)?;
        write_head(w, self.kind(), self.id(), size.0)?;
        self.put(&mut Put(w))?;
        w.flush()?;
        Ok(())
    }

    /// Runs this answer's payload through `s`.
    fn put(&self, s: &mut dyn Sink) -> Result<(), IpcError> {
        match self {
            Self::Picked { path, .. } => put_path(s, path),
            Self::Failed { code, .. } => s.u32(*code),
            Self::Cancelled { .. } | Self::Refused { .. } => Ok(()),
        }
    }

    const fn kind(&self) -> Kind {
        match self {
            Self::Picked { .. } => Kind::Picked,
            Self::Cancelled { .. } => Kind::PickCancelled,
            Self::Refused { .. } => Kind::PickRefused,
            Self::Failed { .. } => Kind::PickFailed,
        }
    }

    /// The dialog this answers.
    #[must_use]
    pub const fn id(&self) -> u32 {
        match self {
            Self::Picked { id, .. }
            | Self::Cancelled { id }
            | Self::Refused { id }
            | Self::Failed { id, .. } => *id,
        }
    }
}

impl fmt::Debug for PickReply<'_> {
    /// Shows ids and a failure's code in hex, never a path.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = self.id();
        match self {
            Self::Picked { .. } => f
                .debug_struct("Picked")
                .field("id", &id)
                .finish_non_exhaustive(),
            Self::Cancelled { .. } => f.debug_struct("Cancelled").field("id", &id).finish(),
            Self::Refused { .. } => f.debug_struct("Refused").field("id", &id).finish(),
            Self::Failed { code, .. } => f
                .debug_struct("Failed")
                .field("id", &id)
                .field("code", &format_args!("{code:#010x}"))
                .finish(),
        }
    }
}
