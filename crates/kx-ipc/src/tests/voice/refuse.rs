//! Each voice ask refusal, and those every frame shares: a length past its kind, a wrong version, a setting outside its
//! range, a bad name, target or path, sound that is not whole samples, extra bytes, and a goodbye with an id.

use super::*;
use crate::MAX_SOUND_PIECES;
use IpcError::{BadField, BadVersion, Inconsistent, TooLong};

/// One change to a field of a good value.
type Change<T> = fn(&mut T);

/// A listen-only start whose detector is `d`.
fn quiet_start() -> VoiceAsk<'static> {
    VoiceAsk::Start {
        id: 1,
        setup: Setup {
            detector: Cow::Borrowed("d"),
            ..setup(None)
        },
    }
}

/// Checks that writing `ask` is refused with `err` and writes nothing.
fn refused_ask(ask: &VoiceAsk, err: IpcError) {
    let mut out = Vec::new();
    assert_eq!(ask.write(&mut out), Err(err), "{ask:?}");
    assert!(out.is_empty());
}

#[test]
fn a_length_past_its_kind_is_refused_before_the_payload() {
    let plenty = u64::try_from(READ_CHUNK).unwrap();
    for &kind in voice_ask::KINDS.iter().chain(voice_note::KINDS) {
        let over = u32::try_from(*kind.lens().end() + 1).unwrap();
        let read: Reader = if voice_ask::KINDS.contains(&kind) {
            read_voice_ask
        } else {
            read_voice_note
        };
        for len in [over, u32::MAX] {
            let raw = head(MAGIC, VERSION, kind.code(), 0, len);
            let mut r = Counted::new((&raw[..]).chain(io::repeat(0).take(plenty)));
            assert_eq!(read(&mut r), Err(TooLong), "{kind:?} {len}");
            assert_eq!(r.taken, HEADER_LEN);
        }
    }
}

#[test]
fn a_wrong_version_or_a_length_off_by_one_is_refused_for_every_voice_frame() {
    let asks = every_voice_ask();
    let asks = asks
        .iter()
        .map(|a| (voice_ask_bytes(a), read_voice_ask as Reader));
    let notes = every_voice_note();
    let notes = notes
        .iter()
        .map(|n| (voice_note_bytes(n), read_voice_note as Reader));
    for (bytes, read) in asks.chain(notes) {
        let mut old = bytes.clone();
        old[MAGIC.len()] = VERSION + 1;
        assert_eq!(read(&mut &old[..]), Err(BadVersion));
        let mut long = bytes.clone();
        long[HEADER_LEN - 4] = long[HEADER_LEN - 4].wrapping_add(1);
        long.push(0);
        let got = read(&mut &long[..]);
        assert!(matches!(got, Err(Inconsistent | TooLong)), "{got:?}");
        let mut short = bytes.clone();
        short[HEADER_LEN - 4] = short[HEADER_LEN - 4].wrapping_sub(1);
        assert!(read(&mut &short[..]).is_err());
    }
}

#[test]
fn a_setting_outside_its_range_is_refused_when_written() {
    let changes: [Change<Setup>; 10] = [
        |s| s.pad_ms = 2001,
        |s| s.piece_s = 9,
        |s| s.piece_s = 121,
        |s| s.record_max_s = 9,
        |s| s.record_max_s = 3601,
        |s| s.level_hz = 4,
        |s| s.level_hz = 61,
        |s| s.silence_stop_s = Some(0),
        |s| s.silence_stop_s = Some(31),
        |s| {
            s.model = Some(Model {
                threads: 65,
                ..model(Target::Cpu)
            });
        },
    ];
    for change in changes {
        let mut setup = setup(None);
        change(&mut setup);
        refused_ask(&VoiceAsk::Start { id: 1, setup }, BadField);
    }
}

