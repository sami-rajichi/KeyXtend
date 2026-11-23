//! What a run may tell in the properties: its stages, levels, speech, pieces, end, wipe and faults.

use super::*;

/// The stages a run may tell.
const STAGES: [SpeechStage; 5] = [
    SpeechStage::Loading,
    SpeechStage::Open,
    SpeechStage::Shut,
    SpeechStage::Limit,
    SpeechStage::Silence,
];

/// The faults a run or a call may tell.
const FAULTS: [SpeechError; 19] = [
    SpeechError::Range("model"),
    SpeechError::BadPath,
    SpeechError::NoMic,
    SpeechError::MicBusy,
    SpeechError::MicRefused,
    SpeechError::Unplugged,
    SpeechError::NoFile,
    SpeechError::BadFile,
    SpeechError::Language,
    SpeechError::Runner,
    SpeechError::NoCard,
    SpeechError::CardRoom,
    SpeechError::CardFailed,
    SpeechError::NoSound,
    SpeechError::OutOfStep,
    SpeechError::Memory,
    SpeechError::Internal,
    SpeechError::Timeout,
    SpeechError::Gone,
];

/// What a run tells, by index into the tables above.
#[derive(Copy, Clone, Debug)]
pub(super) enum Said {
    Stage(usize),
    Level(u8),
    Heard,
    Quiet,
    /// A piece at its place, empty when true.
    Piece(u16, bool),
    Done,
    Wiped,
    Fault(usize),
}

impl Said {
    /// The note it stands for.
    pub(super) fn note(self) -> SpeechNote {
        match self {
            Self::Stage(i) => SpeechNote::Stage(STAGES[i]),
            Self::Level(level) => SpeechNote::Level(level),
            Self::Heard => SpeechNote::Heard,
            Self::Quiet => SpeechNote::Quiet,
            Self::Piece(index, empty) => {
                let text = secret(if empty { " " } else { "mot" });
                SpeechNote::Piece { index, text }
            }
            Self::Done => SpeechNote::Done(done(1)),
            Self::Wiped => SpeechNote::Wiped,
            Self::Fault(i) => SpeechNote::Fault(FAULTS[i]),
        }
    }
}

/// Any note a run may tell.
pub(super) fn said() -> impl Strategy<Value = Said> {
    prop_oneof![
        (0..STAGES.len()).prop_map(Said::Stage),
        any::<u8>().prop_map(Said::Level),
        Just(Said::Heard),
        Just(Said::Quiet),
        (0..3_u16, any::<bool>()).prop_map(|(i, empty)| Said::Piece(i, empty)),
        Just(Said::Done),
        Just(Said::Wiped),
        (0..FAULTS.len()).prop_map(Said::Fault),
    ]
}
