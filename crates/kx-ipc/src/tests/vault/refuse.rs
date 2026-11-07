//! Each vault refusal, reading and writing: a length past its kind or field cap before any payload is allocated, a
//! field past its frame, extra bytes, a count past its cap, an unknown or repeated part, and a field out of range.

use super::*;
use crate::vault::body::part_name;
use crate::{MAX_PARTS, MAX_PASSWORD_BYTES};
use IpcError::{BadField, Inconsistent, TooLong};

/// The dropped counts before a `KeePass` reply's entries, when nothing was dropped.
pub(super) const NONE_DROPPED: [u8; 8] = [0; 8];

/// A field's length as its two-byte prefix.
pub(super) fn two(n: usize) -> [u8; 2] {
    u16::try_from(n).unwrap().to_le_bytes()
}

/// A frame of vault `kind` with `payload` under its true length.
pub(super) fn vault_frame(kind: Kind, payload: &[u8]) -> Vec<u8> {
    frame(kind, 1, payload)
}

/// The payload of a check of `password` against `locked`.
fn check_payload(password: &[u8], locked: &[u8]) -> Vec<u8> {
    [
        &two(password.len())[..],
        password,
        &two(locked.len()),
        locked,
    ]
    .concat()
}

#[test]
fn a_length_past_its_kind_is_refused_before_the_payload() {
    let plenty = u64::try_from(READ_CHUNK).unwrap();
    let kinds = vault_ask::KINDS.iter().chain(vault_reply::KINDS);
    for &kind in kinds {
        let over = u32::try_from(*kind.lens().end() + 1).unwrap();
        for len in [over, u32::MAX] {
            let raw = head(MAGIC, VERSION, kind.code(), 0, len);
            let read: Reader = if vault_ask::KINDS.contains(&kind) {
                read_vault_ask
            } else {
                read_vault_reply
            };
            let mut r = Counted::new((&raw[..]).chain(io::repeat(0).take(plenty)));
            assert_eq!(read(&mut r), Err(TooLong), "{kind:?} {len}");
            assert_eq!(r.taken, HEADER_LEN);
        }
    }
}

#[test]
fn a_field_past_its_cap_is_refused_before_its_bytes() {
    let raw = [
        &head(MAGIC, VERSION, Kind::CheckPassword.code(), 1, 64)[..],
        &two(MAX_PASSWORD_BYTES + 1),
    ]
    .concat();
    let mut r = Counted::new((&raw[..]).chain(io::repeat(0)));
    assert_eq!(read_vault_ask(&mut r), Err(TooLong));
    assert_eq!(r.taken, raw.len());
}

#[test]
fn a_field_longer_than_its_frame_is_inconsistent_before_its_bytes() {
    let mut payload = check_payload(b"pw", b"x");
    payload.truncate(payload.len() - 1);
    assert_eq!(
        read_vault_ask(&mut &vault_frame(Kind::CheckPassword, &payload)[..]),
        Err(Inconsistent)
    );
    let raw = [
        &head(MAGIC, VERSION, Kind::CheckPassword.code(), 1, 8)[..],
        &two(100),
    ]
    .concat();
    let mut r = Counted::new((&raw[..]).chain(io::repeat(0)));
    assert_eq!(read_vault_ask(&mut r), Err(Inconsistent));
    assert_eq!(r.taken, raw.len());
}

#[test]
fn bytes_after_the_last_field_are_inconsistent() {
    let mut payload = check_payload(b"pw", b"locked");
    payload.push(0);
    let bytes = vault_frame(Kind::CheckPassword, &payload);
    assert_eq!(read_vault_ask(&mut &bytes[..]), Err(Inconsistent));
    let padlock = vault_frame(Kind::Padlock, b"\x01r\x01\0l\0");
    assert_eq!(read_vault_reply(&mut &padlock[..]), Err(Inconsistent));
}