#[test]
fn a_setting_outside_its_range_is_refused_when_read() {
    let good = voice_ask_bytes(&quiet_start());
    let reads = [
        (0, &[2][..]),
        (1, &2001u16.to_le_bytes()),
        (3, &[9]),
        (3, &[121]),
        (4, &3601u16.to_le_bytes()),
        (6, &[61]),
        (7, &[31]),
        (10, b"\0"),
    ];
    for (at, value) in reads {
        let bytes = patch(good.clone(), at, value);
        let got = read_voice_ask(&mut &bytes[..]);
        assert_eq!(got, Err(BadField), "{at} {value:?}");
    }
}

#[test]
fn a_bad_name_target_or_path_in_a_model_is_refused_when_read() {
    let good = voice_ask_bytes(&small_check());
    let reads = [
        (4, b" "),
        (4, b"\xc3"),
        (8, b"\0"),
        // Built-in graphics is a device kind, aimed at as a card by its id, never a target's kind.
        (8, b"\x03"),
        (16, b"\x41"),
        (19, b"\0"),
    ];
    for (at, value) in reads {
        let bytes = patch(good.clone(), at, value);
        let got = read_voice_ask(&mut &bytes[..]);
        assert_eq!(got, Err(BadField), "{at} {value:?}");
    }
}

#[test]
fn a_bad_name_or_path_in_a_model_is_refused_when_written() {
    let changes: [(Change<Model>, IpcError); 5] = [
        (|m| m.lang = Cow::Borrowed("fr FR"), BadField),
        (|m| m.runner = Cow::Borrowed("é"), BadField),
        (|m| m.lang = Cow::Borrowed(""), Inconsistent),
        (|m| m.path = Cow::Borrowed("a\0b"), BadField),
        (|m| m.path = Cow::Borrowed(""), Inconsistent),
    ];
    for (change, err) in changes {
        let mut model = model(Target::Cpu);
        change(&mut model);
        let clip = Cow::Borrowed("k");
        refused_ask(&VoiceAsk::Check { id: 1, model, clip }, err);
    }
    let long = Cow::Owned("x".repeat(crate::MAX_PATH_BYTES + 1));
    refused_ask(&VoiceAsk::Facts { id: 1, path: long }, TooLong);
}

#[test]
fn sound_that_is_not_whole_samples_or_past_the_last_piece_is_refused_both_ways() {
    let past = u16::try_from(MAX_SOUND_PIECES).unwrap();
    refused_ask(
        &VoiceAsk::Sound {
            id: 1,
            piece: sound(0, true, b"odd"),
        },
        BadField,
    );
    refused_ask(
        &VoiceAsk::Sound {
            id: 1,
            piece: sound(past, true, b""),
        },
        BadField,
    );
    let good = voice_ask_bytes(&VoiceAsk::Sound {
        id: 1,
        piece: sound(0, false, b"ab"),
    });
    let reads = [(0, &past.to_le_bytes()[..]), (2, &[2]), (3, &[1])];
    for (at, value) in reads {
        let mut bytes = patch(good.clone(), at, value);
        if at == 3 {
            bytes.truncate(bytes.len() - 1);
            bytes[HEADER_LEN - 4] -= 1;
        }
        assert_eq!(read_voice_ask(&mut &bytes[..]), Err(BadField), "{at}");
    }
}

#[test]
fn bytes_after_the_last_field_are_inconsistent() {
    let mut payload = voice_ask_bytes(&quiet_start())[HEADER_LEN..].to_vec();
    payload.push(0);
    let bytes = frame(Kind::VoiceStart, 1, &payload);
    assert_eq!(read_voice_ask(&mut &bytes[..]), Err(Inconsistent));
    let facts = frame(Kind::FileFacts, 1, b"\x01t\0\0");
    assert_eq!(read_voice_note(&mut &facts[..]), Err(Inconsistent));
}

#[test]
fn a_goodbye_with_an_id_is_inconsistent() {
    let bytes = frame(Kind::VoiceBye, 5, &[]);
    assert_eq!(read_voice_ask(&mut &bytes[..]), Err(Inconsistent));
}
