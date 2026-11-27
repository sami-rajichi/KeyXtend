//! Pick properties: what is written reads back, and any bytes are refused cleanly, within the frame and the caps.

use super::*;
use crate::tests::props::check_bounded;
use proptest::prelude::*;
use proptest::sample::Index;

/// The longest junk a case reads.
const MAX_JUNK_BYTES: usize = 256;
/// How far around each cap a case's length lands.
const NEAR_CAP: u32 = 2;
/// A plain path: a drive and parts of any script that never name a device or hold a control character.
const PLAIN_PATH: &str = "[A-Z]:(\\\\snip [^\\x00-\\x1f\\x7f-\\x9f\\\\/]{0,8}){0,3}";
/// A plain file name of any script: no control character, separator or mark Windows refuses.
const PLAIN_NAME: &str = "snip [^\\x00-\\x1f\\x7f-\\x9f<>:\"/\\\\|?*]{0,12}";

fn a_path() -> impl Strategy<Value = Cow<'static, str>> {
    PLAIN_PATH.prop_map(Cow::Owned)
}

fn a_pick_ask() -> impl Strategy<Value = PickAsk<'static>> {
    let id = any::<u32>;
    let kind = prop::sample::select(FileKind::ALL);
    let name = PLAIN_NAME.prop_map(Cow::Owned);
    let save = (id(), a_path(), name, kind).prop_map(|(id, folder, name, kind)| PickAsk::Save {
        id,
        folder,
        name,
        kind,
    });
    prop_oneof![
        save,
        (id(), a_path()).prop_map(|(id, at)| PickAsk::Folder { id, at }),
        Just(PickAsk::Bye),
    ]
}

fn a_pick_reply() -> impl Strategy<Value = PickReply<'static>> {
    let id = any::<u32>;
    prop_oneof![
        (id(), a_path()).prop_map(|(id, path)| PickReply::Picked { id, path }),
        id().prop_map(|id| PickReply::Cancelled { id }),
        id().prop_map(|id| PickReply::Refused { id }),
        (id(), any::<u32>()).prop_map(|(id, code)| PickReply::Failed { id, code }),
    ]
}

/// Payload lengths: small ones, ones around the Save ask's cap, and any at all.
fn a_len() -> impl Strategy<Value = u32> {
    let cap = u32::try_from(pick_wire::SAVE_MOST).unwrap();
    prop_oneof![0..64u32, cap - NEAR_CAP..=cap + NEAR_CAP, any::<u32>()]
}

proptest! {
    #[test]
    fn pick_asks_round_trip_and_end_at_their_frame(ask in a_pick_ask()) {
        let bytes = pick_ask_bytes(&ask);
        let mut r = Counted::new(&bytes[..]);
        prop_assert_eq!(PickAsk::read(&mut r), Ok(ask));
        prop_assert_eq!(r.taken, bytes.len());
    }

    #[test]
    fn pick_replies_round_trip_and_end_at_their_frame(reply in a_pick_reply()) {
        let bytes = pick_reply_bytes(&reply);
        let mut r = Counted::new(&bytes[..]);
        prop_assert_eq!(PickReply::read(&mut r), Ok(reply));
        prop_assert_eq!(r.taken, bytes.len());
    }

    #[test]
    fn random_bytes_never_panic_a_pick_reader(junk in prop::collection::vec(any::<u8>(), 0..MAX_JUNK_BYTES)) {
        for read in [read_pick_ask as Reader, read_pick_reply] {
            let mut r = &junk[..];
            while read(&mut r).is_ok() {}
        }
    }

    #[test]
    fn junk_under_a_good_pick_header_never_panics(
        kind in prop::sample::select(pick_ask::KINDS.iter().chain(pick_reply::KINDS).copied().collect::<Vec<_>>()),
        junk in prop::collection::vec(any::<u8>(), 0..MAX_JUNK_BYTES),
    ) {
        let bytes = frame(kind, 1, &junk);
        let _ = read_pick_ask(&mut &bytes[..]);
        let _ = read_pick_reply(&mut &bytes[..]);
    }

    #[test]
    fn a_good_pick_header_is_never_read_past_its_frame_or_cap(
        kind in any::<u8>(),
        len in a_len(),
        tail in prop::collection::vec(any::<u8>(), 0..MAX_JUNK_BYTES),
    ) {
        check_bounded(kind, len, &tail, read_pick_ask, pick_ask::KINDS)?;
        check_bounded(kind, len, &tail, read_pick_reply, pick_reply::KINDS)?;
    }

    #[test]
    fn a_changed_pick_frame_never_panics_and_what_reads_writes_back_the_same(
        ask in a_pick_ask(),
        reply in a_pick_reply(),
        pick in any::<Index>(),
        flip in 1..=u8::MAX,
    ) {
        let mut bytes = pick_ask_bytes(&ask);
        let at = pick.index(bytes.len());
        bytes[at] ^= flip;
        let mut r = Counted::new(&bytes[..]);
        if let Ok(read) = PickAsk::read(&mut r) {
            prop_assert_eq!(pick_ask_bytes(&read), &bytes[..r.taken]);
        }
        let mut bytes = pick_reply_bytes(&reply);
        let at = pick.index(bytes.len());
        bytes[at] ^= flip;
        let mut r = Counted::new(&bytes[..]);
        if let Ok(read) = PickReply::read(&mut r) {
            prop_assert_eq!(pick_reply_bytes(&read), &bytes[..r.taken]);
        }
    }

    #[test]
    fn a_cut_pick_frame_fails_cleanly(ask in a_pick_ask(), reply in a_pick_reply(), pick in any::<Index>()) {
        let cases: [(Vec<u8>, Reader); 2] =
            [(pick_ask_bytes(&ask), read_pick_ask), (pick_reply_bytes(&reply), read_pick_reply)];
        for (bytes, read) in cases {
            let cut = pick.index(bytes.len());
            let want = if cut == 0 { IpcError::Closed } else { IpcError::Short };
            prop_assert_eq!(read(&mut &bytes[..cut]), Err(want));
        }
    }
}
