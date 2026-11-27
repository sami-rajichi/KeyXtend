//! The pick frames, both directions: the wire layout, round trips in Arabic too, the largest frames, ids, each job
//! reading only its own kinds, cut frames, and `Debug` that shows no path or name. Refusals, properties and the answer
//! check have their own files.

mod answer;
mod props;
mod refuse;

use super::*;
use crate::pick::{ask as pick_ask, reply as pick_reply, wire as pick_wire};
use crate::{
    FetchAsk, FetchNote, FileKind, MAX_FILE_NAME_BYTES, MAX_PATH_BYTES, MP4_EXT, NetAsk, NetNote,
    PNG_EXT, PickAsk, PickReply, RecordAsk, RecordNote, VaultAsk, VaultReply, VoiceAsk, VoiceNote,
};

/// The folder a test dialog opens at.
const SNIPS: &str = r"D:\KeyXtend\snips";
/// A folder of Arabic names, "my snips".
const ARABIC: &str = r"D:\المستندات\لقطاتي";
/// The name a test dialog proposes, "snip" and its date.
const NAME: &str = "لقطة 2026-11-24 10-30-05.png";
/// The file a test dialog answers.
const CHOSEN: &str = r"D:\المستندات\لقطاتي\لقطة الشاشة.png";
/// A system code a failed dialog tells: `E_ACCESSDENIED`.
const DENIED: u32 = 0x8007_0005;

fn pick_ask_bytes(ask: &PickAsk) -> Vec<u8> {
    let mut out = Vec::new();
    ask.write(&mut out).unwrap();
    out
}

fn pick_reply_bytes(reply: &PickReply) -> Vec<u8> {
    let mut out = Vec::new();
    reply.write(&mut out).unwrap();
    out
}

fn read_pick_ask(r: &mut dyn Read) -> Result<(), IpcError> {
    PickAsk::read(r).map(drop)
}

fn read_pick_reply(r: &mut dyn Read) -> Result<(), IpcError> {
    PickReply::read(r).map(drop)
}

/// A Save dialog at `folder` proposing `name` as `kind`.
fn save<'a>(folder: &'a str, name: &'a str, kind: FileKind) -> PickAsk<'a> {
    PickAsk::Save {
        id: 1,
        folder: Cow::Borrowed(folder),
        name: Cow::Borrowed(name),
        kind,
    }
}

/// A folder dialog at `at`.
fn folder(at: &str) -> PickAsk<'_> {
    PickAsk::Folder {
        id: 2,
        at: Cow::Borrowed(at),
    }
}

/// The answer `path`.
fn picked(path: &str) -> PickReply<'_> {
    PickReply::Picked {
        id: 1,
        path: Cow::Borrowed(path),
    }
}

fn every_pick_ask() -> Vec<PickAsk<'static>> {
    vec![
        save(SNIPS, "snip.mp4", FileKind::Mp4),
        save(ARABIC, NAME, FileKind::Png),
        folder(ARABIC),
        PickAsk::Bye,
    ]
}

fn every_pick_reply() -> Vec<PickReply<'static>> {
    vec![
        picked(CHOSEN),
        picked(SNIPS),
        PickReply::Cancelled { id: 3 },
        PickReply::Refused { id: 4 },
        PickReply::Failed {
            id: u32::MAX,
            code: DENIED,
        },
        PickReply::Failed {
            id: 5,
            code: PickReply::NO_CODE,
        },
    ]
}

#[test]
fn a_save_and_its_answer_are_little_endian_after_their_kinds() {
    let ask = save(r"D:\a", "b", FileKind::Png);
    let want = b"KXIP\x01\x46\x01\0\0\0\x0a\0\0\0\x01\x04\0D:\\a\x01\0b";
    assert_eq!(pick_ask_bytes(&ask), want);
    let reply = picked(r"D:\c.mp4");
    let want = b"KXIP\x01\x49\x01\0\0\0\x0a\0\0\0\x08\0D:\\c.mp4";
    assert_eq!(pick_reply_bytes(&reply), want);
    let none = pick_reply_bytes(&PickReply::Cancelled { id: 2 });
    assert_eq!(none, b"KXIP\x01\x4a\x02\0\0\0\0\0\0\0");
    let refused = pick_reply_bytes(&PickReply::Refused { id: 3 });
    assert_eq!(refused, b"KXIP\x01\x4e\x03\0\0\0\0\0\0\0");
    let failed = pick_reply_bytes(&PickReply::Failed {
        id: 4,
        code: DENIED,
    });
    assert_eq!(failed, b"KXIP\x01\x4b\x04\0\0\0\x04\0\0\0\x05\0\x07\x80");
}

#[test]
fn each_pick_frame_round_trips() {
    for ask in every_pick_ask() {
        assert_eq!(PickAsk::read(&mut &pick_ask_bytes(&ask)[..]), Ok(ask));
    }
    for reply in every_pick_reply() {
        let bytes = pick_reply_bytes(&reply);
        assert_eq!(PickReply::read(&mut &bytes[..]), Ok(reply));
    }
}

