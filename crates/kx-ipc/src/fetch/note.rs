//! What the fetch job sends on its own: its progress, then that the file is checked and named, or one fixed fault.

use super::FetchFault;
use super::wire::{PROGRESS_LEN, SIZE};
use crate::code::{Code, Kind};
use crate::error::IpcError;
use crate::frame::{Head, write_head};
use crate::limits::FAILED_LEN;
use crate::rules::ok_if;
use crate::wire::{Payload, Put, Sink};
use std::io::{Read, Write};

/// A message from the fetch job to the host; each repeats the id of its download.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FetchNote {
    /// The bytes in the part so far, out of the file's size.
    Progress {
        /// The download.
        id: u32,
        /// At most `total`.
        got: u64,
        /// Within `SIZE`, as the file's size is.
        total: u64,
    },
    /// The file's size and SHA-256 passed and it has its final name.
    Done {
        /// The download.
        id: u32,
    },
    /// The download stopped, for a fixed reason.
    Fault {
        /// The download.
        id: u32,
        /// Why.
        fault: FetchFault,
    },
}

/// The kinds a fetch note may be.
pub(crate) const KINDS: &[Kind] = &[Kind::Progress, Kind::Fetched, Kind::FetchFailed];

impl FetchNote {
    /// Reads one note from the fetch job.
    ///
    /// # Errors
    /// Any `IpcError`, `Closed` when the job ended between frames; the stream is out of step after any other.
    pub fn read<R: Read + ?Sized>(r: &mut R) -> Result<Self, IpcError> {
        let head = Head::read(r, KINDS)?;
        let (id, mut p) = (head.id, Payload::new(r, head.len));
        let note = match head.kind {
            Kind::Progress => {
                let (got, total) = (p.u64()?, p.u64()?);
                progress(got, total)?;
                Self::Progress { id, got, total }
            }
            Kind::Fetched => Self::Done { id },
            Kind::FetchFailed => {
                let fault = FetchFault::from_code(p.byte()?).ok_or(IpcError::BadField)?;
                Self::Fault { id, fault }
            }
            _ => return Err(IpcError::BadKind),
        };
        p.end()?;
        Ok(note)
    }

    /// Writes this note as one frame, then flushes.
    ///
    /// # Errors
    /// `BadField`, before any byte is written, for progress past its total or a total outside a model's size; `Io`
    /// when the stream fails.
    pub fn write<W: Write + ?Sized>(&self, w: &mut W) -> Result<(), IpcError> {
        let mut put = Put(w);
        match self {
            Self::Progress { id, got, total } => {
                progress(*got, *total)?;
                write_head(put.0, Kind::Progress, *id, PROGRESS_LEN)?;
                put.u64(*got)?;
                put.u64(*total)?;
            }
            Self::Done { id } => write_head(put.0, Kind::Fetched, *id, 0)?,
            Self::Fault { id, fault } => {
                write_head(put.0, Kind::FetchFailed, *id, FAILED_LEN)?;
                put.byte(fault.code())?;
            }
        }
        put.0.flush()?;
        Ok(())
    }

    /// The download this note belongs to.
    #[must_use]
    pub const fn id(&self) -> u32 {
        match self {
            Self::Progress { id, .. } | Self::Done { id } | Self::Fault { id, .. } => *id,
        }
    }
}

/// Refuses progress past its total, or a total outside `SIZE`, as a download's size starts at 1.
fn progress(got: u64, total: u64) -> Result<(), IpcError> {
    ok_if(got <= total && SIZE.contains(&total))
}
