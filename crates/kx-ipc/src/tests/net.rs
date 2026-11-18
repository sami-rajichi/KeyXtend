//! The net frames, both directions: the wire layout, round trips, the largest frames, each job reading only its own
//! kinds, and `Debug` that shows no key, sound, address, model or text. Refusals, rules and properties have their own
//! files.

mod cases;
mod frames;
mod ids;
mod notes;
mod props;
mod refuse;
mod rules;

use super::*;
use crate::net::{ask as net_ask, note as net_note, wire as net_wire};
use crate::wire::Spec;
use crate::{
    FetchAsk, FetchNote, HTTPS, MAX_FIXED, MAX_KEY_BYTES, MAX_NAME_BYTES, MAX_NET_TEXT_BYTES,
    MAX_SOUND_BYTES, MAX_SPEECH_PIECES, MAX_TAG_BYTES, MAX_URL_BYTES, NET_AT_ONCE, NET_MAX_S,
    NET_PER_MIN, NET_RENEW_S, NET_TRIES, NET_WAIT_S, NetAsk, NetCall, NetFault, NetField, NetForm,
    NetHeader, NetLimits, NetNote, NetRun, NetShape, SAMPLE_BYTES, VaultAsk, VaultReply, VoiceAsk,
    VoiceNote, Wiped,
};
use cases::*;
use zeroize::Zeroizing;

#[test]
fn a_test_a_sound_and_the_notes_are_little_endian_after_their_kinds() {
    let test = NetAsk::Test {
        id: 1,
        call: NetCall {
            shape: NetShape::Gemini,
            url: Cow::Borrowed("https://a/t"),
            header: NetHeader::Named(Cow::Borrowed("k")),
            key: Wiped::Lent(b"K"),
            wait_s: 60,
        },
    };
    let want = [
        &b"KXIP\x01\x33\x01\0\0\0\x16\0\0\0"[..],
        b"\x04\x0b\0https://a/t\x02\x01k\x01\0K\x3c\0",
    ]
    .concat();
    assert_eq!(net_ask_bytes(&test), want);
    let sound = NetAsk::Sound {
        id: 2,
        piece: 1,
        bytes: Wiped::Lent(b"ab"),
    };
    let want = b"KXIP\x01\x32\x02\0\0\0\x08\0\0\0\x01\0\x02\0\0\0ab";
    assert_eq!(net_ask_bytes(&sound), want);
    let piece = NetNote::Text {
        id: 3,
        index: 1,
        text: text("hi"),
    };
    let want = b"KXIP\x01\x36\x03\0\0\0\x08\0\0\0\x01\0\x02\0\0\0hi";
    assert_eq!(net_note_bytes(&piece), want);
    let renews = NetNote::Fault {
        id: 4,
        fault: NetFault::Allowance { renews_s: Some(90) },
    };
    let want = b"KXIP\x01\x38\x04\0\0\0\x06\0\0\0\x03\x01\x5a\0\0\0";
    assert_eq!(net_note_bytes(&renews), want);
    let done = net_note_bytes(&NetNote::Done { id: 5 });
    assert_eq!(done, b"KXIP\x01\x37\x05\0\0\0\0\0\0\0");
}

#[test]
fn each_net_frame_round_trips() {
    for ask in every_net_ask() {
        assert_eq!(NetAsk::read(&mut &net_ask_bytes(&ask)[..]), Ok(ask));
    }
    for note in every_net_note() {
        assert_eq!(NetNote::read(&mut &net_note_bytes(&note)[..]), Ok(note));
    }
}

