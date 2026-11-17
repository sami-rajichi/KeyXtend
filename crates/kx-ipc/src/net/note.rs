//! What the `speech-net` job sends on its own: each piece's text in order, then that the run is done or the key works,
//! or one fault.

use super::NetFault;
use super::body::piece_index;
use super::code::FaultCode;
use super::wire::TEXT;
use crate::code::{Code, Kind};
use crate::error::IpcError;
use crate::frame::{Head, check_len, write_head};
use crate::limits::NET_RENEW_S;
use crate::rules::within;
use crate::wire::{Payload, Put, Sink, Size};
use std::io::{Read, Write};
use zeroize::Zeroizing;

/// A message from the `speech-net` job to the host; each repeats the id of its run or test.
#[derive(Clone, PartialEq, Eq)]
pub enum NetNote {
    /// A piece's text, told in the pieces' order; it is typed only once all are told.
    Text {
        /// The run.
        id: u32,
        /// The piece's place from 0, below `MAX_PIECES`.
        index: u16,
        /// The service's text, an untrusted reply, at most `MAX_NET_TEXT_BYTES`.
        text: Zeroizing<String>,
    },
    /// Every piece was told, or the key works.
    Done {
        /// The run or test.
        id: u32,
    },
    /// The run or test stopped; texts told stay valid, and the host keeps the recording for Try again.
    Fault {
        /// The run or test.
        id: u32,
        /// Why.
        fault: NetFault,
    },
}

/// The kinds a net note may be.
pub(crate) const KINDS: &[Kind] = &[Kind::NetText, Kind::NetDone, Kind::NetFailed];

impl NetNote {
    /// Reads one note from the `speech-net` job.
    ///
    /// # Errors
    /// Any `IpcError`, `Closed` when the job ended between frames; the stream is out of step after any other.
    pub fn read<R: Read + ?Sized>(r: &mut R) -> Result<Self, IpcError> {
        let head = Head::read(r, KINDS)?;
        let (id, mut p) = (head.id, Payload::new(r, head.len));
        let note = match head.kind {
            Kind::NetText => Self::Text {
                id,
                index: piece_index(p.u16()?)?,
                text: p.secret(TEXT)?,
            },
            Kind::NetDone => Self::Done { id },
            Kind::NetFailed => Self::Fault {
                id,
                fault: read_fault(&mut p)?,
            },
            _ => return Err(IpcError::BadKind),
        };
        p.end()?;
        Ok(note)
    }

    /// Writes this note as one frame, then flushes.
    ///
    /// # Errors
    /// As `check`, before any byte is written; `Io` when the stream fails.
    pub fn write<W: Write + ?Sized>(&self, w: &mut W) -> Result<(), IpcError> {
        let len = self.size()?;
        write_head(w, self.kind(), self.id(), len)?;
        self.put(&mut Put(w))?;
        w.flush()?;
        Ok(())
    }

    /// Checks this note as `write` does, so the job can answer a reply past its cap with a fault instead.
    ///
    /// # Errors
    /// `TooLong` for a text past `MAX_NET_TEXT_BYTES`, and `BadField` for an index or renewal out of range.
    pub fn check(&self) -> Result<(), IpcError> {
        check_len(self.kind(), self.size()?)
    }

    /// The payload's length, once every field is checked.
    fn size(&self) -> Result<usize, IpcError> {
        let mut size = Size::default();
        self.put(&mut size)?;
        Ok(size.0)
    }

    /// Runs this note's payload through `s`.
    fn put(&self, s: &mut dyn Sink) -> Result<(), IpcError> {
        match self {
            Self::Text { index, text, .. } => {
                s.u16(piece_index(*index)?)?;
                s.field(TEXT, text.as_bytes())
            }
            Self::Done { .. } => Ok(()),
            Self::Fault { fault, .. } => put_fault(s, *fault),
        }
    }

    const fn kind(&self) -> Kind {
        match self {
            Self::Text { .. } => Kind::NetText,
            Self::Done { .. } => Kind::NetDone,
            Self::Fault { .. } => Kind::NetFailed,
        }
    }

    /// The run or test this note belongs to.
    #[must_use]
    pub const fn id(&self) -> u32 {
        match self {
            Self::Text { id, .. } | Self::Done { id } | Self::Fault { id, .. } => *id,
        }
    }
}

/// Sends a fault: its code, then for the allowance whether its renewal is told, and the seconds.
fn put_fault(s: &mut dyn Sink, fault: NetFault) -> Result<(), IpcError> {
    s.byte(fault.code().code())?;
    let NetFault::Allowance { renews_s } = fault else {
        return Ok(());
    };
    s.byte(u8::from(renews_s.is_some()))?;
    match renews_s {
        Some(n) => s.u32(within(n, &NET_RENEW_S)?),
        None => Ok(()),
    }
}

/// Reads a fault.
fn read_fault<R: Read + ?Sized>(p: &mut Payload<'_, R>) -> Result<NetFault, IpcError> {
    let code = FaultCode::from_code(p.byte()?).ok_or(IpcError::BadField)?;
    let renews_s = if code == FaultCode::Allowance && p.flag()? {
        Some(within(p.u32()?, &NET_RENEW_S)?)
    } else {
        None
    };
    Ok(NetFault::of(code, renews_s))
}
