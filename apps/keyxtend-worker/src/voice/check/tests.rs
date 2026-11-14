//! A file's facts, and the card's check (edge case 27): the fit check, the warm-up's progress, both texts and times
//! handed back for the module to score, and a card fault that ends the check.

use super::*;
use crate::voice::Event;
use crate::voice::fake::lib::{FakeLib, device};
use kx_ipc::{DeviceKind, MAX_LANGS, Target};
use std::borrow::Cow;
use std::sync::mpsc::{self, Receiver};

/// A shipped check clip.
fn clip_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/voice/check-en.wav")
}

/// A fake library with the processor and card `Vulkan0`, `free_mb` free on it, whose models know English.
fn lib(free_mb: u64) -> FakeLib {
    let lib = FakeLib::default();
    let mut s = lib.script();
    s.found = vec![
        device(DeviceKind::Cpu, "CPU", 8000),
        device(DeviceKind::Card, "Vulkan0", free_mb),
    ];
    s.langs = vec!["en".into(), "fr".into()];
    drop(s);
    lib
}

/// A check of an English model on `Vulkan0`, needing 1000 MB, with `clip`; and where its notes go.
fn exam(clip: PathBuf) -> (Exam, Receiver<Event>) {
    let (tx, rx) = mpsc::channel();
    let model = Model {
        path: Cow::Borrowed("model.gguf"),
        lang: Cow::Borrowed("en"),
        runner: Cow::Borrowed(FakeLib::RUNNER),
        target: Target::Card {
            id: Cow::Borrowed("Vulkan0"),
            need_mb: 1000,
        },
        threads: 0,
    };
    let tell = Tell::new(tx);
    (
        Exam {
            id: 7,
            model,
            clip,
            tell,
        },
        rx,
    )
}

fn told(rx: &Receiver<Event>) -> Vec<VoiceNote> {
    rx.try_iter()
        .filter_map(|e| match e {
            Event::Note(note) => Some(note),
            _ => None,
        })
        .collect()
}

fn fault(fault: VoiceFault) -> VoiceNote {
    VoiceNote::Fault { id: 7, fault }
}

#[test]
fn the_check_warms_up_then_returns_both_texts_and_times() {
    let (lib, (exam, rx)) = (lib(2000), exam(clip_path()));
    let note = check(&lib, 8, &exam);
    let VoiceNote::Checked {
        id,
        card_text,
        cpu_text,
        ..
    } = note
    else {
        panic!("not checked: {note:?}");
    };
    let samples = clip(&clip_path()).unwrap().len();
    assert_eq!(
        (id, card_text.as_str()),
        (7, format!("{samples} en").as_str())
    );
    assert_eq!(*cpu_text, *card_text);
    let warm = |percent| VoiceNote::Warm { id: 7, percent };
    let loading = VoiceNote::Stage {
        id: 7,
        stage: Stage::Loading,
    };
    assert_eq!(told(&rx), [loading, warm(0), warm(MAX_PERCENT)]);
    let s = lib.script();
    assert_eq!(
        s.loads,
        [(Place::Card("Vulkan0".into()), 8), (Place::Cpu, 8)]
    );
    let places: Vec<bool> = s.runs.iter().map(|r| r.0.card()).collect();
    assert_eq!(
        places,
        [true, true, false],
        "the warm-up, then one timed run on each"
    );
}

#[test]
fn a_card_that_computes_wrongly_hands_back_its_own_text_for_the_module_to_score() {
    let lib = lib(2000);
    lib.script().card_text = Some("other words".into());
    let VoiceNote::Checked {
        card_text,
        cpu_text,
        ..
    } = check(&lib, 8, &exam(clip_path()).0)
    else {
        panic!("not checked");
    };
    let samples = clip(&clip_path()).unwrap().len();
    assert_eq!(*card_text, "other words");
    assert_eq!(
        *cpu_text,
        format!("{samples} en"),
        "the processor's own text"
    );
}

