//! What the host sends the voice job: start, stop, cancel, repeat, hand back or close a dictation, list the devices,
//! read a file's facts, check a card, hand in a piece of a recording, or goodbye.

use super::body::{
    put_model, put_path, put_setup, put_sound, read_model, read_path, read_setup, read_sound,
};
use super::wire::HELD;
use super::{Model, Setup, SoundPiece};
use crate::code::Kind;
use crate::error::IpcError;
use crate::frame::{Head, write_head};
use crate::wire::{Payload, Put, Sink, Size};
use std::borrow::Cow;
use std::io::{Read, Write};

/// A message from the host to the voice job; it borrows its fields when written and owns them when read. Every ask
/// but `Bye` carries the id of the run it acts on, which the job's notes repeat.
#[derive(Clone, PartialEq, Eq)]
pub enum VoiceAsk<'a> {
    /// Open the microphone and listen, transcribing pieces while it goes on.
    Start {
        /// The run's id.
        id: u32,
        /// The model, or none to listen only, and the settings.
        setup: Setup<'a>,
    },
    /// Close the microphone and transcribe the rest.
    Stop {
        /// The run's id.
        id: u32,
    },
    /// Close the microphone and drop the text; the recording stays until `Close`.
    Cancel {
        /// The run's id.
        id: u32,
    },
    /// Transcribe the held recording again, kept from a run or handed in by `Sound`.
    Again {
        /// The new run's id.
        id: u32,
        /// The model and the settings; one job loads one model.
        setup: Setup<'a>,
    },
    /// Hand the held recording back in `Sound` notes, to keep it or move it to another job.
    Keep {
        /// The run's id.
        id: u32,
    },
    /// End the run and wipe its recording.
    Close {
        /// The run's id.
        id: u32,
    },
    /// List the processor and the cards.
    Devices {
        /// The ask's id.
        id: u32,
    },
    /// Read a model file's runner and languages.
    Facts {
        /// The ask's id.
        id: u32,
        /// The model file.
        path: Cow<'a, str>,
    },
    /// Run a card's check: the clip on the card and on the processor.
    Check {
        /// The ask's id.
        id: u32,
        /// The model, on the card to check.
        model: Model<'a>,
        /// The shipped clip in a language the model knows.
        clip: Cow<'a, str>,
    },
    /// A piece of a recording another job made, for `Again`.
    Sound {
        /// The run's id.
        id: u32,
        /// The piece.
        piece: SoundPiece<'a>,
    },
    /// End: the voice job wipes its sound and exits after this frame.
    Bye,
}

/// The kinds a voice ask may be.
pub(crate) const KINDS: &[Kind] = &[
    Kind::VoiceStart,
    Kind::VoiceStop,
    Kind::VoiceCancel,
    Kind::VoiceAgain,
    Kind::VoiceKeep,
    Kind::VoiceClose,
    Kind::VoiceDevices,
    Kind::VoiceFacts,
    Kind::VoiceCheck,
    Kind::SoundIn,
    Kind::VoiceBye,
];

impl VoiceAsk<'_> {
    /// Reads one ask from the host.
    ///
    /// # Errors
    /// Any `IpcError`, `Closed` when the host ended between frames; the stream is out of step after any other.
    pub fn read<R: Read + ?Sized>(r: &mut R) -> Result<VoiceAsk<'static>, IpcError> {
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
            Self::Start { setup, .. } | Self::Again { setup, .. } => put_setup(s, setup),
            Self::Facts { path, .. } => put_path(s, path),
            Self::Check { model, clip, .. } => {
                put_model(s, model)?;
                put_path(s, clip)
            }
            Self::Sound { piece, .. } => put_sound(s, piece, HELD),
            Self::Stop { .. }
            | Self::Cancel { .. }
            | Self::Keep { .. }
            | Self::Close { .. }
            | Self::Devices { .. }
            | Self::Bye => Ok(()),
        }
    }

    const fn kind(&self) -> Kind {
        match self {
            Self::Start { .. } => Kind::VoiceStart,
            Self::Stop { .. } => Kind::VoiceStop,
            Self::Cancel { .. } => Kind::VoiceCancel,
            Self::Again { .. } => Kind::VoiceAgain,
            Self::Keep { .. } => Kind::VoiceKeep,
            Self::Close { .. } => Kind::VoiceClose,
            Self::Devices { .. } => Kind::VoiceDevices,
            Self::Facts { .. } => Kind::VoiceFacts,
            Self::Check { .. } => Kind::VoiceCheck,
            Self::Sound { .. } => Kind::SoundIn,
            Self::Bye => Kind::VoiceBye,
        }
    }

    /// The run or ask this acts on; goodbye's is 0.
    #[must_use]
    pub const fn id(&self) -> u32 {
        match self {
            Self::Start { id, .. }
            | Self::Stop { id }
            | Self::Cancel { id }
            | Self::Again { id, .. }
            | Self::Keep { id }
            | Self::Close { id }
            | Self::Devices { id }
            | Self::Facts { id, .. }
            | Self::Check { id, .. }
            | Self::Sound { id, .. } => *id,
            Self::Bye => 0,
        }
    }
}

/// Reads the payload of an ask of `head`'s kind.
fn read_body<R: Read + ?Sized>(
    p: &mut Payload<'_, R>,
    head: Head,
) -> Result<VoiceAsk<'static>, IpcError> {
    let id = head.id;
    Ok(match head.kind {
        Kind::VoiceStart => VoiceAsk::Start {
            id,
            setup: read_setup(p)?,
        },
        Kind::VoiceAgain => VoiceAsk::Again {
            id,
            setup: read_setup(p)?,
        },
        Kind::VoiceStop => VoiceAsk::Stop { id },
        Kind::VoiceCancel => VoiceAsk::Cancel { id },
        Kind::VoiceKeep => VoiceAsk::Keep { id },
        Kind::VoiceClose => VoiceAsk::Close { id },
        Kind::VoiceDevices => VoiceAsk::Devices { id },
        Kind::VoiceFacts => VoiceAsk::Facts {
            id,
            path: read_path(p)?,
        },
        Kind::VoiceCheck => {
            let model = read_model(p)?;
            let clip = read_path(p)?;
            VoiceAsk::Check { id, model, clip }
        }
        Kind::SoundIn => VoiceAsk::Sound {
            id,
            piece: read_sound(p, HELD)?,
        },
        Kind::VoiceBye if id == 0 => VoiceAsk::Bye,
        Kind::VoiceBye => return Err(IpcError::Inconsistent),
        _ => return Err(IpcError::BadKind),
    })
}
