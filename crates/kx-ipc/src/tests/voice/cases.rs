//! The voice tests' cases and helpers: one ask and one note of each kind, and the readers each side uses.

use super::super::*;
use crate::{
    Device, DeviceKind, MAX_PERCENT, Model, Setup, SoundPiece, Stage, Target, VoiceAsk, VoiceFault,
    VoiceNote, Wiped,
};
use zeroize::Zeroizing;

/// Sound bytes the tests send, an even length so they are whole samples.
pub(super) const SPOKEN: &[u8] = b"spoken words";
/// Words no `Debug` may show: a model's path, a detector's or clip's path, sound and texts.
pub(super) const SECRETS: &[&str] = &[
    "models",
    "KeyXtend",
    "spoken",
    "bonjour",
    "مرحبا",
    "clip said",
];

pub(super) fn voice_ask_bytes(ask: &VoiceAsk) -> Vec<u8> {
    let mut out = Vec::new();
    ask.write(&mut out).unwrap();
    out
}

pub(super) fn voice_note_bytes(note: &VoiceNote) -> Vec<u8> {
    let mut out = Vec::new();
    note.write(&mut out).unwrap();
    out
}

pub(super) fn read_voice_ask(r: &mut dyn Read) -> Result<(), IpcError> {
    VoiceAsk::read(r).map(drop)
}

pub(super) fn read_voice_note(r: &mut dyn Read) -> Result<(), IpcError> {
    VoiceNote::read(r).map(drop)
}

pub(super) fn text(s: &str) -> Zeroizing<String> {
    Zeroizing::new(s.to_string())
}

/// A card with a small memory need.
pub(super) fn card() -> Target<'static> {
    Target::Card {
        id: Cow::Borrowed("vulkan0"),
        need_mb: 1024,
    }
}

/// A French model on `target`, on the real cores.
pub(super) fn model(target: Target<'static>) -> Model<'static> {
    Model {
        path: Cow::Borrowed(r"D:\models\parakeet.gguf"),
        lang: Cow::Borrowed("fr-FR"),
        runner: Cow::Borrowed("transcribe"),
        target,
        threads: 0,
    }
}

/// A setup at the settings' starting values, naming `model` or none.
pub(super) fn setup(model: Option<Model<'static>>) -> Setup<'static> {
    Setup {
        model,
        pad_ms: 300,
        piece_s: 30,
        record_max_s: 600,
        level_hz: 30,
        silence_stop_s: None,
        detector: Cow::Borrowed(r"C:\KeyXtend\voice\detect.bin"),
    }
}

/// A sound piece lending `bytes`.
pub(super) fn sound(index: u16, last: bool, bytes: &[u8]) -> SoundPiece<'_> {
    SoundPiece {
        index,
        last,
        bytes: Wiped::Lent(bytes),
    }
}

/// The asks a dictation takes, a listen-only start among them.
fn run_asks() -> Vec<VoiceAsk<'static>> {
    let quiet = Setup {
        silence_stop_s: Some(3),
        ..setup(None)
    };
    vec![
        VoiceAsk::Start {
            id: 1,
            setup: setup(Some(model(Target::Cpu))),
        },
        VoiceAsk::Start {
            id: 2,
            setup: quiet,
        },
        VoiceAsk::Stop { id: 3 },
        VoiceAsk::Cancel { id: 4 },
        VoiceAsk::Again {
            id: 5,
            setup: setup(Some(model(card()))),
        },
        VoiceAsk::Keep { id: 6 },
        VoiceAsk::Close { id: 7 },
    ]
}

/// One ask of each kind, a listen-only start and an empty last sound piece.
pub(super) fn every_voice_ask() -> Vec<VoiceAsk<'static>> {
    let mut asks = run_asks();
    asks.extend([
        VoiceAsk::Devices { id: 8 },
        VoiceAsk::Facts {
            id: 9,
            path: Cow::Borrowed(r"D:\models\other.bin"),
        },
        VoiceAsk::Check {
            id: 10,
            model: model(card()),
            clip: Cow::Borrowed(r"C:\KeyXtend\voice\ar.wav"),
        },
        VoiceAsk::Sound {
            id: 11,
            piece: sound(0, false, SPOKEN),
        },
        VoiceAsk::Sound {
            id: u32::MAX,
            piece: sound(1, true, b""),
        },
        VoiceAsk::Bye,
    ]);
    asks
}

