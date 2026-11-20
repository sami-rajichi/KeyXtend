//! What the voice job sends on its own: levels, speech heard, stages, warm-up progress, each piece's text, the end of a
//! run, a fault, the devices, a file's facts, a card's check result, pieces of its recording and a listen-only run's
//! speech.

mod read;

use super::body::{put_devices, put_sound, put_tag};
use super::wire::{HELD, LANGS, PERCENT, SPOKEN, TEXT};
use super::{Device, SoundPiece, Stage, VoiceFault};
use crate::code::{Code, Kind};
use crate::error::IpcError;
use crate::frame::{Head, check_len, write_head};
use crate::limits::{MAX_PIECES, MAX_SOUND_MS};
use crate::rules::{ok_if, within};
use crate::wire::{Payload, Put, Sink, Size};
use read::{read_answer, read_run};
use std::io::{Read, Write};
use zeroize::Zeroizing;

/// A message from the voice job to the host, sent on its own; each repeats the id of the ask it belongs to.
#[derive(Clone, PartialEq, Eq)]
pub enum VoiceNote {
    /// The sound level, at `level_hz` while listening.
    Level {
        /// The run.
        id: u32,
        /// 0 is silence, `u8::MAX` full scale.
        level: u8,
    },
    /// The detector heard speech start, or stop.
    Heard {
        /// The run.
        id: u32,
        /// True when speech started.
        speech: bool,
    },
    /// The run reached a stage.
    Stage {
        /// The run.
        id: u32,
        /// Which one.
        stage: Stage,
    },
    /// A card's warm-up progress.
    Warm {
        /// The run or check.
        id: u32,
        /// From 0 to `MAX_PERCENT`.
        percent: u8,
    },
    /// A piece's text as soon as it is ready; pieces count from 0.
    Piece {
        /// The run.
        id: u32,
        /// Below `MAX_PIECES`.
        index: u16,
        /// The text, at most `MAX_TEXT_BYTES`.
        text: Zeroizing<String>,
    },
    /// The run ended, every piece told.
    Done {
        /// The run.
        id: u32,
        /// The pieces told, at most `MAX_PIECES`.
        pieces: u16,
        /// The sound heard in ms, at most `MAX_SOUND_MS`.
        sound_ms: u32,
        /// From Stop to the last text, in ms.
        wait_ms: u32,
        /// The model's load, in ms; 0 when it was loaded already.
        load_ms: u32,
        /// The job's peak memory, in MB.
        memory_mb: u32,
        /// True when a card served to the end.
        card: bool,
    },
    /// What was asked could not be done, for a fixed reason.
    Fault {
        /// The ask.
        id: u32,
        /// Why.
        fault: VoiceFault,
    },
    /// The processor and the cards.
    Devices {
        /// The ask.
        id: u32,
        /// From 1 to `MAX_DEVICES`.
        list: Vec<Device>,
    },
    /// A model file's runner and the language codes it names.
    Facts {
        /// The ask.
        id: u32,
        /// The runner that reads it.
        runner: String,
        /// Its language codes, at most `MAX_LANGS`.
        langs: Vec<String>,
    },
    /// A card's check: each device's time and text, for the module to score.
    Checked {
        /// The ask.
        id: u32,
        /// The card's time in ms.
        card_ms: u32,
        /// The processor's time in ms.
        cpu_ms: u32,
        /// The card's text.
        card_text: Zeroizing<String>,
        /// The processor's text.
        cpu_text: Zeroizing<String>,
    },
    /// A piece of the held recording, after `Keep`.
    Sound {
        /// The run.
        id: u32,
        /// The piece.
        piece: SoundPiece<'static>,
    },
    /// A listen-only run's speech at Stop: cut at pauses, trimmed and packed within `piece_s`, its index below
    /// `MAX_SPEECH_PIECES`; the held recording stays whole.
    Speech {
        /// The run.
        id: u32,
        /// The piece.
        piece: SoundPiece<'static>,
    },
}

/// The kinds a voice note may be.
pub(crate) const KINDS: &[Kind] = &[
    Kind::Level,
    Kind::Heard,
    Kind::Stage,
    Kind::Warm,
    Kind::Piece,
    Kind::Done,
    Kind::VoiceFailed,
    Kind::DeviceList,
    Kind::FileFacts,
    Kind::CardChecked,
    Kind::SoundOut,
    Kind::SpeechOut,
];

