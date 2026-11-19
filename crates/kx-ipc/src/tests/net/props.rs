//! Net properties: what is written reads back, any bytes are refused cleanly within the frame and the caps, and what
//! reads from any bytes holds every rule, as writing it back proves.

use super::*;
use crate::tests::props::check_bounded;
use crate::{NET_MAX_MB, NET_RENEW_S};
use proptest::prelude::*;
use proptest::sample::Index;

/// The most fixed fields or pieces a case sends.
const MAX_CASE_ITEMS: usize = 3;
/// The longest sound or text a case sends, in bytes.
const MAX_CASE_BYTES: usize = 64;
/// The longest junk a case reads.
const MAX_JUNK_BYTES: usize = 256;
/// How far around each cap a case's length lands.
const NEAR_CAP: u32 = 2;
/// A plain HTTPS address, its last label never a number, with maybe a port, a path and a query.
const URL: &str = "https://([a-z0-9-]{1,8}\\.){0,2}[a-z][a-z0-9-]{0,7}(:[1-9][0-9]{0,3})?(/[A-Za-z0-9._~/?=&%-]{0,24})?";
/// A request's name.
const NAME: &str = "[A-Za-z0-9_-]{1,16}";
/// A form value: visible ASCII and spaces.
const VALUE: &str = "[ -~]{0,24}";
/// A key: visible ASCII.
const KEY: &str = "[!-~]{1,24}";
/// A model id: names parted by dots, slashes and colons.
const MODEL_ID: &str = "[A-Za-z0-9_-]{1,8}([./:][A-Za-z0-9_-]{1,8}){0,2}";

fn owned(s: String) -> Cow<'static, str> {
    Cow::Owned(s)
}

fn a_shape() -> impl Strategy<Value = NetShape<'static>> {
    let field = (NAME, VALUE).prop_map(|(name, value)| NetField {
        name: owned(name),
        value: owned(value),
    });
    let fixed = prop::collection::vec(field, 0..=MAX_CASE_ITEMS);
    let flags = (any::<bool>(), any::<bool>());
    let region = prop::option::of(NAME);
    let form = (NAME, flags, fixed, region).prop_map(|(lang, (list, paywall), fixed, region)| {
        NetShape::Form(NetForm {
            lang: owned(lang),
            list,
            fixed,
            paywall,
            region: region.map(owned),
        })
    });
    prop_oneof![form, Just(NetShape::Eleven), Just(NetShape::Gemini)]
}

fn a_call() -> impl Strategy<Value = NetCall<'static>> {
    let header = prop_oneof![
        Just(NetHeader::Bearer),
        NAME.prop_map(|n| NetHeader::Named(owned(n)))
    ];
    let key = KEY.prop_map(|k| Wiped::Kept(Zeroizing::new(k.into_bytes())));
    (a_shape(), URL, header, key, NET_WAIT_S).prop_map(|(shape, url, header, key, wait_s)| {
        NetCall {
            shape,
            url: owned(url),
            header,
            key,
            wait_s,
        }
    })
}

fn a_run() -> impl Strategy<Value = NetRun<'static>> {
    let limits = (NET_MAX_MB, NET_MAX_S, NET_PER_MIN, NET_AT_ONCE).prop_map(
        |(max_mb, max_s, per_min, at_once)| NetLimits {
            max_mb,
            max_s,
            per_min,
            at_once,
        },
    );
    let samples = 1..=u32::try_from(MAX_CASE_BYTES / SAMPLE_BYTES).unwrap();
    let size = samples.prop_map(|n| n * u32::try_from(SAMPLE_BYTES).unwrap());
    let sizes = prop::collection::vec(size, 1..=MAX_CASE_ITEMS);
    let model = prop::option::of(MODEL_ID.prop_map(owned));
    (model, NAME, limits, NET_TRIES, sizes).prop_map(|(model, lang, limits, tries, sizes)| NetRun {
        model,
        lang: owned(lang),
        limits,
        tries,
        sizes,
    })
}

fn a_net_ask() -> impl Strategy<Value = NetAsk<'static>> {
    let id = any::<u32>;
    let piece = 0..u16::try_from(MAX_SPEECH_PIECES).unwrap();
    let samples = prop::collection::vec(any::<[u8; 2]>(), 1..MAX_CASE_BYTES / 2);
    let sound = (id(), piece, samples).prop_map(|(id, piece, s)| NetAsk::Sound {
        id,
        piece,
        bytes: Wiped::Kept(Zeroizing::new(s.concat())),
    });
    let send = (id(), a_call(), a_run(), MODEL_ID).prop_map(|(id, call, mut run, named)| {
        if call.shape.code().needs_model() {
            run.model.get_or_insert(owned(named));
        }
        NetAsk::Send { id, call, run }
    });
    prop_oneof![
        send,
        sound,
        (id(), a_call()).prop_map(|(id, call)| NetAsk::Test { id, call }),
        id().prop_map(|id| NetAsk::Cancel { id }),
        Just(NetAsk::Bye),
    ]
}

fn a_fault() -> impl Strategy<Value = NetFault> {
    let renews_s =
        prop::option::of(NET_RENEW_S).prop_map(|renews_s| NetFault::Allowance { renews_s });
    prop_oneof![prop::sample::select(every_fault()), renews_s]
}

