//! The record frames, both directions: the wire layout, round trips, the largest asks, ids, each job reading only its
//! own kinds, cut frames, and `Debug` that shows no path. Refusals and properties have their own files.

mod props;
mod refuse;

use super::*;
use crate::record::{ask as record_ask, note as record_note, wire as record_wire};
use crate::{
    Area, FetchAsk, FetchNote, MAX_PATH_BYTES, MAX_VIDEO_MS, MP4_EXT, NetAsk, NetNote, PickAsk,
    PickReply, RecordAsk, RecordEnd, RecordFault, RecordNote, SNIP_FPS, SNIP_SPEEDS, Switch,
    VaultAsk, VaultReply, VoiceAsk, VoiceNote,
};

/// The video a test recording writes.
const VIDEO: &str = r"D:\KeyXtend\snips\unsaved\snip 1.mp4";
/// The re-timed copy a test writes.
const FAST: &str = r"D:\KeyXtend\snips\unsaved\snip 1 2x.MP4";
/// A recording's length in ms, as Task 1's 3 s files read back.
const LENGTH_MS: u32 = 3008;

fn record_ask_bytes(ask: &RecordAsk) -> Vec<u8> {
    let mut out = Vec::new();
    ask.write(&mut out).unwrap();
    out
}

fn record_note_bytes(note: &RecordNote) -> Vec<u8> {
    let mut out = Vec::new();
    note.write(&mut out).unwrap();
    out
}

fn read_record_ask(r: &mut dyn Read) -> Result<(), IpcError> {
    RecordAsk::read(r).map(drop)
}

fn read_record_note(r: &mut dyn Read) -> Result<(), IpcError> {
    RecordNote::read(r).map(drop)
}

/// A part left of the main screen, as a second screen there puts it.
fn area() -> Area {
    Area {
        x: -1920,
        y: 0,
        w: 1280,
        h: 720,
    }
}

/// A recording of `area` into `file` at the higher rate, with the microphone on.
fn start(file: &str) -> RecordAsk<'_> {
    RecordAsk::Start {
        id: 1,
        area: area(),
        fps: SNIP_FPS[1],
        mic: true,
        sound: false,
        max_min: 30,
        min_free_mb: 2048,
        file: Cow::Borrowed(file),
    }
}

/// A copy of `from` into `to` at the fastest speed.
fn speed<'a>(from: &'a str, to: &'a str) -> RecordAsk<'a> {
    RecordAsk::Speed {
        id: 3,
        from: Cow::Borrowed(from),
        speed: SNIP_SPEEDS[2],
        to: Cow::Borrowed(to),
    }
}

fn every_record_ask() -> Vec<RecordAsk<'static>> {
    vec![
        start(VIDEO),
        RecordAsk::Mic { id: 1, on: false },
        RecordAsk::Sound { id: 1, on: true },
        RecordAsk::Stop { id: 1 },
        RecordAsk::Cancel { id: 2 },
        speed(VIDEO, FAST),
        RecordAsk::Bye,
    ]
}

fn every_record_note() -> Vec<RecordNote> {
    let mut notes = vec![
        RecordNote::Time { id: 1, ms: 0 },
        RecordNote::Time {
            id: 1,
            ms: MAX_VIDEO_MS,
        },
    ];
    let end = |end| RecordNote::End {
        id: 1,
        length_ms: LENGTH_MS,
        end,
    };
    notes.extend(RecordEnd::ALL.iter().map(|&e| end(e)));
    let ack = |switch, on| RecordNote::Switched { id: 1, switch, on };
    notes.extend([ack(Switch::Mic, true), ack(Switch::Sound, false)]);
    let lost = |switch| RecordNote::Lost { id: 1, switch };
    notes.extend(Switch::ALL.iter().map(|&s| lost(s)));
    let fault = |fault| RecordNote::Fault { id: 2, fault };
    notes.extend(RecordFault::ALL.iter().map(|&f| fault(f)));
    notes
}

#[test]
fn a_start_a_time_and_an_ack_are_little_endian_after_their_kinds() {
    let ask = RecordAsk::Start {
        id: 1,
        area: Area {
            x: -1,
            y: 2,
            w: 3,
            h: 4,
        },
        fps: 15,
        mic: true,
        sound: false,
        max_min: 30,
        min_free_mb: 512,
        file: Cow::Borrowed("a.mp4"),
    };
    let want = [
        &b"KXIP\x01\x3c\x01\0\0\0\x20\0\0\0"[..],
        b"\xff\xff\xff\xff\x02\0\0\0\x03\0\0\0\x04\0\0\0",
        b"\x0f\x01\0\x1e\0\0\x02\0\0\x05\0a.mp4",
    ]
    .concat();
    assert_eq!(record_ask_bytes(&ask), want);
    let time = RecordNote::Time { id: 1, ms: 1000 };
    let want = b"KXIP\x01\x43\x01\0\0\0\x04\0\0\0\xe8\x03\0\0";
    assert_eq!(record_note_bytes(&time), want);
    let stop = record_ask_bytes(&RecordAsk::Stop { id: 4 });
    assert_eq!(stop, b"KXIP\x01\x3f\x04\0\0\0\0\0\0\0");
    let ack = record_note_bytes(&RecordNote::Switched {
        id: 1,
        switch: Switch::Sound,
        on: true,
    });
    assert_eq!(ack, b"KXIP\x01\x4c\x01\0\0\0\x02\0\0\0\x02\x01");
    let lost = record_note_bytes(&RecordNote::Lost {
        id: 1,
        switch: Switch::Sound,
    });
    assert_eq!(lost, b"KXIP\x01\x4d\x01\0\0\0\x01\0\0\0\x02");
}

