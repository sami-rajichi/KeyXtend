//! Voice properties: what is written reads back, and any bytes are refused cleanly, within the frame and the caps.

use super::*;
use crate::tests::props::check_bounded;
use crate::{
    LEVEL_HZ, MAX_PERCENT, MAX_PIECES, MAX_SOUND_MS, MAX_SOUND_PIECES, PAD_MS, PIECE_S,
    RECORD_MAX_S, SILENCE_STOP_S, Stage, THREADS, VoiceFault,
};
use proptest::prelude::*;
use proptest::sample::Index;

/// The longest bytes or text a case sends.
const MAX_CASE_BYTES: usize = 64;
/// The most devices or languages a case sends.
const MAX_CASE_ITEMS: usize = 3;
/// The longest junk a case reads.
const MAX_JUNK_BYTES: usize = 256;
/// How far around each cap a case's length lands.
const NEAR_CAP: u32 = 2;
/// A short ASCII name, as a tag holds.
const TAG: &str = "[a-zA-Z0-9._:-]{1,16}";
/// A short path: any text but a NUL.
const PATH: &str = "[^\0]{1,16}";

fn some_text() -> impl Strategy<Value = String> {
    prop::collection::vec(any::<char>(), 0..MAX_CASE_BYTES / 4).prop_map(String::from_iter)
}

fn owned(s: String) -> Cow<'static, str> {
    Cow::Owned(s)
}

fn a_target() -> impl Strategy<Value = Target<'static>> {
    let card = (TAG, any::<u32>()).prop_map(|(id, need_mb)| Target::Card {
        id: owned(id),
        need_mb,
    });
    prop_oneof![Just(Target::Cpu), card]
}

fn a_model() -> impl Strategy<Value = Model<'static>> {
    (PATH, TAG, TAG, a_target(), THREADS).prop_map(|(path, lang, runner, target, threads)| Model {
        path: owned(path),
        lang: owned(lang),
        runner: owned(runner),
        target,
        threads,
    })
}

fn a_setup() -> impl Strategy<Value = Setup<'static>> {
    let numbers = (
        PAD_MS,
        PIECE_S,
        RECORD_MAX_S,
        LEVEL_HZ,
        prop::option::of(SILENCE_STOP_S),
    );
    (prop::option::of(a_model()), numbers, PATH).prop_map(
        |(model, (pad_ms, piece_s, record_max_s, level_hz, silence_stop_s), detector)| Setup {
            model,
            pad_ms,
            piece_s,
            record_max_s,
            level_hz,
            silence_stop_s,
            detector: owned(detector),
        },
    )
}

fn a_piece() -> impl Strategy<Value = SoundPiece<'static>> {
    let samples = prop::collection::vec(any::<[u8; 2]>(), 0..MAX_CASE_BYTES / 2);
    let index = 0..u16::try_from(MAX_SOUND_PIECES).unwrap();
    (index, any::<bool>(), samples).prop_map(|(index, last, samples)| SoundPiece {
        index,
        last,
        bytes: Wiped::Kept(Zeroizing::new(samples.concat())),
    })
}

fn a_voice_ask() -> impl Strategy<Value = VoiceAsk<'static>> {
    let id = any::<u32>;
    prop_oneof![
        (id(), a_setup()).prop_map(|(id, setup)| VoiceAsk::Start { id, setup }),
        (id(), a_setup()).prop_map(|(id, setup)| VoiceAsk::Again { id, setup }),
        id().prop_map(|id| VoiceAsk::Stop { id }),
        id().prop_map(|id| VoiceAsk::Cancel { id }),
        id().prop_map(|id| VoiceAsk::Keep { id }),
        id().prop_map(|id| VoiceAsk::Close { id }),
        id().prop_map(|id| VoiceAsk::Devices { id }),
        (id(), PATH).prop_map(|(id, path)| VoiceAsk::Facts {
            id,
            path: owned(path)
        }),
        (id(), a_model(), PATH).prop_map(|(id, model, clip)| VoiceAsk::Check {
            id,
            model,
            clip: owned(clip)
        }),
        (id(), a_piece()).prop_map(|(id, piece)| VoiceAsk::Sound { id, piece }),
        Just(VoiceAsk::Bye),
    ]
}

fn a_device() -> impl Strategy<Value = Device> {
    let kind = prop::sample::select(DeviceKind::ALL);
    let shown = |n: &String| crate::rules::name(n).is_ok();
    let name = || "[^\\p{Cc}]{0,16}".prop_filter("a shown name", shown);
    (kind, TAG, name(), any::<u32>(), any::<u32>(), name()).prop_map(
        |(kind, id, name, a, b, driver)| Device {
            kind,
            id,
            name,
            total_mb: a.max(b),
            free_mb: a.min(b),
            driver,
        },
    )
}

fn a_run_note() -> impl Strategy<Value = VoiceNote> {
    let id = any::<u32>;
    let index = 0..u16::try_from(MAX_PIECES).unwrap();
    prop_oneof![
        (id(), any::<u8>()).prop_map(|(id, level)| VoiceNote::Level { id, level }),
        (id(), any::<bool>()).prop_map(|(id, speech)| VoiceNote::Heard { id, speech }),
        (id(), prop::sample::select(Stage::ALL))
            .prop_map(|(id, stage)| VoiceNote::Stage { id, stage }),
        (id(), 0..=MAX_PERCENT).prop_map(|(id, percent)| VoiceNote::Warm { id, percent }),
        (id(), index, some_text()).prop_map(|(id, index, s)| VoiceNote::Piece {
            id,
            index,
            text: text(&s)
        }),
        (id(), prop::sample::select(VoiceFault::ALL))
            .prop_map(|(id, fault)| VoiceNote::Fault { id, fault }),
        (id(), a_piece()).prop_map(|(id, piece)| VoiceNote::Sound { id, piece }),
    ]
}