#[test]
fn an_empty_box_or_padlock_is_inconsistent() {
    let open = [&two(2)[..], b"pw", &[0, 0, 0, 0]].concat();
    let bytes = vault_frame(Kind::OpenBox, &open);
    assert_eq!(read_vault_ask(&mut &bytes[..]), Err(Inconsistent));
    let check = check_payload(b"pw", b"");
    let bytes = vault_frame(Kind::CheckPassword, &check);
    assert_eq!(read_vault_ask(&mut &bytes[..]), Err(Inconsistent));
}

/// A seal of `parts`, each a raw name and its bytes, under a small padlock.
fn seal_payload(count: usize, parts: &[(&[u8], &[u8])]) -> Vec<u8> {
    let mut out = [&b"\x01r\x01\0l"[..], &[u8::try_from(count).unwrap()]].concat();
    for (name, bytes) in parts {
        out.push(u8::try_from(name.len()).unwrap());
        out.extend_from_slice(name);
        out.extend_from_slice(&u32::try_from(bytes.len()).unwrap().to_le_bytes());
        out.extend_from_slice(bytes);
    }
    out
}

#[test]
fn an_unknown_repeated_or_extra_part_is_refused_both_ways() {
    let note = NOTEBOOK_PART.as_bytes();
    let cases = [
        (seal_payload(1, &[(b"settings", b"x")]), BadField),
        (seal_payload(1, &[(b"Notebook", b"x")]), BadField),
        (seal_payload(2, &[(note, b"x"), (note, b"y")]), TooLong),
        (seal_payload(MAX_PARTS + 1, &[(note, b"x")]), TooLong),
        (seal_payload(1, &[(b"", b"x")]), Inconsistent),
    ];
    for (payload, err) in cases {
        let bytes = vault_frame(Kind::Seal, &payload);
        assert_eq!(read_vault_ask(&mut &bytes[..]), Err(err), "{payload:?}");
        let opened = vault_frame(Kind::Opened, &payload);
        assert_eq!(read_vault_reply(&mut &opened[..]), Err(err));
    }
    let seal = |name: &'static str| VaultAsk::Seal {
        id: 1,
        recipient: Cow::Borrowed("r"),
        locked: Cow::Borrowed(b"l"),
        parts: vec![BoxPart {
            name: Cow::Borrowed(name),
            bytes: Wiped::Lent(b"x"),
        }],
    };
    for name in ["settings", ""] {
        let mut out = Vec::new();
        assert_eq!(seal(name).write(&mut out), Err(BadField), "{name:?}");
        assert!(out.is_empty());
    }
}

#[test]
fn a_box_with_no_parts_is_refused_both_ways() {
    let none = seal_payload(0, &[]);
    let bytes = vault_frame(Kind::Seal, &none);
    assert_eq!(read_vault_ask(&mut &bytes[..]), Err(Inconsistent));
    let bytes = vault_frame(Kind::Opened, &none);
    assert_eq!(read_vault_reply(&mut &bytes[..]), Err(Inconsistent));
    let seal = VaultAsk::Seal {
        id: 1,
        recipient: Cow::Borrowed("r"),
        locked: Cow::Borrowed(b"l"),
        parts: Vec::new(),
    };
    let mut out = Vec::new();
    assert_eq!(seal.write(&mut out), Err(Inconsistent));
    assert!(out.is_empty());
    assert_eq!(opened(1, Vec::new()).check(), Err(Inconsistent));
}

#[test]
fn a_part_name_is_known_and_new_or_refused() {
    let note = NOTEBOOK_PART.as_bytes();
    assert_eq!(part_name(note, &[]), Ok(NOTEBOOK_PART));
    assert_eq!(part_name(note, &[NOTEBOOK_PART]), Err(BadField));
    assert_eq!(part_name(b"settings", &[]), Err(BadField));
    let mut out = Vec::new();
    let ask = VaultAsk::Seal {
        id: 1,
        recipient: Cow::Borrowed("r"),
        locked: Cow::Borrowed(b"l"),
        parts: vec![notebook(b"x"), notebook(b"y")],
    };
    assert_eq!(ask.write(&mut out), Err(TooLong), "more parts than names");
    assert!(out.is_empty());
}

