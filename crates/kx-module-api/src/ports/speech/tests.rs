//! The speech port: its gate, a card's id and the `device` pick, notes and checks that hide text and sound, the
//! verdicts' numbers, the errors and the frames' faults they mirror. The asks' ranges are in `ask`, the job's in `job`.

use super::*;
use crate::{REDACTED_MARKER, Redacted, Secret, Zeroizing};
use kx_ipc::VoiceFault;
use std::collections::HashSet;

mod ask;
mod job;

/// A transcript no output may show.
const SAID: &str = "bonjour-7731";
/// Sound bytes no output may show.
const SOUND: [u8; 4] = [0x7f, 0x3e, 0x11, 0x5a];

/// `text` as a secret.
fn secret(text: &str) -> Secret {
    Redacted::new(Zeroizing::new(text.to_owned()))
}

#[test]
fn the_speech_service_is_named_and_gated_by_the_microphone() {
    assert_eq!(SpeechKey::ID.as_str(), "speech");
    let gate = Some(Capability::Microphone);
    assert_eq!(SpeechKey::CAPABILITY, gate, "ADR-0042 decision 8");
}

#[test]
fn a_card_id_is_the_engines_plain_name() {
    for good in ["Vulkan0", "vulkan-1", "Metal_2", "card.3"] {
        let id = CardId::new(good).map(|c| c.as_str().to_owned());
        assert_eq!(id.as_deref(), Some(good));
    }
    let long = "a".repeat(*TAG_LEN.end() + 1);
    for bad in ["", "Vulkan 0", "card/1", "..\\x", "مرحبا", long.as_str()] {
        assert_eq!(CardId::new(bad), None, "{bad:?}");
    }
    let full = "a".repeat(*TAG_LEN.end());
    assert!(CardId::new(&full).is_some(), "the frames' cap is taken");
}

#[test]
fn a_card_id_never_takes_a_picks_own_name() {
    for name in [DevicePick::AUTO, DevicePick::CPU] {
        assert_eq!(CardId::new(name), None, "{name}");
    }
}

#[test]
fn the_device_pick_reads_and_writes_auto_cpu_or_a_card() {
    let card = CardId::new("Vulkan1").unwrap();
    let picks = [DevicePick::Auto, DevicePick::Cpu, DevicePick::Card(card)];
    let names = picks.clone().map(|p| p.name().to_owned());
    assert_eq!(names, ["auto", "cpu", "Vulkan1"], "ADR-0044 decision 8");
    for (pick, name) in picks.into_iter().zip(names) {
        assert_eq!(DevicePick::parse(&name), Some(pick));
    }
    let bad = DevicePick::parse("gpu one");
    assert_eq!(bad, None, "edge 35: back to the default");
    assert_eq!(DevicePick::parse(""), None);
}

#[test]
fn a_piece_prints_its_place_and_never_its_text() {
    let note = SpeechNote::Piece {
        index: 2,
        text: secret(SAID),
    };
    let shown = format!("{note:?}");
    assert!(!shown.contains(SAID), "{shown}");
    let parts = shown.contains("index: 2") && shown.contains(REDACTED_MARKER);
    assert!(parts, "{shown}");
}

#[test]
fn kept_sound_never_prints() {
    let sound: SecretBytes = Redacted::new(Zeroizing::new(SOUND.to_vec()));
    let shown = format!("{:?}", vec![sound]);
    assert_eq!(shown, format!("[{REDACTED_MARKER}]"));
}

#[test]
fn the_notes_carry_numbers_only_besides_a_pieces_text() {
    let done = SpeechDone {
        pieces: 3,
        sound_ms: 41_000,
        wait_ms: 900,
        load_ms: 0,
        memory_mb: 1_650,
        card: true,
    };
    let notes = [
        SpeechNote::Stage(SpeechStage::Open),
        SpeechNote::Level(LEVEL_FULL),
        SpeechNote::Warming {
            percent: MAX_PERCENT,
        },
        SpeechNote::Done(done),
        SpeechNote::Fault(SpeechError::Unplugged),
    ];
    let shown = format!("{notes:?}");
    let parts = [
        "Open",
        "Level(255)",
        "percent: 100",
        "sound_ms: 41000",
        "Unplugged",
    ];
    for part in parts {
        assert!(shown.contains(part), "{shown} lacks {part}");
    }
}

#[test]
fn a_checks_texts_never_print() {
    let done = CheckDone {
        card_ms: 2_100,
        cpu_ms: 2_900,
        card_text: secret(SAID),
        cpu_text: secret(SAID),
    };
    let shown = format!("{done:?}");
    assert!(
        !shown.contains(SAID) && shown.contains("cpu_ms: 2900"),
        "{shown}"
    );
}

#[test]
fn a_verdict_carries_the_numbers_its_mark_and_note_show() {
    let (gain, need) = (Hundredths(900), Hundredths(150));
    let verdicts = [
        Verdict::Passed { gain },
        Verdict::Slow {
            gain: Hundredths(120),
            need,
        },
        Verdict::Differs {
            words: 3,
            allowed: 1,
        },
        Verdict::NoRoom,
        Verdict::Failed,
    ];
    let shown = format!("{verdicts:?}");
    let parts = [
        "Hundredths(900)",
        "need: Hundredths(150)",
        "words: 3, allowed: 1",
    ];
    for part in parts {
        assert!(shown.contains(part), "edge 27: the card says why; {shown}");
    }
    assert_eq!(Hundredths::PER_ONE, 100, "1.5× is 150");
}

#[test]
fn only_a_card_fault_leaves_the_run_going() {
    let goes_on = [
        SpeechError::NoCard,
        SpeechError::CardRoom,
        SpeechError::CardFailed,
    ];
    assert!(goes_on.iter().all(|e| !e.ends()), "the processor serves");
    let ends = [
        SpeechError::Unplugged,
        SpeechError::Memory,
        SpeechError::Gone,
    ];
    assert!(ends.iter().all(|e| e.ends()));
}

#[test]
fn each_frame_fault_maps_to_its_own_error() {
    let all = [
        VoiceFault::NoMic,
        VoiceFault::MicBusy,
        VoiceFault::MicRefused,
        VoiceFault::Unplugged,
        VoiceFault::NoFile,
        VoiceFault::BadFile,
        VoiceFault::Language,
        VoiceFault::Runner,
        VoiceFault::NoCard,
        VoiceFault::CardRoom,
        VoiceFault::CardFailed,
        VoiceFault::NoSound,
        VoiceFault::OutOfStep,
        VoiceFault::Memory,
        VoiceFault::Internal,
    ];
    let mapped: HashSet<_> = all.iter().map(|&f| SpeechError::from(f)).collect();
    assert_eq!(mapped.len(), all.len());
    assert_eq!(
        SpeechError::from(VoiceFault::CardRoom),
        SpeechError::CardRoom
    );
}

#[test]
fn each_error_says_something_else() {
    let errors = [
        SpeechError::Range("threads"),
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
        SpeechError::Down {
            retry_in: std::time::Duration::from_secs(1),
        },
    ];
    let texts: HashSet<_> = errors.iter().map(ToString::to_string).collect();
    assert_eq!(texts.len(), errors.len());
    assert!(SpeechError::Range("pad_ms").to_string().contains("pad_ms"));
}
