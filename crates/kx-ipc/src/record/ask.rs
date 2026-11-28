//! What the host sends the record job: start a recording, switch its microphone or sound, stop or cancel it, ask for a
//! re-timed copy, or goodbye.

use super::body::{put_start, put_video, read_start, read_video, speed};
use super::note::RecordNote;
use super::{Area, Switch};
use crate::code::Kind;
use crate::error::IpcError;
use crate::frame::{Head, write_head};
use crate::wire::{Payload, Put, Sink, Size};
use std::borrow::Cow;
use std::fmt;
use std::io::{Read, Write};

/// A message from the host to the record job; it borrows its paths when written and owns them when read. Every ask but
/// `Bye` carries its run's id, which the job's notes repeat.
#[derive(Clone, PartialEq, Eq)]
pub enum RecordAsk<'a> {
    /// Record `area` into `file` until Stop, Cancel, `max_min` or the floor; the first `Time`, at 0, says it started.
    Start {
        /// The run's id.
        id: u32,
        /// The part filmed.
        area: Area,
        /// Frames a second, one of `SNIP_FPS`.
        fps: u8,
        /// The microphone records from the start.
        mic: bool,
        /// The computer's sound records from the start.
        sound: bool,
        /// The longest video in minutes, within `SNIP_MAX_MIN`; the job then stops by itself and keeps it.
        max_min: u16,
        /// The MiB its drive keeps free, within `SNIP_MIN_FREE_MB`; the job stops by itself before passing it.
        min_free_mb: u32,
        /// The new video, directly in the Snips folder's `unsaved` part: a plain path whose name ends in `MP4_EXT`.
        file: Cow<'a, str>,
    },
    /// Switch the microphone on or off at once; while off, the run records silence in step with the picture (edge 12).
    /// The job answers with `ack`'s note, or with the microphone's fault, and the run goes on.
    Mic {
        /// The run's id.
        id: u32,
        /// True to switch it on.
        on: bool,
    },
    /// Switch the computer's sound on or off at once, as `Mic` does.
    Sound {
        /// The run's id.
        id: u32,
        /// True to switch it on.
        on: bool,
    },
    /// Stop; `End` follows with the video's length, and the video stays.
    Stop {
        /// The run's id.
        id: u32,
    },
    /// Stop at once and delete the video, telling nothing more (edge 10), or `Failed` when it could not be deleted.
    Cancel {
        /// The run's id.
        id: u32,
    },
    /// Write a re-timed copy of `from` into `to`, its frames dropped; `End` follows with the copy's length (edge 20).
    Speed {
        /// The copy's id.
        id: u32,
        /// The video copied, a plain path whose name ends in `MP4_EXT`.
        from: Cow<'a, str>,
        /// How many times faster the copy plays, one of `SNIP_SPEEDS`.
        speed: u8,
        /// The new copy, directly in the `unsaved` part, as `from`.
        to: Cow<'a, str>,
    },
    /// End: the job stops a run still going, keeping its video, and exits after this frame.
    Bye,
}

/// The kinds a record ask may be.
pub(crate) const KINDS: &[Kind] = &[
    Kind::RecordStart,
    Kind::RecordMic,
    Kind::RecordSound,
    Kind::RecordStop,
    Kind::RecordCancel,
    Kind::RecordSpeed,
    Kind::RecordBye,
];

