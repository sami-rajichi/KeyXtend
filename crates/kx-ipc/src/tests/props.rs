//! Properties: what is written reads back, and any bytes are refused cleanly, within the frame and the caps.

use super::*;
use crate::{ask, reply};
use proptest::prelude::*;
use proptest::sample::Index;

/// The longest image a case draws.
const MAX_CASE_BYTES: usize = 512;
/// The longest side of a case's pixels.
const MAX_CASE_SIDE: u32 = 8;
/// The longest junk a case reads.
const MAX_JUNK_BYTES: usize = 256;
/// How far around each cap a case's length lands.
const NEAR_CAP: u32 = 2;

fn an_ask() -> impl Strategy<Value = Ask<'static>> {
    let bytes = prop::collection::vec(any::<u8>(), 1..MAX_CASE_BYTES);
    let codec = prop::sample::select(Codec::ALL);
    let draw =
        (any::<u32>(), codec, 1..=MAX_SIDE, bytes).prop_map(|(id, codec, side, bytes)| Ask::Draw {
            id,
            codec,
            side,
            bytes: Cow::Owned(bytes),
        });
    prop_oneof![draw, Just(Ask::Bye)]
}

fn a_reply() -> impl Strategy<Value = Reply> {
    let sides = (any::<u32>(), 1..=MAX_CASE_SIDE, 1..=MAX_CASE_SIDE);
    let pixels = sides.prop_flat_map(|(id, width, height)| {
        let len = width as usize * height as usize * RGBA_BYTES;
        prop::collection::vec(any::<u8>(), len).prop_map(move |rgba| Reply::Pixels {
            id,
            width,
            height,
            rgba,
        })
    });
    let why = prop::sample::select(Why::ALL);
    let failed = (any::<u32>(), why).prop_map(|(id, why)| Reply::Failed { id, why });
    prop_oneof![pixels, failed]
}

/// Payload lengths: small ones, ones around each cap, and any at all.
fn a_len() -> impl Strategy<Value = u32> {
    let near = |cap: usize| {
        let cap = u32::try_from(cap).unwrap();
        cap - NEAR_CAP..=cap + NEAR_CAP
    };
    prop_oneof![
        0..64u32,
        near(MAX_ASK_LEN),
        near(MAX_REPLY_LEN),
        any::<u32>()
    ]
}

/// Checks that a header of `kind` and `len` before `tail` is never read past its frame, and that a length past
/// the kind's cap is refused right after the header.
fn check_bounded(
    kind: u8,
    len: u32,
    tail: &[u8],
    read: Reader,
    kinds: &[Kind],
) -> Result<(), TestCaseError> {
    let bytes = [head(MAGIC, VERSION, kind, 0, len), tail.to_vec()].concat();
    let len = usize::try_from(len).unwrap();
    let mut r = Counted::new(&bytes[..]);
    let got = read(&mut r);
    prop_assert!(r.taken <= HEADER_LEN + len);
    let ours = Kind::from_code(kind).filter(|k| kinds.contains(k));
    if ours.is_some_and(|k| len > *k.lens().end()) {
        prop_assert_eq!(got, Err(IpcError::TooLong));
        prop_assert_eq!(r.taken, HEADER_LEN);
    }
    Ok(())
}

proptest! {
    #[test]
    fn asks_round_trip_and_end_at_their_frame(ask in an_ask()) {
        let bytes = ask_bytes(&ask);
        let mut r = Counted::new(&bytes[..]);
        prop_assert_eq!(Ask::read(&mut r), Ok(ask));
        prop_assert_eq!(r.taken, bytes.len());
    }

    #[test]
    fn replies_round_trip_and_end_at_their_frame(reply in a_reply()) {
        let bytes = reply_bytes(&reply);
        let mut r = Counted::new(&bytes[..]);
        prop_assert_eq!(Reply::read(&mut r), Ok(reply));
        prop_assert_eq!(r.taken, bytes.len());
    }

    #[test]
    fn random_bytes_never_panic(junk in prop::collection::vec(any::<u8>(), 0..MAX_JUNK_BYTES)) {
        for read in [read_ask as Reader, read_reply] {
            let mut r = &junk[..];
            while read(&mut r).is_ok() {}
        }
    }

    #[test]
    fn a_good_header_is_never_read_past_its_frame_or_cap(
        kind in any::<u8>(),
        len in a_len(),
        tail in prop::collection::vec(any::<u8>(), 0..MAX_JUNK_BYTES),
    ) {
        check_bounded(kind, len, &tail, read_ask, ask::KINDS)?;
        check_bounded(kind, len, &tail, read_reply, reply::KINDS)?;
    }

    #[test]
    fn a_read_buffer_grows_past_neither_its_length_nor_twice_what_arrived(
        have in 0..MAX_REPLY_LEN,
        more in 1..MAX_REPLY_LEN,
    ) {
        let len = have + more;
        let next = grow(have, len);
        prop_assert!(have < next && next <= len);
        prop_assert!(next <= (2 * have).max(READ_CHUNK));
        prop_assert!(have + next <= READ_TIMES * len, "the old and new buffers together");
    }

    #[test]
    fn a_changed_ask_never_panics_and_what_reads_writes_back_the_same(
        ask in an_ask(),
        pick in any::<Index>(),
        flip in 1..=u8::MAX,
    ) {
        let mut bytes = ask_bytes(&ask);
        let at = pick.index(bytes.len());
        bytes[at] ^= flip;
        let mut r = Counted::new(&bytes[..]);
        if let Ok(read) = Ask::read(&mut r) {
            prop_assert_eq!(ask_bytes(&read), &bytes[..r.taken]);
        }
    }

    #[test]
    fn a_changed_reply_never_panics_and_what_reads_writes_back_the_same(
        reply in a_reply(),
        pick in any::<Index>(),
        flip in 1..=u8::MAX,
    ) {
        let mut bytes = reply_bytes(&reply);
        let at = pick.index(bytes.len());
        bytes[at] ^= flip;
        let mut r = Counted::new(&bytes[..]);
        if let Ok(read) = Reply::read(&mut r) {
            prop_assert_eq!(reply_bytes(&read), &bytes[..r.taken]);
        }
    }

    #[test]
    fn a_cut_frame_fails_cleanly(ask in an_ask(), reply in a_reply(), pick in any::<Index>()) {
        let cases: [(Vec<u8>, Reader); 2] = [(ask_bytes(&ask), read_ask), (reply_bytes(&reply), read_reply)];
        for (bytes, read) in cases {
            let cut = pick.index(bytes.len());
            let want = if cut == 0 { IpcError::Closed } else { IpcError::Short };
            prop_assert_eq!(read(&mut &bytes[..cut]), Err(want));
        }
    }
}