fn a_net_note() -> impl Strategy<Value = NetNote> {
    let id = any::<u32>;
    let index = 0..u16::try_from(MAX_SPEECH_PIECES).unwrap();
    let said =
        prop::collection::vec(any::<char>(), 0..MAX_CASE_BYTES / 4).prop_map(String::from_iter);
    prop_oneof![
        (id(), index, said).prop_map(|(id, index, s)| NetNote::Text {
            id,
            index,
            text: text(&s)
        }),
        id().prop_map(|id| NetNote::Done { id }),
        (id(), a_fault()).prop_map(|(id, fault)| NetNote::Fault { id, fault }),
    ]
}

/// Payload lengths: small ones, ones around each net kind's cap, and any at all.
fn a_len() -> impl Strategy<Value = u32> {
    let near = |kind: Kind| {
        let cap = u32::try_from(*kind.lens().end()).unwrap();
        cap.saturating_sub(NEAR_CAP)..=cap + NEAR_CAP
    };
    prop_oneof![
        0..64u32,
        near(Kind::NetSend),
        near(Kind::NetSound),
        near(Kind::NetText),
        any::<u32>()
    ]
}

/// A net kind's code, ask or note.
fn a_net_kind() -> impl Strategy<Value = Kind> {
    let kinds = net_ask::KINDS.iter().chain(net_note::KINDS);
    prop::sample::select(kinds.copied().collect::<Vec<_>>())
}

/// Fails the case unless whatever reads from `bytes` writes back as the bytes it took, so it holds every rule.
fn reads_back(bytes: &[u8]) -> Result<(), TestCaseError> {
    let mut r = Counted::new(bytes);
    if let Ok(ask) = NetAsk::read(&mut r) {
        prop_assert_eq!(net_ask_bytes(&ask), &bytes[..r.taken]);
    }
    let mut r = Counted::new(bytes);
    if let Ok(note) = NetNote::read(&mut r) {
        prop_assert_eq!(net_note_bytes(&note), &bytes[..r.taken]);
    }
    Ok(())
}

proptest! {
    #[test]
    fn net_asks_round_trip_and_end_at_their_frame(ask in a_net_ask()) {
        let bytes = net_ask_bytes(&ask);
        let mut r = Counted::new(&bytes[..]);
        prop_assert_eq!(NetAsk::read(&mut r), Ok(ask));
        prop_assert_eq!(r.taken, bytes.len());
    }

    #[test]
    fn net_notes_round_trip_and_end_at_their_frame(note in a_net_note()) {
        let bytes = net_note_bytes(&note);
        let mut r = Counted::new(&bytes[..]);
        prop_assert_eq!(NetNote::read(&mut r), Ok(note));
        prop_assert_eq!(r.taken, bytes.len());
    }

    #[test]
    fn random_bytes_never_panic_a_net_reader(junk in prop::collection::vec(any::<u8>(), 0..MAX_JUNK_BYTES)) {
        for read in [read_net_ask as Reader, read_net_note] {
            let mut r = &junk[..];
            while read(&mut r).is_ok() {}
        }
    }

    #[test]
    fn junk_after_the_magic_and_version_never_panics_and_reads_only_as_frames_that_hold_every_rule(
        kind in prop_oneof![a_net_kind().prop_map(Code::code), any::<u8>()],
        junk in prop::collection::vec(any::<u8>(), 0..MAX_JUNK_BYTES),
    ) {
        let bytes = [&MAGIC[..], &[VERSION, kind], &junk].concat();
        for read in [read_net_ask as Reader, read_net_note] {
            let mut r = &bytes[..];
            while read(&mut r).is_ok() {}
        }
        reads_back(&bytes)?;
    }

    #[test]
    fn random_payloads_under_a_net_header_read_only_as_frames_that_hold_every_rule(
        kind in a_net_kind(),
        id in any::<u32>(),
        payload in prop::collection::vec(any::<u8>(), 0..MAX_JUNK_BYTES),
    ) {
        reads_back(&frame(kind, id, &payload))?;
    }

    #[test]
    fn a_good_net_header_is_never_read_past_its_frame_or_cap(
        kind in any::<u8>(),
        len in a_len(),
        tail in prop::collection::vec(any::<u8>(), 0..MAX_JUNK_BYTES),
    ) {
        check_bounded(kind, len, &tail, read_net_ask, net_ask::KINDS)?;
        check_bounded(kind, len, &tail, read_net_note, net_note::KINDS)?;
    }

    #[test]
    fn a_changed_net_frame_never_panics_and_what_reads_writes_back_the_same(
        ask in a_net_ask(),
        note in a_net_note(),
        pick in any::<Index>(),
        flip in 1..=u8::MAX,
    ) {
        for mut bytes in [net_ask_bytes(&ask), net_note_bytes(&note)] {
            let at = pick.index(bytes.len());
            bytes[at] ^= flip;
            reads_back(&bytes)?;
        }
    }

    #[test]
    fn a_cut_net_frame_fails_cleanly(ask in a_net_ask(), note in a_net_note(), pick in any::<Index>()) {
        let cases: [(Vec<u8>, Reader); 2] =
            [(net_ask_bytes(&ask), read_net_ask), (net_note_bytes(&note), read_net_note)];
        for (bytes, read) in cases {
            let cut = pick.index(bytes.len());
            let want = if cut == 0 { IpcError::Closed } else { IpcError::Short };
            prop_assert_eq!(read(&mut &bytes[..cut]), Err(want));
        }
    }
}