impl RecordAsk<'_> {
    /// Reads one ask from the host.
    ///
    /// # Errors
    /// Any `IpcError`, `Closed` when the host ended between frames; the stream is out of step after any other.
    pub fn read<R: Read + ?Sized>(r: &mut R) -> Result<RecordAsk<'static>, IpcError> {
        let head = Head::read(r, KINDS)?;
        let mut p = Payload::new(r, head.len);
        let ask = read_body(&mut p, head)?;
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
            Self::Start { .. } => put_start(s, self),
            Self::Mic { on, .. } | Self::Sound { on, .. } => s.byte(u8::from(*on)),
            Self::Speed {
                from, speed: x, to, ..
            } => {
                put_video(s, from)?;
                s.byte(speed(*x)?)?;
                put_video(s, to)
            }
            Self::Stop { .. } | Self::Cancel { .. } | Self::Bye => Ok(()),
        }
    }

    const fn kind(&self) -> Kind {
        match self {
            Self::Start { .. } => Kind::RecordStart,
            Self::Mic { .. } => Kind::RecordMic,
            Self::Sound { .. } => Kind::RecordSound,
            Self::Stop { .. } => Kind::RecordStop,
            Self::Cancel { .. } => Kind::RecordCancel,
            Self::Speed { .. } => Kind::RecordSpeed,
            Self::Bye => Kind::RecordBye,
        }
    }

    /// The run or copy this acts on; goodbye's is 0.
    #[must_use]
    pub const fn id(&self) -> u32 {
        match self {
            Self::Start { id, .. }
            | Self::Mic { id, .. }
            | Self::Sound { id, .. }
            | Self::Stop { id }
            | Self::Cancel { id }
            | Self::Speed { id, .. } => *id,
            Self::Bye => 0,
        }
    }

    /// The note that answers this switch once it took effect; `None` for any other ask.
    #[must_use]
    pub const fn ack(&self) -> Option<RecordNote> {
        let (id, switch, on) = match *self {
            Self::Mic { id, on } => (id, Switch::Mic, on),
            Self::Sound { id, on } => (id, Switch::Sound, on),
            _ => return None,
        };
        Some(RecordNote::Switched { id, switch, on })
    }
}

/// Reads the payload of an ask of `head`'s kind.
fn read_body<R: Read + ?Sized>(
    p: &mut Payload<'_, R>,
    head: Head,
) -> Result<RecordAsk<'static>, IpcError> {
    let id = head.id;
    Ok(match head.kind {
        Kind::RecordStart => read_start(p, id)?,
        Kind::RecordMic => RecordAsk::Mic { id, on: p.flag()? },
        Kind::RecordSound => RecordAsk::Sound { id, on: p.flag()? },
        Kind::RecordStop => RecordAsk::Stop { id },
        Kind::RecordCancel => RecordAsk::Cancel { id },
        Kind::RecordSpeed => {
            let from = read_video(p)?;
            let x = speed(p.byte()?)?;
            let to = read_video(p)?;
            RecordAsk::Speed {
                id,
                from,
                speed: x,
                to,
            }
        }
        Kind::RecordBye if id == 0 => RecordAsk::Bye,
        Kind::RecordBye => return Err(IpcError::Inconsistent),
        _ => return Err(IpcError::BadKind),
    })
}

impl fmt::Debug for RecordAsk<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if matches!(self, Self::Bye) {
            return f.write_str("Bye");
        }
        let mut s = f.debug_struct(name(self));
        s.field("id", &self.id());
        match self {
            Self::Start {
                area,
                fps,
                mic,
                sound,
                max_min,
                min_free_mb,
                ..
            } => s
                .field("area", area)
                .field("fps", fps)
                .field("mic", mic)
                .field("sound", sound)
                .field("max_min", max_min)
                .field("min_free_mb", min_free_mb)
                .finish_non_exhaustive(),
            Self::Mic { on, .. } | Self::Sound { on, .. } => s.field("on", on).finish(),
            Self::Speed { speed: x, .. } => s.field("speed", x).finish_non_exhaustive(),
            Self::Stop { .. } | Self::Cancel { .. } | Self::Bye => s.finish(),
        }
    }
}

/// The variant's name, as `Debug` starts it.
const fn name(ask: &RecordAsk<'_>) -> &'static str {
    match ask {
        RecordAsk::Start { .. } => "Start",
        RecordAsk::Mic { .. } => "Mic",
        RecordAsk::Sound { .. } => "Sound",
        RecordAsk::Stop { .. } => "Stop",
        RecordAsk::Cancel { .. } => "Cancel",
        RecordAsk::Speed { .. } => "Speed",
        RecordAsk::Bye => "Bye",
    }
}
