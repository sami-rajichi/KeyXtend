//! The voice job as the script fakes it: notes made for each ask's id, silence, or a frame that cannot be read; and
//! the asks and notes the stream tests send and expect.

use super::{Call, Faked, Frame};
use crate::worker::{Heard, VoiceFrames};
use kx_ipc::{
    Device, DeviceKind, IpcError, LEVEL_HZ, PAD_MS, PIECE_S, RECORD_MAX_S, Setup, Stage, VoiceAsk,
    VoiceFault, VoiceNote,
};
use kx_module_api::Zeroizing;
use std::borrow::Cow;

/// The detector file a test dictation names.
const DETECTOR: &str = "detector.onnx";
/// The words a test piece holds.
const WORDS: &str = "hello";
/// The processor's id and name in a test device list.
const CPU: &str = "cpu";
/// The memory a test device has, in MB.
const MEMORY_MB: u32 = 1;

/// What the voice worker does with the next ask; with none queued it says nothing.
#[derive(Clone, Debug)]
pub enum VoiceAct {
    /// These notes for the ask's id, in order.
    Say(Vec<fn(u32) -> VoiceNote>),
    /// Nothing at all.
    Silent,
    /// A frame that could not be read, such as one of an unknown kind or past its size.
    Broken(IpcError),
}

impl Faked for VoiceFrames {
    type Act = VoiceAct;

    fn usual() -> VoiceAct {
        VoiceAct::Silent
    }

    fn note(ask: VoiceAsk<'static>) -> Option<(u32, Call)> {
        match ask {
            VoiceAsk::Bye => None,
            ask => Some((ask.id(), Call::Voice(ask))),
        }
    }

    fn answer(act: VoiceAct, id: u32) -> Heard<VoiceNote> {
        let first = Self::play(act, id).into_iter().next();
        first.map_or(Heard::Late, Heard::Frame)
    }

    fn play(act: VoiceAct, id: u32) -> Vec<Frame<Self>> {
        match act {
            VoiceAct::Say(notes) => notes.into_iter().map(|note| Ok(note(id))).collect(),
            VoiceAct::Silent => Vec::new(),
            VoiceAct::Broken(e) => vec![Err(e)],
        }
    }
}

/// A listen-only dictation's settings, each at the bottom of its range.
fn setup() -> Setup<'static> {
    Setup {
        model: None,
        pad_ms: *PAD_MS.start(),
        piece_s: *PIECE_S.start(),
        record_max_s: *RECORD_MAX_S.start(),
        level_hz: *LEVEL_HZ.start(),
        silence_stop_s: None,
        detector: Cow::Borrowed(DETECTOR),
    }
}

/// Starts run `id`.
pub fn start(id: u32) -> VoiceAsk<'static> {
    VoiceAsk::Start { id, setup: setup() }
}

/// Starts run `id` at a level rate past its range, which the frames refuse.
pub fn bad_start(id: u32) -> VoiceAsk<'static> {
    let mut setup = setup();
    setup.level_hz = LEVEL_HZ.end() + 1;
    VoiceAsk::Start { id, setup }
}

/// Lists the devices as ask `id`.
pub fn list(id: u32) -> VoiceAsk<'static> {
    VoiceAsk::Devices { id }
}

/// Stops run `id`.
pub fn stop(id: u32) -> VoiceAsk<'static> {
    VoiceAsk::Stop { id }
}

/// Cancels run `id`.
pub fn cancel(id: u32) -> VoiceAsk<'static> {
    VoiceAsk::Cancel { id }
}

/// Closes run `id`.
pub fn close(id: u32) -> VoiceAsk<'static> {
    VoiceAsk::Close { id }
}

/// The microphone opened for run `id`.
pub fn open(id: u32) -> VoiceNote {
    VoiceNote::Stage {
        id,
        stage: Stage::Open,
    }
}

/// The microphone closed for run `id`.
pub fn shut(id: u32) -> VoiceNote {
    VoiceNote::Stage {
        id,
        stage: Stage::Shut,
    }
}

/// A sound level for run `id`.
pub fn level(id: u32) -> VoiceNote {
    VoiceNote::Level { id, level: 1 }
}

/// Run `id`'s first piece of text.
pub fn piece(id: u32) -> VoiceNote {
    VoiceNote::Piece {
        id,
        index: 0,
        text: Zeroizing::new(WORDS.to_owned()),
    }
}

/// Run `id` ended after one piece.
pub fn done(id: u32) -> VoiceNote {
    VoiceNote::Done {
        id,
        pieces: 1,
        sound_ms: 0,
        wait_ms: 0,
        load_ms: 0,
        memory_mb: 0,
        card: false,
    }
}

/// The devices ask `id` found: the processor alone.
pub fn devices(id: u32) -> VoiceNote {
    let cpu = Device {
        kind: DeviceKind::Cpu,
        id: CPU.to_owned(),
        name: CPU.to_owned(),
        total_mb: MEMORY_MB,
        free_mb: MEMORY_MB,
        driver: String::new(),
    };
    VoiceNote::Devices {
        id,
        list: vec![cpu],
    }
}

/// The card failed for run `id`, which goes on on the processor.
pub fn card_failed(id: u32) -> VoiceNote {
    VoiceNote::Fault {
        id,
        fault: VoiceFault::CardFailed,
    }
}

/// Run or ask `id` failed inside, which ends it.
pub fn failed(id: u32) -> VoiceNote {
    VoiceNote::Fault {
        id,
        fault: VoiceFault::Internal,
    }
}