#[test]
fn a_switch_is_acked_with_its_switch_and_new_state_and_no_other_ask_is() {
    let ack = |id, switch, on| Some(RecordNote::Switched { id, switch, on });
    let mic = RecordAsk::Mic { id: 4, on: true };
    assert_eq!(mic.ack(), ack(4, Switch::Mic, true));
    let sound = RecordAsk::Sound { id: 5, on: false };
    assert_eq!(sound.ack(), ack(5, Switch::Sound, false));
    let switch = |a: &RecordAsk| matches!(a, RecordAsk::Mic { .. } | RecordAsk::Sound { .. });
    for ask in every_record_ask().iter().filter(|a| !switch(a)) {
        assert_eq!(ask.ack(), None, "{ask:?}");
    }
}

#[test]
fn each_record_frame_round_trips() {
    for ask in every_record_ask() {
        assert_eq!(RecordAsk::read(&mut &record_ask_bytes(&ask)[..]), Ok(ask));
    }
    for note in every_record_note() {
        let bytes = record_note_bytes(&note);
        assert_eq!(RecordNote::read(&mut &bytes[..]), Ok(note));
    }
}

#[test]
fn the_largest_start_and_speed_fill_their_caps_and_round_trip() {
    let longest = format!(
        "{}.{MP4_EXT}",
        "p".repeat(MAX_PATH_BYTES - MP4_EXT.len() - 1)
    );
    assert_eq!(longest.len(), MAX_PATH_BYTES);
    let bytes = record_ask_bytes(&start(&longest));
    assert_eq!(bytes.len(), HEADER_LEN + record_wire::START_MOST);
    assert_eq!(RecordAsk::read(&mut &bytes[..]), Ok(start(&longest)));
    let bytes = record_ask_bytes(&speed(&longest, &longest));
    assert_eq!(bytes.len(), HEADER_LEN + record_wire::SPEED_MOST);
    assert_eq!(
        RecordAsk::read(&mut &bytes[..]),
        Ok(speed(&longest, &longest))
    );
}

#[test]
fn every_frame_names_its_run() {
    let ids: Vec<u32> = every_record_ask().iter().map(RecordAsk::id).collect();
    assert_eq!(ids, [1, 1, 1, 1, 2, 3, 0]);
    let ids: Vec<u32> = every_record_note().iter().map(RecordNote::id).collect();
    assert_eq!(ids[..3], [1, 1, 1]);
    assert_eq!(ids.last(), Some(&2));
}

#[test]
fn the_longest_video_is_the_top_of_max_min_in_ms() {
    let top = u32::from(*crate::SNIP_MAX_MIN.end());
    assert_eq!(MAX_VIDEO_MS, top * 60 * crate::MS_PER_S);
}

#[test]
fn debug_shows_no_path() {
    let said = format!("{:?}{:?}", every_record_ask(), every_record_note());
    assert!(
        !said.contains("unsaved") && !said.contains("snip 1"),
        "{said}"
    );
    let mic = format!("{:?}", RecordAsk::Mic { id: 2, on: true });
    assert_eq!(mic, "Mic { id: 2, on: true }");
    let ack = RecordNote::Switched {
        id: 2,
        switch: Switch::Mic,
        on: false,
    };
    let ack = format!("{ack:?}");
    assert_eq!(ack, "Switched { id: 2, switch: Mic, on: false }");
    let lost = RecordNote::Lost {
        id: 2,
        switch: Switch::Sound,
    };
    assert_eq!(format!("{lost:?}"), "Lost { id: 2, switch: Sound }");
    let said = format!("{:?}", start(VIDEO));
    assert!(
        said.contains("fps: 30") && said.contains("w: 1280"),
        "{said}"
    );
}

#[test]
fn a_cut_record_frame_is_short_and_an_empty_stream_closed() {
    let asks = every_record_ask();
    let asks = asks
        .iter()
        .map(|a| (record_ask_bytes(a), read_record_ask as Reader));
    let notes = every_record_note();
    let notes = notes
        .iter()
        .map(|n| (record_note_bytes(n), read_record_note as Reader));
    for (bytes, read) in asks.chain(notes) {
        assert_eq!(read(&mut &bytes[..0]), Err(IpcError::Closed));
        for cut in 1..bytes.len() {
            let got = read(&mut &bytes[..cut]);
            assert_eq!(got, Err(IpcError::Short), "cut at {cut}");
        }
    }
}

#[test]
fn each_record_frame_is_refused_by_every_other_reader() {
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
        |r| PickAsk::read(r).map(drop),
        |r| PickReply::read(r).map(drop),
    ];
    let asks: Vec<_> = every_record_ask().iter().map(record_ask_bytes).collect();
    let notes: Vec<_> = every_record_note().iter().map(record_note_bytes).collect();
    for bytes in asks.iter().chain(&notes) {
        for read in others {
            assert_eq!(read(&mut &bytes[..]), Err(IpcError::BadKind));
        }
    }
    for bytes in &asks {
        assert_eq!(read_record_note(&mut &bytes[..]), Err(IpcError::BadKind));
    }
    for bytes in &notes {
        assert_eq!(read_record_ask(&mut &bytes[..]), Err(IpcError::BadKind));
    }
}

#[test]
fn record_kinds_have_their_lengths() {
    for &kind in record_ask::KINDS.iter().chain(record_note::KINDS) {
        let lens = kind.lens();
        assert!(lens.start() <= lens.end(), "{kind:?}");
    }
    assert_eq!(Kind::RecordStart.lens().end(), &record_wire::START_MOST);
    let ack = record_wire::SWITCHED_LEN;
    assert_eq!(Kind::RecordSwitched.lens(), ack..=ack);
    let lost = record_wire::LOST_LEN;
    assert_eq!(Kind::RecordLost.lens(), lost..=lost);
}
