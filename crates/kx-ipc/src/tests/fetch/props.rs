//! Fetch properties: what is written reads back, and any bytes are refused cleanly, within the frame and the caps.

use super::*;
use crate::tests::props::check_bounded;
use crate::{MAX_MODEL_BYTES, STALL_S};
use proptest::prelude::*;
use proptest::sample::Index;

/// The most addresses or hosts a case sends.
const MAX_CASE_ITEMS: usize = 3;
/// The longest junk a case reads.
const MAX_JUNK_BYTES: usize = 256;
/// How far around each cap a case's length lands.
const NEAR_CAP: u32 = 2;

fn a_get() -> impl Strategy<Value = FetchAsk<'static>> {
    let url = "https://[a-z0-9./?=_-]{1,32}".prop_map(Cow::Owned);
    let host = "[a-z0-9.-]{1,32}".prop_map(Cow::Owned);
    let urls = prop::collection::vec(url, 1..=MAX_CASE_ITEMS);
    let hosts = prop::collection::vec(host, 1..=MAX_CASE_ITEMS);
    let path = "[^\\x00]{1,16}".prop_map(Cow::Owned);
    let file = (1..=MAX_MODEL_BYTES, any::<[u8; SHA256_BYTES]>(), STALL_S);
    (any::<u32>(), urls, hosts, path, file).prop_map(
        |(id, urls, hosts, path, (size, sha256, stall_s))| FetchAsk::Get {
            id,
            urls,
            hosts,
            path,
            size,
            sha256,
            stall_s,
        },
    )
}

fn a_fetch_ask() -> impl Strategy<Value = FetchAsk<'static>> {
    let id = any::<u32>;
    prop_oneof![
        a_get(),
        id().prop_map(|id| FetchAsk::Pause { id }),
        id().prop_map(|id| FetchAsk::Resume { id }),
        id().prop_map(|id| FetchAsk::Cancel { id }),
        Just(FetchAsk::Bye),
    ]
}

fn a_fetch_note() -> impl Strategy<Value = FetchNote> {
    let id = any::<u32>;
    let sizes = (0..=MAX_MODEL_BYTES, 1..=MAX_MODEL_BYTES);
    let progress = (id(), sizes).prop_map(|(id, (a, b))| FetchNote::Progress {
        id,
        got: a.min(b),
        total: a.max(b),
    });
    let fault = prop::sample::select(FetchFault::ALL);
    prop_oneof![
        progress,
        id().prop_map(|id| FetchNote::Done { id }),
        (id(), fault).prop_map(|(id, fault)| FetchNote::Fault { id, fault }),
    ]
}

/// Payload lengths: small ones, ones around the download ask's cap, and any at all.
fn a_len() -> impl Strategy<Value = u32> {
    let cap = u32::try_from(*Kind::Fetch.lens().end()).unwrap();
    prop_oneof![0..64u32, cap - NEAR_CAP..=cap + NEAR_CAP, any::<u32>()]
}

proptest! {
    #[test]
    fn fetch_asks_round_trip_and_end_at_their_frame(ask in a_fetch_ask()) {
        let bytes = fetch_ask_bytes(&ask);
        let mut r = Counted::new(&bytes[..]);
        prop_assert_eq!(FetchAsk::read(&mut r), Ok(ask));
        prop_assert_eq!(r.taken, bytes.len());
    }

    #[test]
    fn fetch_notes_round_trip_and_end_at_their_frame(note in a_fetch_note()) {
        let bytes = fetch_note_bytes(&note);
        let mut r = Counted::new(&bytes[..]);
        prop_assert_eq!(FetchNote::read(&mut r), Ok(note));
        prop_assert_eq!(r.taken, bytes.len());
    }

    #[test]
    fn random_bytes_never_panic_a_fetch_reader(junk in prop::collection::vec(any::<u8>(), 0..MAX_JUNK_BYTES)) {
        for read in [read_fetch_ask as Reader, read_fetch_note] {
            let mut r = &junk[..];
            while read(&mut r).is_ok() {}
        }
    }

    #[test]
    fn a_good_fetch_header_is_never_read_past_its_frame_or_cap(
        kind in any::<u8>(),
        len in a_len(),
        tail in prop::collection::vec(any::<u8>(), 0..MAX_JUNK_BYTES),
    ) {
        check_bounded(kind, len, &tail, read_fetch_ask, fetch_ask::KINDS)?;
        check_bounded(kind, len, &tail, read_fetch_note, fetch_note::KINDS)?;
    }

    #[test]
    fn a_changed_fetch_frame_never_panics_and_what_reads_writes_back_the_same(
        ask in a_fetch_ask(),
        note in a_fetch_note(),
        pick in any::<Index>(),
        flip in 1..=u8::MAX,
    ) {
        let mut bytes = fetch_ask_bytes(&ask);
        let at = pick.index(bytes.len());
        bytes[at] ^= flip;
        let mut r = Counted::new(&bytes[..]);
        if let Ok(read) = FetchAsk::read(&mut r) {
            prop_assert_eq!(fetch_ask_bytes(&read), &bytes[..r.taken]);
        }
        let mut bytes = fetch_note_bytes(&note);
        let at = pick.index(bytes.len());
        bytes[at] ^= flip;
        let mut r = Counted::new(&bytes[..]);
        if let Ok(read) = FetchNote::read(&mut r) {
            prop_assert_eq!(fetch_note_bytes(&read), &bytes[..r.taken]);
        }
    }

    #[test]
    fn a_cut_fetch_frame_fails_cleanly(ask in a_fetch_ask(), note in a_fetch_note(), pick in any::<Index>()) {
        let cases: [(Vec<u8>, Reader); 2] =
            [(fetch_ask_bytes(&ask), read_fetch_ask), (fetch_note_bytes(&note), read_fetch_note)];
        for (bytes, read) in cases {
            let cut = pick.index(bytes.len());
            let want = if cut == 0 { IpcError::Closed } else { IpcError::Short };
            prop_assert_eq!(read(&mut &bytes[..cut]), Err(want));
        }
    }
}