fn a_done() -> impl Strategy<Value = VoiceNote> {
    let pieces = 0..=u16::try_from(MAX_PIECES).unwrap();
    let times = (0..=MAX_SOUND_MS, any::<u32>(), any::<u32>(), any::<u32>());
    (any::<u32>(), pieces, times, any::<bool>()).prop_map(
        |(id, pieces, (sound_ms, wait_ms, load_ms, memory_mb), card)| VoiceNote::Done {
            id,
            pieces,
            sound_ms,
            wait_ms,
            load_ms,
            memory_mb,
            card,
        },
    )
}

fn a_voice_note() -> impl Strategy<Value = VoiceNote> {
    let id = any::<u32>;
    let list = prop::collection::vec(a_device(), 1..=MAX_CASE_ITEMS);
    let langs = prop::collection::vec(TAG, 0..=MAX_CASE_ITEMS);
    let checked = (id(), any::<u32>(), any::<u32>(), some_text(), some_text());
    prop_oneof![
        a_run_note(),
        a_done(),
        (id(), list).prop_map(|(id, list)| VoiceNote::Devices { id, list }),
        (id(), TAG, langs).prop_map(|(id, runner, langs)| VoiceNote::Facts { id, runner, langs }),
        checked.prop_map(|(id, card_ms, cpu_ms, a, b)| VoiceNote::Checked {
            id,
            card_ms,
            cpu_ms,
            card_text: text(&a),
            cpu_text: text(&b),
        }),
    ]
}

/// Payload lengths: small ones, ones around each voice kind's cap, and any at all.
fn a_len() -> impl Strategy<Value = u32> {
    let near = |kind: Kind| {
        let cap = u32::try_from(*kind.lens().end()).unwrap();
        cap.saturating_sub(NEAR_CAP)..=cap + NEAR_CAP
    };
    prop_oneof![
        0..64u32,
        near(Kind::VoiceStart),
        near(Kind::SoundIn),
        near(Kind::DeviceList),
        near(Kind::CardChecked),
        any::<u32>()
    ]
}

proptest! {
    #[test]
    fn voice_asks_round_trip_and_end_at_their_frame(ask in a_voice_ask()) {
        let bytes = voice_ask_bytes(&ask);
        let mut r = Counted::new(&bytes[..]);
        prop_assert_eq!(VoiceAsk::read(&mut r), Ok(ask));
        prop_assert_eq!(r.taken, bytes.len());
    }

    #[test]
    fn voice_notes_round_trip_and_end_at_their_frame(note in a_voice_note()) {
        let bytes = voice_note_bytes(&note);
        let mut r = Counted::new(&bytes[..]);
        prop_assert_eq!(VoiceNote::read(&mut r), Ok(note));
        prop_assert_eq!(r.taken, bytes.len());
    }

    #[test]
    fn random_bytes_never_panic_a_voice_reader(junk in prop::collection::vec(any::<u8>(), 0..MAX_JUNK_BYTES)) {
        for read in [read_voice_ask as Reader, read_voice_note] {
            let mut r = &junk[..];
            while read(&mut r).is_ok() {}
        }
    }

    #[test]
    fn a_good_voice_header_is_never_read_past_its_frame_or_cap(
        kind in any::<u8>(),
        len in a_len(),
        tail in prop::collection::vec(any::<u8>(), 0..MAX_JUNK_BYTES),
    ) {
        check_bounded(kind, len, &tail, read_voice_ask, voice_ask::KINDS)?;
        check_bounded(kind, len, &tail, read_voice_note, voice_note::KINDS)?;
    }

    #[test]
    fn a_changed_voice_ask_never_panics_and_what_reads_writes_back_the_same(
        ask in a_voice_ask(),
        pick in any::<Index>(),
        flip in 1..=u8::MAX,
    ) {
        let mut bytes = voice_ask_bytes(&ask);
        let at = pick.index(bytes.len());
        bytes[at] ^= flip;
        let mut r = Counted::new(&bytes[..]);
        if let Ok(read) = VoiceAsk::read(&mut r) {
            prop_assert_eq!(voice_ask_bytes(&read), &bytes[..r.taken]);
        }
    }

    #[test]
    fn a_changed_voice_note_never_panics_and_what_reads_writes_back_the_same(
        note in a_voice_note(),
        pick in any::<Index>(),
        flip in 1..=u8::MAX,
    ) {
        let mut bytes = voice_note_bytes(&note);
        let at = pick.index(bytes.len());
        bytes[at] ^= flip;
        let mut r = Counted::new(&bytes[..]);
        if let Ok(read) = VoiceNote::read(&mut r) {
            prop_assert_eq!(voice_note_bytes(&read), &bytes[..r.taken]);
        }
    }

    #[test]
    fn a_cut_voice_frame_fails_cleanly(ask in a_voice_ask(), note in a_voice_note(), pick in any::<Index>()) {
        let cases: [(Vec<u8>, Reader); 2] =
            [(voice_ask_bytes(&ask), read_voice_ask), (voice_note_bytes(&note), read_voice_note)];
        for (bytes, read) in cases {
            let cut = pick.index(bytes.len());
            let want = if cut == 0 { IpcError::Closed } else { IpcError::Short };
            prop_assert_eq!(read(&mut &bytes[..cut]), Err(want));
        }
    }
}