/// A call whose every field is at its longest.
fn longest_call() -> NetCall<'static> {
    let tag = || Cow::Owned("t".repeat(MAX_TAG_BYTES));
    let field = NetField {
        name: tag(),
        value: Cow::Owned("v".repeat(MAX_NAME_BYTES)),
    };
    let path = "p".repeat(MAX_URL_BYTES - HTTPS.len() - "h/".len());
    NetCall {
        shape: NetShape::Form(NetForm {
            lang: tag(),
            list: true,
            fixed: vec![field; MAX_FIXED],
            paywall: true,
        }),
        url: Cow::Owned(format!("{HTTPS}h/{path}")),
        header: NetHeader::Named(tag()),
        key: Wiped::Kept(Zeroizing::new(vec![b'k'; MAX_KEY_BYTES])),
        wait_s: *NET_WAIT_S.end(),
    }
}

/// A run whose every field is at its longest, its limits at their tops but one megabyte, and the most pieces.
fn longest_run() -> NetRun<'static> {
    let tag: Cow<str> = Cow::Owned("t".repeat(MAX_TAG_BYTES));
    let limits = NetLimits {
        max_mb: 1,
        max_s: *NET_MAX_S.end(),
        per_min: *NET_PER_MIN.end(),
        at_once: *NET_AT_ONCE.end(),
    };
    let sample = u32::try_from(SAMPLE_BYTES).unwrap();
    NetRun {
        model: Some(tag.clone()),
        lang: tag,
        limits,
        tries: *NET_TRIES.end(),
        sizes: vec![sample; MAX_SPEECH_PIECES],
    }
}

#[test]
fn the_largest_send_and_test_fill_their_caps_and_round_trip() {
    let cases = [
        (
            NetAsk::Send {
                id: 1,
                call: longest_call(),
                run: longest_run(),
            },
            net_wire::SEND_MOST,
        ),
        (
            NetAsk::Test {
                id: 2,
                call: longest_call(),
            },
            net_wire::TEST_MOST,
        ),
        (
            NetAsk::Sound {
                id: 3,
                piece: u16::try_from(MAX_SPEECH_PIECES - 1).unwrap(),
                bytes: Wiped::Kept(Zeroizing::new(vec![7; MAX_SOUND_BYTES])),
            },
            net_wire::SOUND_MOST,
        ),
    ];
    for (ask, most) in cases {
        let bytes = net_ask_bytes(&ask);
        assert_eq!(bytes.len(), HEADER_LEN + most);
        assert_eq!(NetAsk::read(&mut &bytes[..]), Ok(ask));
    }
}

#[test]
fn the_longest_text_and_a_told_renewal_fill_their_caps_and_round_trip() {
    let cases = [
        (
            NetNote::Text {
                id: 1,
                index: u16::try_from(MAX_SPEECH_PIECES - 1).unwrap(),
                text: text(&"x".repeat(MAX_NET_TEXT_BYTES)),
            },
            net_wire::TEXT_MOST,
        ),
        (
            NetNote::Fault {
                id: 2,
                fault: NetFault::Allowance {
                    renews_s: Some(*NET_RENEW_S.end()),
                },
            },
            net_wire::FAULT_MOST,
        ),
    ];
    for (note, most) in cases {
        let bytes = net_note_bytes(&note);
        assert_eq!(bytes.len(), HEADER_LEN + most);
        assert_eq!(NetNote::read(&mut &bytes[..]), Ok(note));
    }
}

#[test]
fn a_run_and_its_sound_read_back_in_order_and_then_close() {
    let asks = [
        send(1, groq()),
        NetAsk::Sound {
            id: 1,
            piece: 0,
            bytes: Wiped::Lent(SPOKEN),
        },
        NetAsk::Cancel { id: 1 },
        NetAsk::Bye,
    ];
    let stream: Vec<u8> = asks.iter().flat_map(net_ask_bytes).collect();
    let mut r = &stream[..];
    for ask in asks {
        assert_eq!(NetAsk::read(&mut r), Ok(ask));
    }
    assert_eq!(NetAsk::read(&mut r), Err(IpcError::Closed));
}