impl VoiceNote {
    /// Reads one note from the voice job.
    ///
    /// # Errors
    /// Any `IpcError`, `Closed` when the job ended between frames; the stream is out of step after any other.
    pub fn read<R: Read + ?Sized>(r: &mut R) -> Result<Self, IpcError> {
        let head = Head::read(r, KINDS)?;
        let mut p = Payload::new(r, head.len);
        let note = match head.kind {
            Kind::DeviceList | Kind::FileFacts | Kind::CardChecked | Kind::SoundOut => {
                read_answer(&mut p, head)?
            }
            _ => read_run(&mut p, head)?,
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

    /// Checks this note as `write` does, so the job can cut a text or answer a fault instead.
    ///
    /// # Errors
    /// `TooLong` past a cap, `BadField` for a value out of range, and `Inconsistent` for an empty list or name.
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
            Self::Level { level, .. } => s.byte(*level),
            Self::Heard { speech, .. } => s.byte(u8::from(*speech)),
            Self::Stage { stage, .. } => s.byte(stage.code()),
            Self::Warm { percent, .. } => s.byte(within(*percent, &PERCENT)?),
            Self::Piece { index, text, .. } => {
                ok_if(usize::from(*index) < MAX_PIECES)?;
                s.u16(*index)?;
                s.field(TEXT, text.as_bytes())
            }
            Self::Done { .. } => self.put_done(s),
            Self::Fault { fault, .. } => s.byte(fault.code()),
            Self::Devices { list, .. } => put_devices(s, list),
            Self::Facts { runner, langs, .. } => {
                put_tag(s, runner)?;
                s.count(LANGS, langs.len())?;
                langs.iter().try_for_each(|lang| put_tag(s, lang))
            }
            Self::Checked {
                card_ms,
                cpu_ms,
                card_text,
                cpu_text,
                ..
            } => {
                s.u32(*card_ms)?;
                s.u32(*cpu_ms)?;
                s.field(TEXT, card_text.as_bytes())?;
                s.field(TEXT, cpu_text.as_bytes())
            }
            Self::Sound { piece, .. } => put_sound(s, piece, HELD),
            Self::Speech { piece, .. } => put_sound(s, piece, SPOKEN),
        }
    }

    /// Runs a done note's fields through `s`.
    fn put_done(&self, s: &mut dyn Sink) -> Result<(), IpcError> {
        if let Self::Done {
            pieces,
            sound_ms,
            wait_ms,
            load_ms,
            memory_mb,
            card,
            ..
        } = self
        {
            ok_if(usize::from(*pieces) <= MAX_PIECES && *sound_ms <= MAX_SOUND_MS)?;
            s.u16(*pieces)?;
            [*sound_ms, *wait_ms, *load_ms, *memory_mb]
                .into_iter()
                .try_for_each(|n| s.u32(n))?;
            s.byte(u8::from(*card))?;
        }
        Ok(())
    }

    const fn kind(&self) -> Kind {
        match self {
            Self::Level { .. } => Kind::Level,
            Self::Heard { .. } => Kind::Heard,
            Self::Stage { .. } => Kind::Stage,
            Self::Warm { .. } => Kind::Warm,
            Self::Piece { .. } => Kind::Piece,
            Self::Done { .. } => Kind::Done,
            Self::Fault { .. } => Kind::VoiceFailed,
            Self::Devices { .. } => Kind::DeviceList,
            Self::Facts { .. } => Kind::FileFacts,
            Self::Checked { .. } => Kind::CardChecked,
            Self::Sound { .. } => Kind::SoundOut,
            Self::Speech { .. } => Kind::SpeechOut,
        }
    }

    /// The ask this note belongs to.
    #[must_use]
    pub const fn id(&self) -> u32 {
        match self {
            Self::Level { id, .. }
            | Self::Heard { id, .. }
            | Self::Stage { id, .. }
            | Self::Warm { id, .. }
            | Self::Piece { id, .. }
            | Self::Done { id, .. }
            | Self::Fault { id, .. }
            | Self::Devices { id, .. }
            | Self::Facts { id, .. }
            | Self::Checked { id, .. }
            | Self::Sound { id, .. }
            | Self::Speech { id, .. } => *id,
        }
    }
}
