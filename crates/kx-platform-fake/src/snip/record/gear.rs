//! What the fake recorder checks before it records: the encoders, a device switched on, and the folder's answers.

use kx_module_api::ports::{Gear, RecordAsk, RecordError, RecordFacts, SaveError};

/// Fine for a ready microphone, else its error.
pub(super) fn mic_ready(gear: Gear) -> Result<(), RecordError> {
    match gear {
        Gear::Ready => Ok(()),
        Gear::Missing => Err(RecordError::NoMic),
        Gear::Refused => Err(RecordError::MicRefused),
    }
}

/// Fine for the computer's sound when ready, else `NoSound`.
pub(super) fn sound_ready(gear: Gear) -> Result<(), RecordError> {
    (gear == Gear::Ready)
        .then_some(())
        .ok_or(RecordError::NoSound)
}

/// Refuses a start without encoders, or with a device switched on that cannot record.
pub(super) fn able(facts: RecordFacts, ask: &RecordAsk) -> Result<(), RecordError> {
    if !facts.video {
        return Err(RecordError::NoVideo);
    }
    if ask.mic.is_some() {
        mic_ready(facts.mic)?;
    }
    if ask.sound {
        sound_ready(facts.sound)?;
    }
    Ok(())
}

/// `NoFolder` for a hidden run's folder error, else `other`.
pub(super) fn folder_error(error: SaveError, other: RecordError) -> RecordError {
    if error == SaveError::NoFolder {
        RecordError::NoFolder
    } else {
        other
    }
}