#[test]
fn the_largest_save_and_answer_fill_their_caps_and_round_trip() {
    let path = "p".repeat(MAX_PATH_BYTES);
    let mut name = "ل".repeat(MAX_FILE_NAME_BYTES / "ل".len());
    name.extend(std::iter::repeat_n('a', MAX_FILE_NAME_BYTES - name.len()));
    assert_eq!(name.len(), MAX_FILE_NAME_BYTES);
    let ask = save(&path, &name, FileKind::Mp4);
    let bytes = pick_ask_bytes(&ask);
    assert_eq!(bytes.len(), HEADER_LEN + pick_wire::SAVE_MOST);
    assert_eq!(PickAsk::read(&mut &bytes[..]), Ok(ask));
    let bytes = pick_reply_bytes(&picked(&path));
    assert_eq!(bytes.len(), HEADER_LEN + pick_wire::PATH.most());
    assert_eq!(PickReply::read(&mut &bytes[..]), Ok(picked(&path)));
}

#[test]
fn a_file_kind_names_its_extension() {
    assert_eq!(FileKind::Png.ext(), PNG_EXT);
    assert_eq!(FileKind::Mp4.ext(), MP4_EXT);
    let codes: Vec<u8> = FileKind::ALL.iter().map(|k| k.code()).collect();
    assert_eq!(codes, [1, 2]);
}

#[test]
fn every_frame_names_its_dialog() {
    let ids: Vec<u32> = every_pick_ask().iter().map(PickAsk::id).collect();
    assert_eq!(ids, [1, 1, 2, 0]);
    let ids: Vec<u32> = every_pick_reply().iter().map(PickReply::id).collect();
    assert_eq!(ids, [1, 1, 3, 4, u32::MAX, 5]);
}

#[test]
fn debug_shows_no_path_or_name() {
    let said = format!("{:?}{:?}", every_pick_ask(), every_pick_reply());
    for private in ["KeyXtend", "لقط", "snip.mp4"] {
        assert!(!said.contains(private), "{said}");
    }
    let ask = format!("{:?}", save(SNIPS, NAME, FileKind::Png));
    assert_eq!(ask, "Save { id: 1, kind: Png, .. }");
    assert_eq!(format!("{:?}", picked(CHOSEN)), "Picked { id: 1, .. }");
    let failed = PickReply::Failed {
        id: 6,
        code: DENIED,
    };
    assert_eq!(format!("{failed:?}"), "Failed { id: 6, code: 0x80070005 }");
    assert_eq!(
        format!("{:?}", PickReply::Refused { id: 7 }),
        "Refused { id: 7 }"
    );
}

#[test]
fn a_cut_pick_frame_is_short_and_an_empty_stream_closed() {
    let asks = every_pick_ask();
    let asks = asks
        .iter()
        .map(|a| (pick_ask_bytes(a), read_pick_ask as Reader));
    let replies = every_pick_reply();
    let replies = replies
        .iter()
        .map(|r| (pick_reply_bytes(r), read_pick_reply as Reader));
    for (bytes, read) in asks.chain(replies) {
        assert_eq!(read(&mut &bytes[..0]), Err(IpcError::Closed));
        for cut in 1..bytes.len() {
            let got = read(&mut &bytes[..cut]);
            assert_eq!(got, Err(IpcError::Short), "cut at {cut}");
        }
    }
}

#[test]
fn each_pick_frame_is_refused_by_every_other_reader() {
    let others: [Reader; 12] = [
        read_ask,
        read_reply,
        |r| VaultAsk::read(r).map(drop),
        |r| VaultReply::read(r).map(drop),
        |r| VoiceAsk::read(r).map(drop),
        |r| VoiceNote::read(r).map(drop),
        |r| FetchAsk::read(r).map(drop),
        |r| FetchNote::read(r).map(drop),
        |r| NetAsk::read(r).map(drop),
        |r| NetNote::read(r).map(drop),
        |r| RecordAsk::read(r).map(drop),
        |r| RecordNote::read(r).map(drop),
    ];
    let asks: Vec<_> = every_pick_ask().iter().map(pick_ask_bytes).collect();
    let replies: Vec<_> = every_pick_reply().iter().map(pick_reply_bytes).collect();
    for bytes in asks.iter().chain(&replies) {
        for read in others {
            assert_eq!(read(&mut &bytes[..]), Err(IpcError::BadKind));
        }
    }
    for bytes in &asks {
        assert_eq!(read_pick_reply(&mut &bytes[..]), Err(IpcError::BadKind));
    }
    for bytes in &replies {
        assert_eq!(read_pick_ask(&mut &bytes[..]), Err(IpcError::BadKind));
    }
}

#[test]
fn pick_kinds_have_their_lengths() {
    for &kind in pick_ask::KINDS.iter().chain(pick_reply::KINDS) {
        let lens = kind.lens();
        assert!(lens.start() <= lens.end(), "{kind:?}");
    }
    assert_eq!(Kind::Picked.lens().end(), &pick_wire::PATH.most());
    let code = size_of::<u32>();
    assert_eq!(Kind::PickFailed.lens(), code..=code, "the system's code");
    assert_eq!(Kind::PickRefused.lens(), 0..=0);
}
