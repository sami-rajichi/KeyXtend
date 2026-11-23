//! The fetch frames, both directions: the wire layout, round trips, the largest ask, the `.part` name, each job
//! reading only its own kinds, and `Debug` that shows no path. Refusals and properties have their own files.

mod hosts;
mod props;
mod refuse;

use super::*;
use crate::fetch::{ask as fetch_ask, note as fetch_note, wire as fetch_wire};
use crate::{
    FetchAsk, FetchFault, FetchNote, HTTPS, MAX_HOST_BYTES, MAX_HOSTS, MAX_MODEL_BYTES,
    MAX_PATH_BYTES, MAX_URL_BYTES, MAX_URLS, PART_EXT, SHA256_BYTES, VoiceAsk, VoiceNote,
    part_path,
};

/// The fingerprint the tests' downloads expect.
const SHA: [u8; SHA256_BYTES] = [0xab; SHA256_BYTES];

fn fetch_ask_bytes(ask: &FetchAsk) -> Vec<u8> {
    let mut out = Vec::new();
    ask.write(&mut out).unwrap();
    out
}

fn fetch_note_bytes(note: &FetchNote) -> Vec<u8> {
    let mut out = Vec::new();
    note.write(&mut out).unwrap();
    out
}

fn read_fetch_ask(r: &mut dyn Read) -> Result<(), IpcError> {
    FetchAsk::read(r).map(drop)
}

fn read_fetch_note(r: &mut dyn Read) -> Result<(), IpcError> {
    FetchNote::read(r).map(drop)
}

/// A download of one model from two addresses, allowed on three hosts.
fn get() -> FetchAsk<'static> {
    FetchAsk::Get {
        id: 1,
        urls: vec![
            Cow::Borrowed("https://huggingface.co/handy-computer/m/resolve/abc/m.gguf"),
            Cow::Borrowed("https://mirror.example/m.gguf"),
        ],
        hosts: vec![
            Cow::Borrowed("huggingface.co"),
            Cow::Borrowed("cas-bridge.xethub.hf.co"),
            Cow::Borrowed("mirror.example"),
        ],
        path: Cow::Borrowed(r"D:\KeyXtend\models\m.gguf"),
        size: 740_000_000,
        sha256: SHA,
        stall_s: 30,
    }
}

fn every_fetch_ask() -> Vec<FetchAsk<'static>> {
    vec![
        get(),
        FetchAsk::Pause { id: 2 },
        FetchAsk::Resume { id: 3 },
        FetchAsk::Cancel { id: u32::MAX },
        FetchAsk::Bye,
    ]
}

fn every_fetch_note() -> Vec<FetchNote> {
    let mut notes = vec![
        FetchNote::Progress {
            id: 1,
            got: 0,
            total: 740_000_000,
        },
        FetchNote::Progress {
            id: 1,
            got: MAX_MODEL_BYTES,
            total: MAX_MODEL_BYTES,
        },
        FetchNote::Done { id: 2 },
    ];
    notes.extend(
        FetchFault::ALL
            .iter()
            .map(|&fault| FetchNote::Fault { id: 3, fault }),
    );
    notes
}

#[test]
fn a_download_ask_and_its_progress_are_little_endian_after_their_kinds() {
    let ask = FetchAsk::Get {
        id: 1,
        urls: vec![Cow::Borrowed("https://a/m")],
        hosts: vec![Cow::Borrowed("a")],
        path: Cow::Borrowed("p"),
        size: 7,
        sha256: [1; SHA256_BYTES],
        stall_s: 30,
    };
    let want = [
        &b"KXIP\x01\x29\x01\0\0\0\x3e\0\0\0"[..],
        b"\x01\x0b\0https://a/m\x01\x01a\x01\0p",
        b"\x07\0\0\0\0\0\0\0",
        &[1; SHA256_BYTES],
        b"\x1e\0",
    ]
    .concat();
    assert_eq!(fetch_ask_bytes(&ask), want);
    let progress = FetchNote::Progress {
        id: 1,
        got: 5,
        total: 9,
    };
    let want = [
        &b"KXIP\x01\x2e\x01\0\0\0\x10\0\0\0"[..],
        b"\x05\0\0\0\0\0\0\0\x09\0\0\0\0\0\0\0",
    ]
    .concat();
    assert_eq!(fetch_note_bytes(&progress), want);
    let done = fetch_note_bytes(&FetchNote::Done { id: 4 });
    assert_eq!(done, b"KXIP\x01\x2f\x04\0\0\0\0\0\0\0");
}

