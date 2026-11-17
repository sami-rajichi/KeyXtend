//! What the host sends the `speech-net` job: send a run's pieces, hand in their sound, test a key, cancel, or goodbye.

use super::body::{named, put_call, put_run, put_sound, read_call, read_run, read_sound};
use super::{NetCall, NetRun};
use crate::code::Kind;
use crate::error::IpcError;
use crate::frame::{Head, write_head};
use crate::wiped::Wiped;
use crate::wire::{Payload, Put, Sink, Size};
use std::io::{Read, Write};

/// A message from the host to the `speech-net` job; it borrows its fields when written and owns them when read. Every
/// ask but `Bye` carries the id of the run or test it acts on, which the job's notes repeat.
#[derive(Clone, PartialEq, Eq)]
pub enum NetAsk<'a> {
    /// Send a dictation's pieces to a service, several at a time within the row's limits; their sound follows.
    Send {
        /// The run's id.
        id: u32,
        /// Where the pieces go and how the key goes with them.
        call: NetCall<'a>,
        /// The model, the language, the limits, the tries and each piece's size.
        run: NetRun<'a>,
    },
    /// A part of a piece's sound, a secret: the pieces come in order, each in parts of at most `MAX_SOUND_BYTES`.
    Sound {
        /// The run's id.
        id: u32,
        /// The piece's place in the run from 0, below `MAX_PIECES`.
        piece: u16,
        /// Whole samples of 16 kHz mono 16-bit sound, never empty.
        bytes: Wiped<'a>,
    },
    /// Say whether a key works, by a call that sends no sound.
    Test {
        /// The test's id.
        id: u32,
        /// The row's test address, header and the key.
        call: NetCall<'a>,
    },
    /// Drop a run or a test: its requests in flight are dropped, its sound wiped, and nothing more is told of it.
    Cancel {
        /// The run's or test's id.
        id: u32,
    },
    /// End: the job wipes its keys and sound and exits after this frame.
    Bye,
}

/// The kinds a net ask may be.
pub(crate) const KINDS: &[Kind] = &[
    Kind::NetSend,
    Kind::NetSound,
    Kind::NetTest,
    Kind::NetCancel,
    Kind::NetBye,
];

impl NetAsk<'_> {
    /// Reads one ask from the host.
    ///
    /// # Errors
    /// Any `IpcError`, `Closed` when the host ended between frames; the stream is out of step after any other.
    pub fn read<R: Read + ?Sized>(r: &mut R) -> Result<NetAsk<'static>, IpcError> {
        let head = Head::read(r, KINDS)?;
        let (id, mut p) = (head.id, Payload::new(r, head.len));
        let ask = match head.kind {
            Kind::NetSend => {
                let call = read_call(&mut p)?;
                let run = read_run(&mut p)?;
                named(&call, &run)?;
                NetAsk::Send { id, call, run }
            }
            Kind::NetSound => {
                let (piece, bytes) = read_sound(&mut p)?;
                NetAsk::Sound { id, piece, bytes }
            }
            Kind::NetTest => NetAsk::Test {
                id,
                call: read_call(&mut p)?,
            },
            Kind::NetCancel => NetAsk::Cancel { id },
            Kind::NetBye if id == 0 => NetAsk::Bye,
            Kind::NetBye => return Err(IpcError::Inconsistent),
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
            Self::Send { call, run, .. } => {
                put_call(s, call)?;
                put_run(s, run)?;
                named(call, run)
            }
            Self::Sound { piece, bytes, .. } => put_sound(s, *piece, bytes),
            Self::Test { call, .. } => put_call(s, call),
            Self::Cancel { .. } | Self::Bye => Ok(()),
        }
    }

    const fn kind(&self) -> Kind {
        match self {
            Self::Send { .. } => Kind::NetSend,
            Self::Sound { .. } => Kind::NetSound,
            Self::Test { .. } => Kind::NetTest,
            Self::Cancel { .. } => Kind::NetCancel,
            Self::Bye => Kind::NetBye,
        }
    }

    /// The run or test this acts on; goodbye's is 0.
    #[must_use]
    pub const fn id(&self) -> u32 {
        match self {
            Self::Send { id, .. }
            | Self::Sound { id, .. }
            | Self::Test { id, .. }
            | Self::Cancel { id } => *id,
            Self::Bye => 0,
        }
    }
}