#[test]
fn a_card_short_of_room_or_gone_ends_the_check_before_any_load() {
    for (free, gone, why) in [
        (999, false, VoiceFault::CardRoom),
        (5000, true, VoiceFault::NoCard),
    ] {
        let lib = lib(free);
        if gone {
            lib.script().found.truncate(1);
        }
        let (exam, rx) = exam(clip_path());
        assert_eq!(check(&lib, 8, &exam), fault(why));
        assert!(told(&rx).is_empty() && lib.script().loads.is_empty());
    }
}

#[test]
fn a_card_that_fails_to_load_or_run_fails_the_check() {
    let failing = lib(2000);
    failing.script().card_load = Some(Failed::Device);
    assert_eq!(
        check(&failing, 8, &exam(clip_path()).0),
        fault(VoiceFault::CardFailed)
    );
    let running = lib(2000);
    running.script().card_runs = Some(1);
    assert_eq!(
        check(&running, 8, &exam(clip_path()).0),
        fault(VoiceFault::CardFailed)
    );
    let unread = lib(2000);
    unread.script().card_load = Some(Failed::BadFile);
    assert_eq!(
        check(&unread, 8, &exam(clip_path()).0),
        fault(VoiceFault::BadFile)
    );
}

#[test]
fn a_model_without_the_clips_language_or_a_bad_clip_fails_the_check() {
    let lib = lib(2000);
    lib.script().langs = vec!["fr".into()];
    assert_eq!(
        check(&lib, 8, &exam(clip_path()).0),
        fault(VoiceFault::Language)
    );
    let missing = clip_path().with_extension("none");
    assert_eq!(check(&lib, 8, &exam(missing).0), fault(VoiceFault::NoFile));
    let not_wav = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    assert_eq!(check(&lib, 8, &exam(not_wav).0), fault(VoiceFault::BadFile));
}

#[test]
fn facts_name_the_runner_and_the_codes_the_frames_take() {
    let lib = lib(0);
    let mut codes: Vec<String> = (0..MAX_LANGS + 5).map(|k| format!("l{k}")).collect();
    codes.insert(0, "bad code".into());
    lib.script().langs = codes;
    let VoiceNote::Facts { id, runner, langs } = facts(&lib, 3, Path::new("m.gguf")) else {
        panic!("no facts");
    };
    assert_eq!((id, runner.as_str()), (3, FakeLib::RUNNER));
    assert_eq!(langs.len(), MAX_LANGS);
    assert_eq!(langs[0], "l0");
    assert_eq!(lib.script().read, [PathBuf::from("m.gguf")]);
}

#[test]
fn a_file_that_cannot_be_read_is_bad_unless_missing() {
    for (why, want) in [
        (Failed::NoFile, VoiceFault::NoFile),
        (Failed::Other, VoiceFault::BadFile),
        (Failed::Memory, VoiceFault::Memory),
    ] {
        let lib = lib(0);
        lib.script().cpu_load = Some(why);
        let got = facts(&lib, 3, Path::new("m.gguf"));
        assert_eq!(got, VoiceNote::Fault { id: 3, fault: want });
    }
}

#[test]
fn a_clip_becomes_16_khz_mono_however_it_was_stored() {
    let en = clip(&clip_path()).unwrap();
    assert!((5 * SOUND_HZ..=6 * SOUND_HZ).contains(&en.len()));
    let dir = std::env::temp_dir().join(format!("kx-clip-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let stereo = dir.join("stereo.wav");
    std::fs::write(&stereo, stereo_wav(48_000, 4800)).unwrap();
    let got = clip(&stereo).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    assert!((1590..=1610).contains(&got.len()), "{}", got.len());
}

/// A 16-bit stereo WAV of `frames` silent frames at `rate`.
fn stereo_wav(rate: u32, frames: u32) -> Vec<u8> {
    let data = frames * 4;
    let mut out = b"RIFF".to_vec();
    out.extend_from_slice(&(36 + data).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16_u32.to_le_bytes());
    for v in [1_u16, 2] {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out.extend_from_slice(&rate.to_le_bytes());
    out.extend_from_slice(&(rate * 4).to_le_bytes());
    for v in [4_u16, 16] {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data.to_le_bytes());
    out.resize(out.len() + data as usize, 0);
    out
}
