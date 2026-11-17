//! The voice job's notes and losses as the speech port tells them: a run's stages, levels, speech, warm-up, pieces,
//! end and faults; the devices with the system's cores, memory and drivers; and a lost worker as `Timeout` or `Gone`.

use super::Hardware;
use crate::worker::{Lost, Unsent};
use kx_ipc::{DeviceKind as Kind, VoiceNote};
use kx_module_api::Redacted;
use kx_module_api::ports::{
    CardId, Device, DeviceKind, Memory, SpeechDone, SpeechError, SpeechNote,
};

/// `note` as a run tells it; `None` for an answer, a sound piece or speech, which no run hears.
pub(super) fn note(note: VoiceNote) -> Option<SpeechNote> {
    Some(match note {
        VoiceNote::Level { level, .. } => SpeechNote::Level(level),
        VoiceNote::Heard { speech: true, .. } => SpeechNote::Heard,
        VoiceNote::Heard { speech: false, .. } => SpeechNote::Quiet,
        VoiceNote::Stage { stage, .. } => SpeechNote::Stage(stage),
        VoiceNote::Warm { percent, .. } => SpeechNote::Warming { percent },
        VoiceNote::Piece { index, text, .. } => SpeechNote::Piece {
            index,
            text: Redacted::new(text),
        },
        VoiceNote::Done {
            pieces,
            sound_ms,
            wait_ms,
            load_ms,
            memory_mb,
            card,
            ..
        } => SpeechNote::Done(SpeechDone {
            pieces,
            sound_ms,
            wait_ms,
            load_ms,
            memory_mb,
            card,
        }),
        VoiceNote::Fault { fault, .. } => SpeechNote::Fault(fault.into()),
        VoiceNote::Devices { .. }
        | VoiceNote::Facts { .. }
        | VoiceNote::Checked { .. }
        | VoiceNote::Sound { .. }
        | VoiceNote::Speech { .. } => return None,
    })
}

/// The error a lost worker gives: `Timeout` when it was late, else `Gone`, or `Down` while it rests.
pub(super) fn lost(lost: Lost) -> SpeechError {
    match lost {
        Lost::Late => SpeechError::Timeout,
        Lost::Gone => SpeechError::Gone,
        Lost::Down { retry_in } => SpeechError::Down { retry_in },
    }
}

/// The error an ask the stream did not send gives; one its frames refuse is the job's own fault.
pub(super) fn unsent(unsent: Unsent) -> SpeechError {
    match unsent {
        Unsent::Refused(_) => SpeechError::Internal,
        Unsent::Lost(gone) => lost(gone),
    }
}

/// The engine's device as the port's: the processor with its real cores and the system's memory reading, a card with
/// its driver from the system, and built-in graphics as a card with its mark. A card whose id the port refuses is left
/// out.
pub(super) fn device(found: kx_ipc::Device, hardware: &dyn Hardware) -> Option<Device> {
    let engine = Memory {
        total_mb: found.total_mb,
        free_mb: found.free_mb,
    };
    let built_in = found.kind == Kind::BuiltIn;
    let (kind, memory, cores, driver) = match found.kind {
        Kind::Cpu => {
            let cores = u16::try_from(hardware.cores()).unwrap_or(u16::MAX);
            let memory = hardware.memory().unwrap_or(engine);
            (DeviceKind::Cpu, memory, cores, String::new())
        }
        Kind::Card | Kind::BuiltIn => {
            let card = DeviceKind::Card(CardId::new(&found.id)?);
            let driver = hardware.driver(&found.name).unwrap_or(found.driver);
            (card, engine, 0, driver)
        }
    };
    Some(Device {
        kind,
        name: found.name,
        memory,
        cores,
        built_in,
        driver,
    })
}
