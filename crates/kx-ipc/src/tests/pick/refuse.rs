//! Each pick refusal, writing and reading: a length past its kind, a path or name past its limit, a path with a
//! control character or a device name, a name that is no plain file name, an unknown kind, and a goodbye with an id.

use super::*;
use IpcError::{BadField, Inconsistent, TooLong};

/// Paths no dialog may open at or answer: a control character, or a part that names a device.
const BAD_PATHS: [&str; 7] = [
    "D:\\a\nb",
    "D:\\a\0b",
    "D:\\a\u{9b}b",
    r"D:\con",
    r"D:\a\LPT1.txt",
    r"D:/a/Aux .png",
    r"\\.\COM³",
];

/// Names no Save dialog may propose: a separator, a mark Windows refuses, a control character, or a device name.
const BAD_NAMES: [&str; 12] = [
    r"a\b.png", "a/b.png", "a:b.png", "a*.png", "a?.png", "a\".png", "a<.png", "a>.png", "a|.png",
    "a\tb.png", "con.png", "nul",
];

/// Checks that writing `ask` is refused with `err` and writes nothing.
fn refused(ask: &PickAsk, err: IpcError) {
    let mut out = Vec::new();
    assert_eq!(ask.write(&mut out), Err(err), "{ask:?}");
    assert!(out.is_empty());
}

/// Checks that writing `reply` is refused with `err` and writes nothing.
fn refused_reply(reply: &PickReply, err: IpcError) {
    let mut out = Vec::new();
    assert_eq!(reply.write(&mut out), Err(err), "{reply:?}");
    assert!(out.is_empty());
}

/// A text field: its two-byte length, then its bytes.
fn field(text: &[u8]) -> Vec<u8> {
    let len = u16::try_from(text.len()).unwrap().to_le_bytes();
    [&len[..], text].concat()
}

#[test]
fn a_length_past_its_kind_is_refused_before_the_payload() {
    let plenty = u64::try_from(READ_CHUNK).unwrap();
    for &kind in pick_ask::KINDS.iter().chain(pick_reply::KINDS) {
        let over = u32::try_from(*kind.lens().end() + 1).unwrap();
        let read: Reader = if pick_ask::KINDS.contains(&kind) {
            read_pick_ask
        } else {
            read_pick_reply
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
fn a_path_or_name_past_its_limit_or_empty_is_refused_both_ways() {
    let long = "p".repeat(MAX_PATH_BYTES + 1);
    refused(&save(&long, NAME, FileKind::Png), TooLong);
    refused(&folder(&long), TooLong);
    refused_reply(&picked(&long), TooLong);
    let name = "n".repeat(MAX_FILE_NAME_BYTES + 1);
    refused(&save(SNIPS, &name, FileKind::Png), TooLong);
    refused(&save("", NAME, FileKind::Png), Inconsistent);
    refused(&save(SNIPS, "", FileKind::Png), Inconsistent);
    refused(&folder(""), Inconsistent);
    refused_reply(&picked(""), Inconsistent);
    let over = u16::try_from(MAX_PATH_BYTES + 1).unwrap().to_le_bytes();
    let bytes = frame(Kind::Picked, 1, &[&over[..], &b"p"[..]].concat());
    assert_eq!(read_pick_reply(&mut &bytes[..]), Err(TooLong));
    let empty = frame(Kind::Picked, 1, &field(b""));
    assert_eq!(read_pick_reply(&mut &empty[..]), Err(Inconsistent));
}

#[test]
fn a_path_with_a_control_character_or_a_device_name_is_refused_both_ways() {
    for path in BAD_PATHS {
        refused(&save(path, NAME, FileKind::Png), BadField);
        refused(&folder(path), BadField);
        refused_reply(&picked(path), BadField);
        let bytes = frame(Kind::Picked, 1, &field(path.as_bytes()));
        assert_eq!(read_pick_reply(&mut &bytes[..]), Err(BadField), "{path:?}");
        let bytes = frame(Kind::PickFolder, 1, &field(path.as_bytes()));
        assert_eq!(read_pick_ask(&mut &bytes[..]), Err(BadField), "{path:?}");
    }
    let lookalikes = [
        r"D:\con-1",
        r"D:\lpt10\a.png",
        r"D:\a\nul1.png",
        r"D:\console",
    ];
    for path in lookalikes {
        let bytes = pick_reply_bytes(&picked(path));
        assert_eq!(PickReply::read(&mut &bytes[..]), Ok(picked(path)));
    }
    let not_text = frame(Kind::Picked, 1, &field(b"D:\\\xff"));
    assert_eq!(read_pick_reply(&mut &not_text[..]), Err(BadField));
}

#[test]
fn a_name_that_is_no_plain_file_name_is_refused_both_ways() {
    for name in BAD_NAMES {
        refused(&save(SNIPS, name, FileKind::Png), BadField);
        let payload = [
            &[FileKind::Png.code()][..],
            &field(SNIPS.as_bytes()),
            &field(name.as_bytes()),
        ]
        .concat();
        let bytes = frame(Kind::PickSave, 1, &payload);
        assert_eq!(read_pick_ask(&mut &bytes[..]), Err(BadField), "{name:?}");
    }
}

#[test]
fn an_unknown_kind_or_a_bare_frame_with_a_payload_or_an_id_is_refused() {
    let next = u8::try_from(FileKind::ALL.len() + 1).unwrap();
    for byte in [0, next, u8::MAX] {
        let payload = [&[byte][..], &field(SNIPS.as_bytes()), &field(b"a.png")].concat();
        let bytes = frame(Kind::PickSave, 1, &payload);
        assert_eq!(read_pick_ask(&mut &bytes[..]), Err(BadField), "{byte}");
    }
    let bye = frame(Kind::PickBye, 5, &[]);
    assert_eq!(read_pick_ask(&mut &bye[..]), Err(Inconsistent));
    for kind in [Kind::PickCancelled, Kind::PickRefused] {
        let bytes = frame(kind, 1, &[0]);
        assert_eq!(read_pick_reply(&mut &bytes[..]), Err(TooLong), "{kind:?}");
    }
    let short = frame(Kind::PickFailed, 1, &[0; 3]);
    assert_eq!(read_pick_reply(&mut &short[..]), Err(Inconsistent));
    let long = frame(Kind::PickFailed, 1, &[0; 5]);
    assert_eq!(read_pick_reply(&mut &long[..]), Err(TooLong));
}

#[test]
fn a_wrong_version_or_a_length_off_by_one_is_refused_for_every_pick_frame() {
    let asks = every_pick_ask();
    let asks = asks
        .iter()
        .map(|a| (pick_ask_bytes(a), read_pick_ask as Reader));
    let replies = every_pick_reply();
    let replies = replies
        .iter()
        .map(|r| (pick_reply_bytes(r), read_pick_reply as Reader));
    for (bytes, read) in asks.chain(replies) {
        let mut old = bytes.clone();
        old[MAGIC.len()] = VERSION + 1;
        assert_eq!(read(&mut &old[..]), Err(IpcError::BadVersion));
        let mut long = bytes.clone();
        long[HEADER_LEN - 4] = long[HEADER_LEN - 4].wrapping_add(1);
        long.push(0);
        let got = read(&mut &long[..]);
        assert!(matches!(got, Err(Inconsistent | TooLong)), "{got:?}");
    }
}
