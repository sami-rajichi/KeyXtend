//! Each record refusal, writing and reading: a length past its kind, an empty or overflowing part, a rate, length,
//! floor or speed out of range, a path past its limit, with a control character or a device name or of no video, a time
//! past the longest video, and a code or switch out of range.

use super::*;
use crate::{SNIP_MAX_MIN, SNIP_MIN_FREE_MB};
use IpcError::{BadField, Inconsistent, TooLong};

/// Checks that writing `ask` is refused with `err` and writes nothing.
fn refused(ask: &RecordAsk, err: IpcError) {
    let mut out = Vec::new();
    assert_eq!(ask.write(&mut out), Err(err), "{ask:?}");
    assert!(out.is_empty());
}

/// A start of `area` with these numbers into the test video.
fn numbers(area: Area, fps: u8, max_min: u16, min_free_mb: u32) -> RecordAsk<'static> {
    RecordAsk::Start {
        id: 1,
        area,
        fps,
        mic: false,
        sound: false,
        max_min,
        min_free_mb,
        file: Cow::Borrowed(VIDEO),
    }
}

/// `start`'s bytes with `value` written over its payload at `at`.
fn changed(at: usize, value: &[u8]) -> Vec<u8> {
    let mut bytes = record_ask_bytes(&start(VIDEO));
    let at = HEADER_LEN + at;
    bytes[at..at + value.len()].copy_from_slice(value);
    bytes
}

/// Where a start's rate lies in its payload, after the part's four edges; its two switches follow.
const FPS_AT: usize = 16;
/// Where a start's longest length lies, after the rate and the switches.
const MAX_MIN_AT: usize = FPS_AT + 3;
/// Where a start's free-space floor lies, after the longest length.
const MIN_FREE_AT: usize = MAX_MIN_AT + 2;

