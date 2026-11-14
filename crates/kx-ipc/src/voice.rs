//! The voice job's frames (ADR-0042): asks that start, stop, cancel, repeat, hand back and close a dictation, list the
//! devices, read a file's facts and check a card, and the notes the job sends on its own. Text and sound travel in
//! buffers wiped when dropped, and `Debug` never shows them or a path.

pub(crate) mod ask;
pub(crate) mod body;
pub(crate) mod code;
pub(crate) mod note;
mod show;
pub(crate) mod wire;

pub use code::{DeviceKind, Stage, VoiceFault};

use crate::wiped::Wiped;
use std::borrow::Cow;

/// How a dictation runs: the model, or none to listen only, and the settings the job needs.
#[derive(Clone, PartialEq, Eq)]
pub struct Setup<'a> {
    /// The model to load, or `None` to record and hand the sound back at Stop.
    pub model: Option<Model<'a>>,
    /// Silence kept around speech in ms, within `PAD_MS`.
    pub pad_ms: u16,
    /// Seconds before a cut at the next pause, within `PIECE_S`.
    pub piece_s: u8,
    /// The longest dictation in seconds, within `RECORD_MAX_S`.
    pub record_max_s: u16,
    /// Sound levels per second, within `LEVEL_HZ`.
    pub level_hz: u8,
    /// Seconds of silence after speech that stop it, within `SILENCE_STOP_S`; `None` when that is off.
    pub silence_stop_s: Option<u8>,
    /// The voice detector's file.
    pub detector: Cow<'a, str>,
}

/// One model file and how to run it; the job holds no table, so the ask carries every field.
#[derive(Clone, PartialEq, Eq)]
pub struct Model<'a> {
    /// The model file.
    pub path: Cow<'a, str>,
    /// The model's own code for the language, such as `fr-FR`.
    pub lang: Cow<'a, str>,
    /// The runner that reads it, as the table names it.
    pub runner: Cow<'a, str>,
    /// Where it runs.
    pub target: Target<'a>,
    /// Processor threads within `THREADS`; 0 means the real cores.
    pub threads: u8,
}

/// Where a model runs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target<'a> {
    /// The processor.
    Cpu,
    /// A card, refused before loading unless `need_mb` is free on it then.
    Card {
        /// The card's id, as the device list gives it.
        id: Cow<'a, str>,
        /// The model's card memory plus the margin, in MB.
        need_mb: u32,
    },
}

/// A piece of a recording handed over: 16 kHz mono 16-bit sound, a secret.
#[derive(Clone, PartialEq, Eq)]
pub struct SoundPiece<'a> {
    /// Its place in the recording from 0, below `MAX_SOUND_PIECES`.
    pub index: u16,
    /// True for the recording's last piece.
    pub last: bool,
    /// Whole samples, at most `MAX_SOUND_BYTES`.
    pub bytes: Wiped<'a>,
}

/// One device the engine found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Device {
    /// The processor or a card.
    pub kind: DeviceKind,
    /// Its id, which a `Target::Card` names.
    pub id: String,
    /// Its shown name.
    pub name: String,
    /// Its memory in MB.
    pub total_mb: u32,
    /// Its free memory in MB now, at most `total_mb`.
    pub free_mb: u32,
    /// Its driver's version, empty for the processor.
    pub driver: String,
}
