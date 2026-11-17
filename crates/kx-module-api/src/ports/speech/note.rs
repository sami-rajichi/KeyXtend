//! What a run and a check tell, as the `voice` job's frames do: the stages, the level, speech heard, the warm-up, each
//! piece's text, the end with its times and memory, and a fault, then a check's verdict with its numbers. Only a piece
//! and a check carry text, kept secret.

use super::SpeechError;
use crate::Secret;

/// The frames' stages of a run and the top of a warm-up's percent, so the key's light follows the microphone exactly.
pub use kx_ipc::{MAX_PERCENT, Stage as SpeechStage};

/// The loudest level a note or `VoiceLevel` carries, the frames' full scale; 0 is silence.
pub const LEVEL_FULL: u8 = u8::MAX;

/// The end of a run: every piece was told.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SpeechDone {
    /// How many pieces were told.
    pub pieces: u16,
    /// How long the sound heard was, in ms.
    pub sound_ms: u32,
    /// How long the text took after Stop, in ms.
    pub wait_ms: u32,
    /// How long the model took to load, in ms; 0 when it was loaded already.
    pub load_ms: u32,
    /// The most memory the job took, in MB.
    pub memory_mb: u32,
    /// A card served to the end.
    pub card: bool,
}

/// One note from a run or a check, told in order on any thread.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpeechNote {
    /// The run reached a stage: the model loads, or the microphone opened or closed and why.
    Stage(SpeechStage),
    /// The sound level, 0 to `LEVEL_FULL`, `SpeechAsk::level_hz` times a second while listening.
    Level(u8),
    /// The voice detector hears speech.
    Heard,
    /// The voice detector hears silence again.
    Quiet,
    /// The card warms up for this model, once before its first use, from 0 to `MAX_PERCENT`.
    Warming {
        /// How far it is.
        percent: u8,
    },
    /// A piece's text, the first piece at index 0; it is never shown before Stop.
    Piece {
        /// The piece's place in the recording.
        index: u16,
        /// Its text.
        text: Secret,
    },
    /// Every piece was told.
    Done(SpeechDone),
    /// The job ended after `JobLimits::idle_min` and wiped the recording, so `again` and `keep` are refused.
    Wiped,
    /// A fault; the run stops unless `SpeechError::ends` says the processor carries on. Pieces told stay valid.
    Fault(SpeechError),
}

/// A factor kept exactly in hundredths, such as 150 for 1.5×, so a view carries it and the ui prints it.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hundredths(pub u32);

impl Hundredths {
    /// Hundredths in one.
    pub const PER_ONE: u32 = 100;
}

/// How the voice module scores a card's check from its times and texts, with the numbers its mark and note show.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Verdict {
    /// The card may serve this model, as Passed · 9× faster.
    Passed {
        /// How many times faster than the processor it was.
        gain: Hundredths,
    },
    /// The card was not `card_gain` times faster than the processor.
    Slow {
        /// How many times faster it was.
        gain: Hundredths,
        /// `card_gain`, the factor it needed.
        need: Hundredths,
    },
    /// The card's text differed from the processor's by more than `card_diff` words.
    Differs {
        /// The words that differed.
        words: u16,
        /// `card_diff`, the words allowed.
        allowed: u8,
    },
    /// The card had too little free memory for the model and its margin.
    NoRoom,
    /// The card reported an error.
    Failed,
}

/// A card's check as the job ran it: each device's time and text on the clip, for the voice module to score.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckDone {
    /// How long the card took on the clip, in ms.
    pub card_ms: u32,
    /// How long the processor took on the clip, in ms.
    pub cpu_ms: u32,
    /// The card's text.
    pub card_text: Secret,
    /// The processor's text.
    pub cpu_text: Secret,
}