#[test]
fn a_work_factor_outside_the_settings_range_is_refused_both_ways() {
    for work in [0, MIN_BOX_WORK - 1, MAX_BOX_WORK + 1, u8::MAX] {
        let ask = VaultAsk::Padlock {
            id: 1,
            work,
            password: Wiped::Lent(b"pw"),
        };
        let mut out = Vec::new();
        assert_eq!(ask.write(&mut out), Err(BadField), "{work}");
        assert!(out.is_empty());
        let payload = [&[work][..], &two(2), b"pw"].concat();
        let bytes = vault_frame(Kind::MakePadlock, &payload);
        assert_eq!(read_vault_ask(&mut &bytes[..]), Err(BadField), "{work}");
    }
}

#[test]
fn text_that_is_not_utf8_is_refused_both_ways() {
    let bad = b"\xff\xfe";
    let ask = VaultAsk::Check {
        id: 1,
        password: Wiped::Lent(bad),
        locked: Cow::Borrowed(b"l"),
    };
    assert_eq!(ask.write(&mut Vec::new()), Err(BadField));
    let bytes = vault_frame(Kind::CheckPassword, &check_payload(bad, b"l"));
    assert_eq!(read_vault_ask(&mut &bytes[..]), Err(BadField));
    let padlock = [&b"\x02"[..], bad, b"\x01\0l"].concat();
    let bytes = vault_frame(Kind::Padlock, &padlock);
    assert_eq!(read_vault_reply(&mut &bytes[..]), Err(BadField));
    let entry = [&NONE_DROPPED[..], b"\x01\0\x02\0", bad, &[0; 9]].concat();
    let bytes = vault_frame(Kind::Entries, &entry);
    assert_eq!(read_vault_reply(&mut &bytes[..]), Err(BadField));
}

#[test]
fn a_yes_no_or_a_reason_out_of_range_is_refused() {
    for byte in [2, u8::MAX] {
        let bytes = vault_frame(Kind::Checked, &[byte]);
        assert_eq!(read_vault_reply(&mut &bytes[..]), Err(BadField));
    }
    let next = u8::try_from(Fault::ALL.len() + 1).unwrap();
    for byte in [0, next, u8::MAX] {
        let bytes = vault_frame(Kind::VaultFailed, &[byte]);
        assert_eq!(read_vault_reply(&mut &bytes[..]), Err(BadField));
    }
}

#[test]
fn a_writer_refuses_an_empty_or_oversized_box_and_writes_nothing() {
    let big = vec![0; MAX_BOX_BYTES + 1];
    let long = "x".repeat(MAX_PASSWORD_BYTES + 1);
    let asks = [
        (
            VaultAsk::Open {
                id: 1,
                password: Wiped::Lent(b"pw"),
                bytes: Cow::Borrowed(b""),
            },
            Inconsistent,
        ),
        (
            VaultAsk::Keepass {
                id: 1,
                password: Wiped::Lent(b"pw"),
                bytes: Cow::Borrowed(&big),
            },
            TooLong,
        ),
        (
            VaultAsk::Open {
                id: 1,
                password: Wiped::Lent(long.as_bytes()),
                bytes: Cow::Borrowed(b"b"),
            },
            TooLong,
        ),
    ];
    for (ask, err) in asks {
        let mut out = Vec::new();
        assert_eq!(ask.write(&mut out), Err(err), "{ask:?}");
        assert!(out.is_empty());
    }
    let reply = VaultReply::Sealed { id: 1, bytes: big };
    assert_eq!(reply.write(&mut Vec::new()), Err(TooLong));
}

#[test]
fn a_goodbye_with_an_id_is_inconsistent() {
    let bytes = frame(Kind::VaultBye, 5, &[]);
    assert_eq!(read_vault_ask(&mut &bytes[..]), Err(Inconsistent));
}
