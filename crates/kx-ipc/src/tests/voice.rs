//! The voice frames, both directions: the wire layout, round trips, the largest frames, each job reading only its own
//! kinds, and `Debug` that shows no text, sound or path. Refusals, limits and properties have their own files.

mod cases;
mod kinds;
mod limits;
mod notes;
mod props;
mod refuse;

use super::*;
use crate::voice::{ask as voice_ask, note as voice_note, wire as voice_wire};
use crate::{
    Device, DeviceKind, MAX_DEVICES, MAX_LANGS, MAX_NAME_BYTES, MAX_PATH_BYTES, MAX_SOUND_BYTES,
    MAX_TAG_BYTES, MAX_TEXT_BYTES, Model, Setup, SoundPiece, Target, VoiceAsk, VoiceNote, Wiped,
};
use cases::*;
use std::ops::RangeInclusive;
use zeroize::Zeroizing;

#[test]
fn a_stop_and_a_level_are_little_endian_after_their_kinds() {
    let stop = VoiceAsk::Stop { id: 7 };
    assert_eq!(voice_ask_bytes(&stop), b"KXIP\x01\x14\x07\0\0\0\0\0\0\0");
    let level = VoiceNote::Level { id: 3, level: 9 };
    assert_eq!(
        voice_note_bytes(&level),
        b"KXIP\x01\x1e\x03\0\0\0\x01\0\0\0\x09"
    );
}

#[test]
fn a_listen_only_start_sends_no_model_then_its_numbers_and_detector() {
    let ask = VoiceAsk::Start {
        id: 1,
        setup: Setup {
            detector: Cow::Borrowed("d"),
            ..setup(None)
        },
    };
    let want = [
        &b"KXIP\x01\x13\x01\0\0\0\x0b\0\0\0"[..],
        b"\0\x2c\x01\x1e\x58\x02\x1e\0",
        b"\x01\0d",
    ]
    .concat();
    assert_eq!(voice_ask_bytes(&ask), want);
}

/// A check of a small model on card `c0` with clip `k`.
pub(super) fn small_check() -> VoiceAsk<'static> {
    let model = Model {
        path: Cow::Borrowed("m"),
        lang: Cow::Borrowed("ar"),
        runner: Cow::Borrowed("t"),
        target: Target::Card {
            id: Cow::Borrowed("c0"),
            need_mb: 5,
        },
        threads: 4,
    };
    VoiceAsk::Check {
        id: 2,
        model,
        clip: Cow::Borrowed("k"),
    }
}

#[test]
fn a_check_sends_its_model_on_its_card_then_the_clip() {
    let want = [
        &b"KXIP\x01\x1b\x02\0\0\0\x14\0\0\0"[..],
        b"\x01\0m\x02ar\x01t",
        b"\x02\x02c0\x05\0\0\0\x04",
        b"\x01\0k",
    ]
    .concat();
    assert_eq!(voice_ask_bytes(&small_check()), want);
}

#[test]
fn each_voice_frame_round_trips() {
    for ask in every_voice_ask() {
        assert_eq!(VoiceAsk::read(&mut &voice_ask_bytes(&ask)[..]), Ok(ask));
    }
    for note in every_voice_note() {
        let bytes = voice_note_bytes(&note);
        assert_eq!(VoiceNote::read(&mut &bytes[..]), Ok(note));
    }
}

/// The top of `range`, or its bottom.
fn end<T: Copy>(range: &RangeInclusive<T>, top: bool) -> T {
    if top { *range.end() } else { *range.start() }
}

#[test]
fn each_setting_at_either_end_of_its_range_round_trips() {
    use crate::{LEVEL_HZ, PAD_MS, PIECE_S, RECORD_MAX_S, SILENCE_STOP_S, THREADS};
    for top in [false, true] {
        let setup = Setup {
            model: Some(Model {
                threads: end(&THREADS, top),
                ..model(Target::Cpu)
            }),
            pad_ms: end(&PAD_MS, top),
            piece_s: end(&PIECE_S, top),
            record_max_s: end(&RECORD_MAX_S, top),
            level_hz: end(&LEVEL_HZ, top),
            silence_stop_s: Some(end(&SILENCE_STOP_S, top)),
            ..setup(None)
        };
        let ask = VoiceAsk::Again { id: 1, setup };
        assert_eq!(VoiceAsk::read(&mut &voice_ask_bytes(&ask)[..]), Ok(ask));
    }
}

#[test]
fn the_largest_sound_piece_round_trips_both_ways_and_fills_its_cap() {
    let largest = vec![7; MAX_SOUND_BYTES];
    let ask = VoiceAsk::Sound {
        id: 1,
        piece: sound(0, false, &largest),
    };
    let bytes = voice_ask_bytes(&ask);
    assert_eq!(bytes.len(), HEADER_LEN + voice_wire::SOUND_MOST);
    assert_eq!(VoiceAsk::read(&mut &bytes[..]), Ok(ask));
    let note = VoiceNote::Sound {
        id: 2,
        piece: SoundPiece {
            index: 1,
            last: true,
            bytes: Wiped::Kept(Zeroizing::new(largest)),
        },
    };
    let bytes = voice_note_bytes(&note);
    assert_eq!(VoiceNote::read(&mut &bytes[..]), Ok(note));
}

