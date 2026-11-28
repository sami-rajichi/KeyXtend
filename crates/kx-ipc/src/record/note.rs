//! What the record job sends on its own: the time while a run records, each switch made, a sound lost, the end of a
//! video with its length, or a fault.

use super::wire::{END_LEN, LOST_LEN, SWITCHED_LEN, TIME_LEN};
use super::{RecordEnd, RecordFault, Switch};
use crate::code::{Code, Kind};
use crate::error::IpcError;
use crate::frame::{Head, write_head};
use crate::limits::FAILED_LEN;
use crate::limits::snip::MAX_VIDEO_MS;
use crate::rules::ok_if;
use crate::wire::{Payload, Put, Sink};
use std::io::{Read, Write};

/// A message from the record job to the host, sent on its own; each repeats the id of its run or copy. None holds a
/// path or a pixel, so `Debug` shows it all.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecordNote {
    /// How long the run has recorded, about once a second; the first, at 0, says it started.
    Time {
        /// The run.
        id: u32,
        /// At most `MAX_VIDEO_MS`.
        ms: u32,
    },
    /// The run's video, or a copy, is whole and kept.
    End {
        /// The run or copy.
        id: u32,
        /// The video's length, at most `MAX_VIDEO_MS`.
        length_ms: u32,
        /// Why it ended.
        end: RecordEnd,
    },
    /// A fault, for a fixed reason; `RecordFault` says whether the run goes on.
    Fault {
        /// The run or copy.
        id: u32,
        /// Why.
        fault: RecordFault,
    },
    /// A `Mic` or `Sound` ask took effect: its answer, as `RecordAsk::ack` builds it.
    Switched {
        /// The run.
        id: u32,
        /// The sound switched.
        switch: Switch,
        /// Its new state, true for on.
        on: bool,
    },
    /// A sound's device stopped mid-run with no switch asked; the run records silence for it, as when switched off, and
    /// goes on.
    Lost {
        /// The run.
        id: u32,
        /// The sound lost.
        switch: Switch,
    },
}

/// The kinds a record note may be.
pub(crate) const KINDS: &[Kind] = &[
    Kind::RecordTime,
    Kind::Recorded,
    Kind::RecordFailed,
    Kind::RecordSwitched,
    Kind::RecordLost,
];

impl RecordNote {
    /// Reads one note from the record job.
    ///
    /// # Errors
    /// Any `IpcError`, `Closed` when the job ended between frames; the stream is out of step after any other.
    pub fn read<R: Read + ?Sized>(r: &mut R) -> Result<Self, IpcError> {
        let head = Head::read(r, KINDS)?;
        let (id, mut p) = (head.id, Payload::new(r, head.len));
        let note = match head.kind {
            Kind::RecordTime => Self::Time {
                id,
                ms: video_ms(p.u32()?)?,
            },
            Kind::Recorded => {
                let length_ms = video_ms(p.u32()?)?;
                let end = RecordEnd::from_code(p.byte()?).ok_or(IpcError::BadField)?;
                Self::End { id, length_ms, end }
            }
            Kind::RecordFailed => {
                let fault = RecordFault::from_code(p.byte()?).ok_or(IpcError::BadField)?;
                Self::Fault { id, fault }
            }
            Kind::RecordSwitched => {
                let switch = Switch::from_code(p.byte()?).ok_or(IpcError::BadField)?;
                let on = p.flag()?;
                Self::Switched { id, switch, on }
            }
            Kind::RecordLost => {
                let switch = Switch::from_code(p.byte()?).ok_or(IpcError::BadField)?;
                Self::Lost { id, switch }
            }
            _ => return Err(IpcError::BadKind),
        };
        p.end()?;
        Ok(note)
    }

    /// Writes this note as one frame, then flushes.
    ///
    /// # Errors
    /// `BadField`, before any byte is written, for a time or length past `MAX_VIDEO_MS`; `Io` when the stream fails.
    pub fn write<W: Write + ?Sized>(&self, w: &mut W) -> Result<(), IpcError> {
        let mut put = Put(w);
        match self {
            Self::Time { id, ms } => {
                video_ms(*ms)?;
                write_head(put.0, Kind::RecordTime, *id, TIME_LEN)?;
                put.u32(*ms)?;
            }
            Self::End { id, length_ms, end } => {
                video_ms(*length_ms)?;
                write_head(put.0, Kind::Recorded, *id, END_LEN)?;
                put.u32(*length_ms)?;
                put.byte(end.code())?;
            }
            Self::Fault { id, fault } => {
                write_head(put.0, Kind::RecordFailed, *id, FAILED_LEN)?;
                put.byte(fault.code())?;
            }
            Self::Switched { id, switch, on } => {
                write_head(put.0, Kind::RecordSwitched, *id, SWITCHED_LEN)?;
                put.byte(switch.code())?;
                put.byte(u8::from(*on))?;
            }
            Self::Lost { id, switch } => {
                write_head(put.0, Kind::RecordLost, *id, LOST_LEN)?;
                put.byte(switch.code())?;
            }
        }
        put.0.flush()?;
        Ok(())
    }

    /// The run or copy this note belongs to.
    #[must_use]
    pub const fn id(&self) -> u32 {
        match self {
            Self::Time { id, .. }
            | Self::End { id, .. }
            | Self::Fault { id, .. }
            | Self::Switched { id, .. }
            | Self::Lost { id, .. } => *id,
        }
    }
}

/// `ms` when it is at most the longest video.
fn video_ms(ms: u32) -> Result<u32, IpcError> {
    ok_if(ms <= MAX_VIDEO_MS)?;
    Ok(ms)
}