#[test]
fn each_fetch_frame_round_trips() {
    for ask in every_fetch_ask() {
        assert_eq!(FetchAsk::read(&mut &fetch_ask_bytes(&ask)[..]), Ok(ask));
    }
    for note in every_fetch_note() {
        assert_eq!(FetchNote::read(&mut &fetch_note_bytes(&note)[..]), Ok(note));
    }
}

#[test]
fn the_largest_download_ask_fills_its_cap_and_round_trips() {
    let url = format!("{HTTPS}{}", "u".repeat(MAX_URL_BYTES - HTTPS.len()));
    let longest = MAX_PATH_BYTES - PART_EXT.len();
    let ask = FetchAsk::Get {
        id: 1,
        urls: vec![Cow::Owned(url); MAX_URLS],
        hosts: vec![Cow::Owned("h".repeat(MAX_HOST_BYTES)); MAX_HOSTS],
        path: Cow::Owned("p".repeat(longest)),
        size: MAX_MODEL_BYTES,
        sha256: SHA,
        stall_s: *crate::STALL_S.end(),
    };
    let bytes = fetch_ask_bytes(&ask);
    assert_eq!(bytes.len(), HEADER_LEN + fetch_wire::GET_MOST);
    assert_eq!(FetchAsk::read(&mut &bytes[..]), Ok(ask));
}

#[test]
fn the_part_file_is_the_final_name_and_part_ext_within_the_path_cap() {
    assert_eq!(part_path(r"D:\m\a.gguf"), format!(r"D:\m\a.gguf{PART_EXT}"));
    let longest = "p".repeat(MAX_PATH_BYTES - PART_EXT.len());
    assert_eq!(part_path(&longest).len(), MAX_PATH_BYTES);
}

#[test]
fn a_note_names_its_download() {
    let ids: Vec<u32> = every_fetch_note().iter().map(FetchNote::id).collect();
    assert_eq!(ids[..3], [1, 1, 2]);
    let ids: Vec<u32> = every_fetch_ask().iter().map(FetchAsk::id).collect();
    assert_eq!(ids, [1, 2, 3, u32::MAX, 0]);
}

#[test]
fn debug_shows_no_path_or_fingerprint() {
    let said = format!("{:?}{:?}", every_fetch_ask(), every_fetch_note());
    assert!(!said.contains("models") && !said.contains("171"), "{said}");
    let pause = format!("{:?}", FetchAsk::Pause { id: 2 });
    assert_eq!(pause, "Pause { id: 2 }");
}

#[test]
fn a_cut_fetch_frame_is_short_and_an_empty_stream_closed() {
    let asks = every_fetch_ask();
    let asks = asks
        .iter()
        .map(|a| (fetch_ask_bytes(a), read_fetch_ask as Reader));
    let notes = every_fetch_note();
    let notes = notes
        .iter()
        .map(|n| (fetch_note_bytes(n), read_fetch_note as Reader));
    for (bytes, read) in asks.chain(notes) {
        assert_eq!(read(&mut &bytes[..0]), Err(IpcError::Closed));
        for cut in 1..bytes.len() {
            assert_eq!(
                read(&mut &bytes[..cut]),
                Err(IpcError::Short),
                "cut at {cut}"
            );
        }
    }
}

#[test]
fn each_fetch_frame_is_refused_by_every_other_reader() {
    let others: [Reader; 8] = [
        read_ask,
        read_reply,
        |r| crate::VaultAsk::read(r).map(drop),
        |r| crate::VaultReply::read(r).map(drop),
        |r| VoiceAsk::read(r).map(drop),
        |r| VoiceNote::read(r).map(drop),
        |r| crate::NetAsk::read(r).map(drop),
        |r| crate::NetNote::read(r).map(drop),
    ];
    let asks = every_fetch_ask()
        .iter()
        .map(fetch_ask_bytes)
        .collect::<Vec<_>>();
    let notes = every_fetch_note()
        .iter()
        .map(fetch_note_bytes)
        .collect::<Vec<_>>();
    for bytes in asks.iter().chain(&notes) {
        for read in others {
            assert_eq!(read(&mut &bytes[..]), Err(IpcError::BadKind));
        }
    }
    for bytes in &asks {
        assert_eq!(read_fetch_note(&mut &bytes[..]), Err(IpcError::BadKind));
    }
    for bytes in &notes {
        assert_eq!(read_fetch_ask(&mut &bytes[..]), Err(IpcError::BadKind));
    }
}
