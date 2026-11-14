//! The voice job's one-byte codes: device kinds, stages and faults; code 0 is never used.

use crate::code::Code;

/// What kind of device runs a model.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum DeviceKind {
    /// The processor.
    Cpu = 1,
    /// A graphics card with its own memory.
    Card = 2,
    /// Graphics built into the processor, sharing its memory; a model targets it as a card.
    BuiltIn = 3,
}

impl Code for DeviceKind {
    const ALL: &'static [Self] = &[Self::Cpu, Self::Card, Self::BuiltIn];

    fn code(self) -> u8 {
        self as u8
    }
}

/// A stage of a run the job tells on its own, so the key's light follows the microphone exactly.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Stage {
    /// The model is loading.
    Loading = 1,
    /// The microphone is open.
    Open = 2,
    /// The microphone closed at Stop or Cancel.
    Shut = 3,
    /// The microphone closed at `record_max_s`.
    Limit = 4,
    /// The microphone closed after `silence_stop_s` of silence.
    Silence = 5,
}

impl Code for Stage {
    const ALL: &'static [Self] = &[
        Self::Loading,
        Self::Open,
        Self::Shut,
        Self::Limit,
        Self::Silence,
    ];

    fn code(self) -> u8 {
        self as u8
    }
}

/// Why the voice job could not do what was asked; a fixed reason, never a library's words. The card faults leave the
/// run going on the processor; every other ends the ask.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum VoiceFault {
    /// No microphone is there.
    NoMic = 1,
    /// Another app holds the microphone.
    MicBusy = 2,
    /// The system's privacy switch refused the microphone.
    MicRefused = 3,
    /// The microphone was unplugged while listening.
    Unplugged = 4,
    /// A named file is missing: the model, the detector or the clip.
    NoFile = 5,
    /// A file the runner or the detector cannot read.
    BadFile = 6,
    /// The model does not know the language code.
    Language = 7,
    /// The runner is unknown, or not built in.
    Runner = 8,
    /// The named card is gone; the processor serves.
    NoCard = 9,
    /// The card lacks the free memory now; the processor serves.
    CardRoom = 10,
    /// The card failed; the processor serves the same recording.
    CardFailed = 11,
    /// No recording is held to transcribe again or hand back.
    NoSound = 12,
    /// An ask the job cannot take now, such as a second model.
    OutOfStep = 13,
    /// The job reached its memory cap.
    Memory = 14,
    /// The job failed inside.
    Internal = 15,
}

impl Code for VoiceFault {
    const ALL: &'static [Self] = &[
        Self::NoMic,
        Self::MicBusy,
        Self::MicRefused,
        Self::Unplugged,
        Self::NoFile,
        Self::BadFile,
        Self::Language,
        Self::Runner,
        Self::NoCard,
        Self::CardRoom,
        Self::CardFailed,
        Self::NoSound,
        Self::OutOfStep,
        Self::Memory,
        Self::Internal,
    ];

    fn code(self) -> u8 {
        self as u8
    }
}