/// A model whose every name is at its longest, on a card.
fn longest_model() -> Model<'static> {
    let (path, tag) = ("p".repeat(MAX_PATH_BYTES), "t".repeat(MAX_TAG_BYTES));
    Model {
        path: Cow::Owned(path),
        lang: Cow::Owned(tag.clone()),
        runner: Cow::Owned(tag.clone()),
        target: Target::Card {
            id: Cow::Owned(tag),
            need_mb: u32::MAX,
        },
        threads: 1,
    }
}

#[test]
fn the_largest_setup_and_check_fill_their_caps_and_round_trip() {
    let detector: Cow<'static, str> = Cow::Owned("d".repeat(MAX_PATH_BYTES));
    let start = VoiceAsk::Start {
        id: 1,
        setup: Setup {
            model: Some(longest_model()),
            detector: detector.clone(),
            ..setup(None)
        },
    };
    let bytes = voice_ask_bytes(&start);
    assert_eq!(bytes.len(), HEADER_LEN + voice_wire::SETUP_MOST);
    assert_eq!(VoiceAsk::read(&mut &bytes[..]), Ok(start));
    let check = VoiceAsk::Check {
        id: 2,
        model: longest_model(),
        clip: detector,
    };
    let bytes = voice_ask_bytes(&check);
    assert_eq!(bytes.len(), HEADER_LEN + voice_wire::CHECK_MOST);
    assert_eq!(VoiceAsk::read(&mut &bytes[..]), Ok(check));
}

#[test]
fn the_largest_device_list_facts_and_check_result_fill_their_caps() {
    let device = Device {
        kind: DeviceKind::Card,
        id: "i".repeat(MAX_TAG_BYTES),
        name: "n".repeat(MAX_NAME_BYTES),
        total_mb: u32::MAX,
        free_mb: u32::MAX,
        driver: "v".repeat(MAX_NAME_BYTES),
    };
    let list = vec![device; MAX_DEVICES];
    let notes = [
        (VoiceNote::Devices { id: 1, list }, voice_wire::DEVICES_MOST),
        (
            VoiceNote::Facts {
                id: 2,
                runner: "r".repeat(MAX_TAG_BYTES),
                langs: vec!["l".repeat(MAX_TAG_BYTES); MAX_LANGS],
            },
            voice_wire::FACTS_MOST,
        ),
        (
            VoiceNote::Checked {
                id: 3,
                card_ms: u32::MAX,
                cpu_ms: u32::MAX,
                card_text: text(&"c".repeat(MAX_TEXT_BYTES)),
                cpu_text: text(&"p".repeat(MAX_TEXT_BYTES)),
            },
            voice_wire::CHECKED_MOST,
        ),
    ];
    for (note, most) in notes {
        let bytes = voice_note_bytes(&note);
        assert_eq!(bytes.len(), HEADER_LEN + most);
        assert_eq!(VoiceNote::read(&mut &bytes[..]), Ok(note));
    }
}

#[test]
fn a_note_names_its_ask_and_an_ask_its_run() {
    let ids: Vec<u32> = every_voice_note().iter().map(VoiceNote::id).collect();
    assert_eq!(ids[..4], [1, 1, 1, 1]);
    let ids: Vec<u32> = every_voice_ask().iter().map(VoiceAsk::id).collect();
    assert_eq!(ids[..11], [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
    assert_eq!(VoiceAsk::Bye.id(), 0);
}

#[test]
fn a_note_that_reads_back_passes_its_check() {
    for note in every_voice_note() {
        assert_eq!(note.check(), Ok(()), "{note:?}");
    }
}

#[test]
fn debug_shows_no_text_sound_or_path() {
    let said = format!("{:?}{:?}", every_voice_ask(), every_voice_note());
    for secret in SECRETS {
        assert!(!said.contains(secret), "{secret} in {said}");
    }
    let piece = VoiceNote::Piece {
        id: 3,
        index: 1,
        text: text("bonjour"),
    };
    assert_eq!(format!("{piece:?}"), "Piece { id: 3, index: 1, len: 7 }");
    let stop = format!("{:?}", VoiceAsk::Stop { id: 4 });
    assert_eq!(stop, "Stop { id: 4 }");
    let notes = every_voice_note();
    let done = notes.iter().find(|n| matches!(n, VoiceNote::Done { .. }));
    let done = format!("{:?}", done.unwrap());
    assert!(
        done.ends_with("load_ms: 2400, memory_mb: 1200, card: true }"),
        "{done}"
    );
}

#[test]
fn a_cut_voice_frame_is_short_and_an_empty_stream_closed() {
    let asks = every_voice_ask();
    let asks = asks
        .iter()
        .map(|a| (voice_ask_bytes(a), read_voice_ask as Reader));
    let notes = every_voice_note();
    let notes = notes
        .iter()
        .map(|n| (voice_note_bytes(n), read_voice_note as Reader));
    for (bytes, read) in asks.chain(notes) {
        assert_eq!(read(&mut &bytes[..0]), Err(IpcError::Closed));
        for cut in 1..bytes.len() {
            let got = read(&mut &bytes[..cut]);
            assert_eq!(got, Err(IpcError::Short), "cut at {cut}");
        }
    }
}

#[test]
fn voice_frames_in_one_stream_read_back_in_order_and_then_close() {
    let notes = every_voice_note();
    let stream: Vec<u8> = notes.iter().flat_map(voice_note_bytes).collect();
    let mut r = &stream[..];
    for note in notes {
        assert_eq!(VoiceNote::read(&mut r), Ok(note));
    }
    assert_eq!(VoiceNote::read(&mut r), Err(IpcError::Closed));
}