#[test]
fn a_note_names_its_run_and_an_ask_its_run() {
    let ids: Vec<u32> = every_net_ask().iter().map(NetAsk::id).collect();
    assert_eq!(ids, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 0]);
    let ids: Vec<u32> = every_net_note().iter().map(NetNote::id).collect();
    assert_eq!(ids[..5], [1, 1, 1, 1, 2]);
}

#[test]
fn a_note_that_reads_back_passes_its_check() {
    for note in every_net_note() {
        assert_eq!(note.check(), Ok(()), "{note:?}");
    }
}

#[test]
fn debug_shows_no_key_sound_address_model_or_text() {
    let (asks, notes) = (every_net_ask(), every_net_note());
    let said = format!("{asks:?}{asks:#?}{notes:?}{notes:#?}");
    for secret in SECRETS {
        assert!(!said.contains(secret), "{secret} in {said}");
    }
    let sound = NetAsk::Sound {
        id: 1,
        piece: 0,
        bytes: Wiped::Lent(SPOKEN),
    };
    assert_eq!(format!("{sound:?}"), "Sound { id: 1, piece: 0, len: 12 }");
    let piece = NetNote::Text {
        id: 3,
        index: 1,
        text: text("bonjour"),
    };
    assert_eq!(format!("{piece:?}"), "Text { id: 3, index: 1, len: 7 }");
    let send = format!("{:?}", send(1, groq()));
    for part in ["lang: \"fr\"", "Bearer", "max_mb: 25", "pieces: 1"] {
        assert!(send.contains(part), "{send} lacks {part}");
    }
    assert!(!send.contains("key") && !send.contains("Wiped"), "{send}");
}

#[test]
fn a_cut_net_frame_is_short_and_an_empty_stream_closed() {
    let asks = every_net_ask();
    let asks = asks
        .iter()
        .map(|a| (net_ask_bytes(a), read_net_ask as Reader));
    let notes = every_net_note();
    let notes = notes
        .iter()
        .map(|n| (net_note_bytes(n), read_net_note as Reader));
    for (bytes, read) in asks.chain(notes) {
        assert_eq!(read(&mut &bytes[..0]), Err(IpcError::Closed));
        for cut in 1..bytes.len() {
            let got = read(&mut &bytes[..cut]);
            assert_eq!(got, Err(IpcError::Short), "cut at {cut}");
        }
    }
}

#[test]
fn each_net_frame_is_refused_by_every_other_reader() {
    let others: [Reader; 8] = [
        read_ask,
        read_reply,
        |r| VaultAsk::read(r).map(drop),
        |r| VaultReply::read(r).map(drop),
        |r| VoiceAsk::read(r).map(drop),
        |r| VoiceNote::read(r).map(drop),
        |r| FetchAsk::read(r).map(drop),
        |r| FetchNote::read(r).map(drop),
    ];
    let asks: Vec<_> = every_net_ask().iter().map(net_ask_bytes).collect();
    let notes: Vec<_> = every_net_note().iter().map(net_note_bytes).collect();
    for bytes in asks.iter().chain(&notes) {
        for read in others {
            assert_eq!(read(&mut &bytes[..]), Err(IpcError::BadKind));
        }
    }
    for bytes in &asks {
        assert_eq!(read_net_note(&mut &bytes[..]), Err(IpcError::BadKind));
    }
    for bytes in &notes {
        assert_eq!(read_net_ask(&mut &bytes[..]), Err(IpcError::BadKind));
    }
}

#[test]
fn the_key_and_fixed_caps_are_the_frames_own_and_fit_their_widths() {
    let fits = |spec: Spec| u64::try_from(spec.max).unwrap() < 1u64 << (8 * spec.width);
    assert_eq!(net_wire::KEY.max, MAX_KEY_BYTES, "the port's key cap");
    assert_eq!(net_wire::FIXED.max, MAX_FIXED, "the port's fixed fields");
    assert!(fits(net_wire::KEY) && fits(net_wire::FIXED));
}
