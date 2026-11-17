//! Each voice note refusal, reading and writing: a code, flag or percentage out of range, a piece past the last index
//! or text that is not UTF-8, a done note past the longest dictation, and a bad device list or name.

use super::*;
use crate::{
    MAX_PERCENT, MAX_PIECES, MAX_SOUND_MS, MAX_SOUND_PIECES, MAX_SPEECH_PIECES, Stage, VoiceFault,
};
use IpcError::{BadField, Inconsistent, TooLong};

/// Checks that writing `note` is refused with `err` and writes nothing, as its `check` says.
fn refused_note(note: &VoiceNote, err: IpcError) {
    let mut out = Vec::new();
    assert_eq!(note.write(&mut out), Err(err), "{note:?}");
    assert!(out.is_empty());
    assert_eq!(note.check(), Err(err));
}

#[test]
fn a_code_flag_or_percent_out_of_range_is_refused() {
    let next = |n: usize| u8::try_from(n + 1).unwrap();
    let cases = [
        (Kind::Heard, vec![2]),
        (Kind::Stage, vec![0]),
        (Kind::Stage, vec![next(Stage::ALL.len())]),
        (Kind::Warm, vec![MAX_PERCENT + 1]),
        (Kind::VoiceFailed, vec![0]),
        (Kind::VoiceFailed, vec![next(VoiceFault::ALL.len())]),
    ];
    for (kind, payload) in cases {
        let bytes = frame(kind, 1, &payload);
        assert_eq!(read_voice_note(&mut &bytes[..]), Err(BadField), "{kind:?}");
    }
    refused_note(
        &VoiceNote::Warm {
            id: 1,
            percent: MAX_PERCENT + 1,
        },
        BadField,
    );
}

#[test]
fn a_piece_past_the_last_index_or_text_that_is_not_utf8_is_refused() {
    let past = u16::try_from(MAX_PIECES).unwrap();
    refused_note(
        &VoiceNote::Piece {
            id: 1,
            index: past,
            text: text("x"),
        },
        BadField,
    );
    let index = [&past.to_le_bytes()[..], b"\x01\0x"].concat();
    let bytes = frame(Kind::Piece, 1, &index);
    assert_eq!(read_voice_note(&mut &bytes[..]), Err(BadField));
    let bad = frame(Kind::Piece, 1, b"\0\0\x02\0\xff\xfe");
    assert_eq!(read_voice_note(&mut &bad[..]), Err(BadField));
    let long = text(&"x".repeat(MAX_TEXT_BYTES + 1));
    refused_note(
        &VoiceNote::Piece {
            id: 1,
            index: 0,
            text: long,
        },
        TooLong,
    );
}

/// A done note with `pieces` and `sound_ms`.
fn done(pieces: u16, sound_ms: u32) -> VoiceNote {
    VoiceNote::Done {
        id: 1,
        pieces,
        sound_ms,
        wait_ms: 0,
        load_ms: 0,
        memory_mb: 0,
        card: false,
    }
}

#[test]
fn a_done_note_past_the_longest_dictation_is_refused_both_ways() {
    let past = u16::try_from(MAX_PIECES + 1).unwrap();
    refused_note(&done(past, 0), BadField);
    refused_note(&done(0, MAX_SOUND_MS + 1), BadField);
    let good = voice_note_bytes(&done(1, 1));
    let reads = [
        (0, &past.to_le_bytes()[..]),
        (2, &(MAX_SOUND_MS + 1).to_le_bytes()),
        (18, &[2]),
    ];
    for (at, value) in reads {
        let bytes = patch(good.clone(), at, value);
        assert_eq!(read_voice_note(&mut &bytes[..]), Err(BadField), "{at}");
    }
}

#[test]
fn a_bad_device_list_is_refused_both_ways() {
    let mut list = devices();
    list[0].free_mb = list[0].total_mb + 1;
    refused_note(&VoiceNote::Devices { id: 1, list }, BadField);
    let mut list = devices();
    list[1].name = "GPU\nfake line".into();
    refused_note(&VoiceNote::Devices { id: 1, list }, BadField);
    refused_note(
        &VoiceNote::Devices {
            id: 1,
            list: Vec::new(),
        },
        Inconsistent,
    );
    let many = vec![devices().remove(0); crate::MAX_DEVICES + 1];
    refused_note(&VoiceNote::Devices { id: 1, list: many }, TooLong);
    let one = vec![devices().remove(1)];
    let good = voice_note_bytes(&VoiceNote::Devices { id: 1, list: one });
    let past = u8::try_from(DeviceKind::ALL.len() + 1).unwrap();
    let bytes = patch(good.clone(), 1, &[past]);
    assert_eq!(read_voice_note(&mut &bytes[..]), Err(BadField));
    let none = frame(Kind::DeviceList, 1, &[0]);
    assert_eq!(read_voice_note(&mut &none[..]), Err(Inconsistent));
}

/// The first and last mark of each run of format marks that reorder or hide text in a right-to-left panel.
const MARKS: [char; 10] = [
    '\u{061C}', '\u{200B}', '\u{200F}', '\u{202A}', '\u{202E}', '\u{2060}', '\u{2064}', '\u{2066}',
    '\u{2069}', '\u{FEFF}',
];

#[test]
fn a_device_name_or_driver_with_a_hidden_format_mark_is_refused_both_ways() {
    for mark in MARKS {
        let mut list = devices();
        list[1].name = format!("GPU{mark}");
        refused_note(&VoiceNote::Devices { id: 1, list }, BadField);
        let mut list = devices();
        list[1].driver = format!("{mark}32.0");
        refused_note(&VoiceNote::Devices { id: 1, list }, BadField);
    }
    let mut list = devices();
    list[1].name = "GPU abc".into();
    let good = voice_note_bytes(&VoiceNote::Devices { id: 1, list });
    let at = good.windows(3).position(|w| w == b"abc").unwrap();
    let mut bytes = good;
    bytes[at..at + 3].copy_from_slice("\u{202E}".as_bytes());
    assert_eq!(read_voice_note(&mut &bytes[..]), Err(BadField));
    let mut list = devices();
    list[1].name = "بطاقة رسومية, carte graphique".into();
    assert_eq!(VoiceNote::Devices { id: 1, list }.check(), Ok(()));
}

#[test]
fn a_speech_piece_may_pass_the_hand_overs_count_but_never_its_own() {
    let speech = |index| VoiceNote::Speech {
        id: 1,
        piece: SoundPiece {
            index,
            last: false,
            bytes: Wiped::Kept(Zeroizing::new(b"ab".to_vec())),
        },
    };
    let beyond = speech(u16::try_from(MAX_SOUND_PIECES).unwrap());
    let read = VoiceNote::read(&mut &voice_note_bytes(&beyond)[..]);
    assert_eq!(read, Ok(beyond), "past the hand-over's count");
    let past = u16::try_from(MAX_SPEECH_PIECES).unwrap();
    refused_note(&speech(past), BadField);
    let bytes = patch(voice_note_bytes(&speech(0)), 0, &past.to_le_bytes());
    assert_eq!(read_voice_note(&mut &bytes[..]), Err(BadField));
}