/// The processor, one card and built-in graphics.
pub(super) fn devices() -> Vec<Device> {
    let cpu = Device {
        kind: DeviceKind::Cpu,
        id: "cpu".into(),
        name: "8-core processor".into(),
        total_mb: 16_000,
        free_mb: 9_000,
        driver: String::new(),
    };
    let card = Device {
        kind: DeviceKind::Card,
        id: "vulkan0".into(),
        name: "Laptop GPU".into(),
        total_mb: 8_192,
        free_mb: 8_192,
        driver: "32.0.15.6094".into(),
    };
    let built_in = Device {
        kind: DeviceKind::BuiltIn,
        id: "vulkan1".into(),
        name: "Integrated graphics".into(),
        total_mb: 16_000,
        free_mb: 7_000,
        driver: "31.0.101.5186".into(),
    };
    vec![cpu, card, built_in]
}

/// The notes a dictation tells, in order.
fn run_notes() -> Vec<VoiceNote> {
    let mut notes = vec![
        VoiceNote::Level { id: 1, level: 0 },
        VoiceNote::Level {
            id: 1,
            level: u8::MAX,
        },
        VoiceNote::Heard {
            id: 1,
            speech: true,
        },
        VoiceNote::Heard {
            id: 1,
            speech: false,
        },
    ];
    notes.extend(
        Stage::ALL
            .iter()
            .map(|&stage| VoiceNote::Stage { id: 1, stage }),
    );
    let piece = |index, s| VoiceNote::Piece {
        id: 3,
        index,
        text: text(s),
    };
    notes.extend([
        VoiceNote::Warm {
            id: 2,
            percent: MAX_PERCENT,
        },
        piece(0, "bonjour à tous"),
        piece(1, "مرحبا بكم"),
        piece(2, ""),
    ]);
    notes
}

/// One note of each kind, a fault for each reason and an empty language list.
pub(super) fn every_voice_note() -> Vec<VoiceNote> {
    let mut notes = run_notes();
    notes.push(VoiceNote::Done {
        id: 3,
        pieces: 3,
        sound_ms: 61_000,
        wait_ms: 900,
        load_ms: 2_400,
        memory_mb: 1_200,
        card: true,
    });
    let faults = VoiceFault::ALL.iter();
    notes.extend(faults.map(|&fault| VoiceNote::Fault { id: 3, fault }));
    notes.extend(answer_notes());
    notes
}

/// The notes that answer an ask: devices, facts with and without languages, a check result and a sound piece.
fn answer_notes() -> Vec<VoiceNote> {
    let langs = vec!["en".into(), "fr-FR".into(), "ar".into()];
    let facts = |langs| VoiceNote::Facts {
        id: 5,
        runner: "transcribe".into(),
        langs,
    };
    vec![
        VoiceNote::Devices {
            id: 4,
            list: devices(),
        },
        facts(langs),
        facts(Vec::new()),
        VoiceNote::Checked {
            id: 6,
            card_ms: 400,
            cpu_ms: 2_900,
            card_text: text("the clip said this"),
            cpu_text: text("the clip said that"),
        },
        VoiceNote::Sound {
            id: 7,
            piece: SoundPiece {
                index: 0,
                last: true,
                bytes: Wiped::Kept(Zeroizing::new(SPOKEN.to_vec())),
            },
        },
        VoiceNote::Speech {
            id: 8,
            piece: SoundPiece {
                index: 1,
                last: true,
                bytes: Wiped::Kept(Zeroizing::new(SPOKEN.to_vec())),
            },
        },
    ]
}

/// `bytes` with the payload bytes from `at` set to `value`.
pub(super) fn patch(mut bytes: Vec<u8>, at: usize, value: &[u8]) -> Vec<u8> {
    let at = HEADER_LEN + at;
    bytes[at..at + value.len()].copy_from_slice(value);
    bytes
}
