//! Record properties: what is written reads back, and any bytes are refused cleanly, within the frame and the caps.

use super::*;
use crate::tests::props::check_bounded;
use crate::{SNIP_MAX_MIN, SNIP_MIN_FREE_MB};
use proptest::prelude::*;
use proptest::sample::Index;

/// The longest junk a case reads.
const MAX_JUNK_BYTES: usize = 256;
/// How far around each cap a case's length lands.
const NEAR_CAP: u32 = 2;
/// A plain video's path: a drive, folders and a name that never names a device, with the video's extension.
const VIDEO_PATH: &str = "[A-Z]:(\\\\snip [^\\x00-\\x1f\\x7f-\\x9f\\\\/]{0,8}){1,3}\\.[mM][pP]4";

fn a_path() -> impl Strategy<Value = Cow<'static, str>> {
    VIDEO_PATH.prop_map(Cow::Owned)
}

fn an_area() -> impl Strategy<Value = Area> {
    (any::<i32>(), any::<i32>(), 1..=i32::MAX, 1..=i32::MAX).prop_map(|(x, y, w, h)| Area {
        x: x.min(i32::MAX - w),
        y: y.min(i32::MAX - h),
        w,
        h,
    })
}

fn a_start() -> impl Strategy<Value = RecordAsk<'static>> {
    let fps = prop::sample::select(SNIP_FPS.to_vec());
    let numbers = (SNIP_MAX_MIN, SNIP_MIN_FREE_MB);
    let switches = (any::<bool>(), any::<bool>());
    (any::<u32>(), an_area(), fps, switches, numbers, a_path()).prop_map(
        |(id, area, fps, (mic, sound), (max_min, min_free_mb), file)| RecordAsk::Start {
            id,
            area,
            fps,
            mic,
            sound,
            max_min,
            min_free_mb,
            file,
        },
    )
}

fn a_record_ask() -> impl Strategy<Value = RecordAsk<'static>> {
    let id = any::<u32>;
    let speed = prop::sample::select(SNIP_SPEEDS.to_vec());
    let copy =
        (id(), a_path(), speed, a_path()).prop_map(|(id, from, speed, to)| RecordAsk::Speed {
            id,
            from,
            speed,
            to,
        });
    prop_oneof![
        a_start(),
        (id(), any::<bool>()).prop_map(|(id, on)| RecordAsk::Mic { id, on }),
        (id(), any::<bool>()).prop_map(|(id, on)| RecordAsk::Sound { id, on }),
        id().prop_map(|id| RecordAsk::Stop { id }),
        id().prop_map(|id| RecordAsk::Cancel { id }),
        copy,
        Just(RecordAsk::Bye),
    ]
}

fn a_record_note() -> impl Strategy<Value = RecordNote> {
    let id = any::<u32>;
    let ms = || 0..=MAX_VIDEO_MS;
    let end = prop::sample::select(RecordEnd::ALL);
    let fault = prop::sample::select(RecordFault::ALL);
    let switch = prop::sample::select(Switch::ALL);
    let ack = (id(), switch, any::<bool>()).prop_map(|(id, switch, on)| RecordNote::Switched {
        id,
        switch,
        on,
    });
    prop_oneof![
        (id(), ms()).prop_map(|(id, ms)| RecordNote::Time { id, ms }),
        (id(), ms(), end).prop_map(|(id, length_ms, end)| RecordNote::End { id, length_ms, end }),
        (id(), fault).prop_map(|(id, fault)| RecordNote::Fault { id, fault }),
        ack,
        (id(), prop::sample::select(Switch::ALL))
            .prop_map(|(id, switch)| RecordNote::Lost { id, switch }),
    ]
}

/// Payload lengths: small ones, ones around the largest record ask's cap, and any at all.
fn a_len() -> impl Strategy<Value = u32> {
    let cap = u32::try_from(record_wire::SPEED_MOST).unwrap();
    prop_oneof![0..64u32, cap - NEAR_CAP..=cap + NEAR_CAP, any::<u32>()]
}

proptest! {
    #[test]
    fn record_asks_round_trip_and_end_at_their_frame(ask in a_record_ask()) {
        let bytes = record_ask_bytes(&ask);
        let mut r = Counted::new(&bytes[..]);
        prop_assert_eq!(RecordAsk::read(&mut r), Ok(ask));
        prop_assert_eq!(r.taken, bytes.len());
    }

    #[test]
    fn record_notes_round_trip_and_end_at_their_frame(note in a_record_note()) {
        let bytes = record_note_bytes(&note);
        let mut r = Counted::new(&bytes[..]);
        prop_assert_eq!(RecordNote::read(&mut r), Ok(note));
        prop_assert_eq!(r.taken, bytes.len());
    }

    #[test]
    fn random_bytes_never_panic_a_record_reader(junk in prop::collection::vec(any::<u8>(), 0..MAX_JUNK_BYTES)) {
        for read in [read_record_ask as Reader, read_record_note] {
            let mut r = &junk[..];
            while read(&mut r).is_ok() {}
        }
    }

    #[test]
    fn junk_under_a_good_record_header_never_panics(
        kind in prop::sample::select(record_ask::KINDS.iter().chain(record_note::KINDS).copied().collect::<Vec<_>>()),
        junk in prop::collection::vec(any::<u8>(), 0..MAX_JUNK_BYTES),
    ) {
        let bytes = frame(kind, 1, &junk);
        let _ = read_record_ask(&mut &bytes[..]);
        let _ = read_record_note(&mut &bytes[..]);
    }

    #[test]
    fn a_good_record_header_is_never_read_past_its_frame_or_cap(
        kind in any::<u8>(),
        len in a_len(),
        tail in prop::collection::vec(any::<u8>(), 0..MAX_JUNK_BYTES),
    ) {
        check_bounded(kind, len, &tail, read_record_ask, record_ask::KINDS)?;
        check_bounded(kind, len, &tail, read_record_note, record_note::KINDS)?;
    }

    #[test]
    fn a_changed_record_frame_never_panics_and_what_reads_writes_back_the_same(
        ask in a_record_ask(),
        note in a_record_note(),
        pick in any::<Index>(),
        flip in 1..=u8::MAX,
    ) {
        let mut bytes = record_ask_bytes(&ask);
        let at = pick.index(bytes.len());
        bytes[at] ^= flip;
        let mut r = Counted::new(&bytes[..]);
        if let Ok(read) = RecordAsk::read(&mut r) {
            prop_assert_eq!(record_ask_bytes(&read), &bytes[..r.taken]);
        }
        let mut bytes = record_note_bytes(&note);
        let at = pick.index(bytes.len());
        bytes[at] ^= flip;
        let mut r = Counted::new(&bytes[..]);
        if let Ok(read) = RecordNote::read(&mut r) {
            prop_assert_eq!(record_note_bytes(&read), &bytes[..r.taken]);
        }
    }

    #[test]
    fn a_cut_record_frame_fails_cleanly(ask in a_record_ask(), note in a_record_note(), pick in any::<Index>()) {
        let cases: [(Vec<u8>, Reader); 2] =
            [(record_ask_bytes(&ask), read_record_ask), (record_note_bytes(&note), read_record_note)];
        for (bytes, read) in cases {
            let cut = pick.index(bytes.len());
            let want = if cut == 0 { IpcError::Closed } else { IpcError::Short };
            prop_assert_eq!(read(&mut &bytes[..cut]), Err(want));
        }
    }
}