#[test]
fn a_length_past_its_kind_is_refused_before_the_payload() {
    let plenty = u64::try_from(READ_CHUNK).unwrap();
    for &kind in record_ask::KINDS.iter().chain(record_note::KINDS) {
        let over = u32::try_from(*kind.lens().end() + 1).unwrap();
        let read: Reader = if record_ask::KINDS.contains(&kind) {
            read_record_ask
        } else {
            read_record_note
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
fn an_empty_or_overflowing_part_is_refused_both_ways() {
    let part = |x, y, w, h| Area { x, y, w, h };
    let bad = [
        part(0, 0, 0, 1),
        part(0, 0, 1, 0),
        part(0, 0, -1, 1),
        part(i32::MAX, 0, 1, 1),
        part(0, i32::MAX, 1, 1),
    ];
    let free = *SNIP_MIN_FREE_MB.start();
    for area in bad {
        refused(&numbers(area, SNIP_FPS[0], 1, free), BadField);
        let edges = [area.x, area.y, area.w, area.h]
            .map(i32::to_le_bytes)
            .concat();
        let bytes = changed(0, &edges);
        assert_eq!(read_record_ask(&mut &bytes[..]), Err(BadField), "{area:?}");
    }
    let edge = part(i32::MAX - 1, i32::MIN, 1, 1);
    let ask = numbers(edge, SNIP_FPS[0], 1, free);
    assert_eq!(RecordAsk::read(&mut &record_ask_bytes(&ask)[..]), Ok(ask));
}

#[test]
fn a_rate_length_or_floor_out_of_range_is_refused_both_ways() {
    let free = *SNIP_MIN_FREE_MB.start();
    let longest = *SNIP_MAX_MIN.end();
    let cases = [
        numbers(area(), 0, 1, free),
        numbers(area(), 20, 1, free),
        numbers(area(), 60, 1, free),
        numbers(area(), SNIP_FPS[0], 0, free),
        numbers(area(), SNIP_FPS[0], longest + 1, free),
        numbers(area(), SNIP_FPS[0], 1, free - 1),
        numbers(area(), SNIP_FPS[0], 1, *SNIP_MIN_FREE_MB.end() + 1),
    ];
    for ask in cases {
        refused(&ask, BadField);
    }
    let reads = [
        (FPS_AT, &[20][..]),
        (MAX_MIN_AT, &(longest + 1).to_le_bytes()),
        (MIN_FREE_AT, &(free - 1).to_le_bytes()),
    ];
    for (at, value) in reads {
        let bytes = changed(at, value);
        assert_eq!(read_record_ask(&mut &bytes[..]), Err(BadField), "{at}");
    }
}

#[test]
fn a_switch_byte_or_a_speed_out_of_range_is_refused_both_ways() {
    for at in [FPS_AT + 1, FPS_AT + 2] {
        let bytes = changed(at, &[2]);
        assert_eq!(read_record_ask(&mut &bytes[..]), Err(BadField), "{at}");
    }
    let bytes = frame(Kind::RecordMic, 1, &[2]);
    assert_eq!(read_record_ask(&mut &bytes[..]), Err(BadField));
    for times in [0, 3, 8] {
        let ask = RecordAsk::Speed {
            id: 1,
            from: Cow::Borrowed(VIDEO),
            speed: times,
            to: Cow::Borrowed(FAST),
        };
        refused(&ask, BadField);
    }
    let mut bytes = record_ask_bytes(&speed(VIDEO, FAST));
    bytes[HEADER_LEN + 2 + VIDEO.len()] = 3;
    assert_eq!(read_record_ask(&mut &bytes[..]), Err(BadField));
}

#[test]
fn a_path_past_its_limit_or_not_a_plain_video_is_refused_both_ways() {
    let long = format!("{}.mp4", "p".repeat(MAX_PATH_BYTES - 3));
    refused(&start(&long), TooLong);
    refused(&speed(VIDEO, &long), TooLong);
    refused(&start(""), Inconsistent);
    let bad = [
        "D:\\snips\\a\nb.mp4",
        "D:\\snips\\a\0b.mp4",
        "D:\\snips\\a\u{85}.mp4",
        r"D:\con\a.mp4",
        r"D:\snips\NUL.mp4",
        r"D:/snips/com1.tar.mp4",
        r"\\.\COM¹\a.mp4",
        r"D:\snips\a.png",
        r"D:\snips\.mp4",
        r"D:\snips\a.mp4\b",
    ];
    for path in bad {
        refused(&start(path), BadField);
        refused(&speed(path, FAST), BadField);
        refused(&speed(VIDEO, path), BadField);
    }
    for (at, value) in [(2 + 3, b'\n'), (2 + 4, b'\0')] {
        let mut bytes = record_ask_bytes(&speed(VIDEO, FAST));
        bytes[HEADER_LEN + at] = value;
        assert_eq!(read_record_ask(&mut &bytes[..]), Err(BadField), "{at}");
    }
    let mut bytes = record_ask_bytes(&speed(r"D:\a\cab.mp4", FAST));
    let name = HEADER_LEN + 2 + r"D:\a\".len();
    bytes[name..name + 3].copy_from_slice(b"aux");
    assert_eq!(read_record_ask(&mut &bytes[..]), Err(BadField));
}

#[test]
fn a_time_or_length_past_the_longest_video_is_refused_both_ways() {
    let past = MAX_VIDEO_MS + 1;
    let notes = [
        RecordNote::Time { id: 1, ms: past },
        RecordNote::End {
            id: 1,
            length_ms: past,
            end: RecordEnd::Stopped,
        },
    ];
    for note in notes {
        let mut out = Vec::new();
        assert_eq!(note.write(&mut out), Err(BadField));
        assert!(out.is_empty());
    }
    let time = frame(Kind::RecordTime, 1, &past.to_le_bytes());
    assert_eq!(read_record_note(&mut &time[..]), Err(BadField));
    let end = [&past.to_le_bytes()[..], &[RecordEnd::Stopped.code()]].concat();
    let end = frame(Kind::Recorded, 1, &end);
    assert_eq!(read_record_note(&mut &end[..]), Err(BadField));
}

#[test]
fn a_code_out_of_range_or_a_goodbye_with_an_id_is_refused() {
    let next = |n: usize| u8::try_from(n + 1).unwrap();
    for byte in [0, next(RecordFault::ALL.len()), u8::MAX] {
        let bytes = frame(Kind::RecordFailed, 1, &[byte]);
        assert_eq!(read_record_note(&mut &bytes[..]), Err(BadField));
    }
    for byte in [0, next(RecordEnd::ALL.len()), u8::MAX] {
        let payload = [&LENGTH_MS.to_le_bytes()[..], &[byte]].concat();
        let bytes = frame(Kind::Recorded, 1, &payload);
        assert_eq!(read_record_note(&mut &bytes[..]), Err(BadField));
    }
    let mic = Switch::Mic.code();
    for payload in [
        [0, 1],
        [next(Switch::ALL.len()), 0],
        [mic, 2],
        [mic, u8::MAX],
    ] {
        let bytes = frame(Kind::RecordSwitched, 1, &payload);
        let got = read_record_note(&mut &bytes[..]);
        assert_eq!(got, Err(BadField), "{payload:?}");
    }
    for byte in [0, next(Switch::ALL.len()), u8::MAX] {
        let bytes = frame(Kind::RecordLost, 1, &[byte]);
        assert_eq!(read_record_note(&mut &bytes[..]), Err(BadField), "{byte}");
    }
    let bye = frame(Kind::RecordBye, 5, &[]);
    assert_eq!(read_record_ask(&mut &bye[..]), Err(Inconsistent));
}

#[test]
fn a_wrong_version_or_a_length_off_by_one_is_refused_for_every_record_frame() {
    let asks = every_record_ask();
    let asks = asks
        .iter()
        .map(|a| (record_ask_bytes(a), read_record_ask as Reader));
    let notes = every_record_note();
    let notes = notes
        .iter()
        .map(|n| (record_note_bytes(n), read_record_note as Reader));
    for (bytes, read) in asks.chain(notes) {
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
