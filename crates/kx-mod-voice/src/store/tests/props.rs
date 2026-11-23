//! Properties of the file forms: random bytes, a good file cut or changed anywhere, and a shelf of random blobs never
//! panic; any text, count, check or score comes back as it was saved.

use std::collections::BTreeMap;

use proptest::prelude::*;

use super::*;
use crate::counts::{Bucket, Checks, Counts, Device, Model, Scores, Tally};
use crate::history::{Entry, Sound};

/// Texts of Latin, Arabic, marks and spaces.
fn words() -> impl Strategy<Value = String> {
    let chars = vec!['a', 'É', 'ع', 'ة', 'ً', ' ', '\u{200f}', '7', '.'];
    prop::collection::vec(prop::sample::select(chars), 0..12).prop_map(|v| v.into_iter().collect())
}

fn entry() -> impl Strategy<Value = Entry> {
    let sound = prop::option::of(
        (1..u64::MAX, 1..u32::MAX).prop_map(|(bytes, blobs)| Sound { bytes, blobs }),
    );
    let ids = (
        any::<u64>(),
        any::<u64>(),
        any::<i64>(),
        any::<i16>(),
        any::<bool>(),
    );
    (ids, words(), words(), words(), sound).prop_map(
        |((id, seq, unix_ms, offset_min, pinned), t, l, m, sound)| Entry {
            id,
            seq,
            at: Stamp {
                unix_ms,
                offset_min,
            },
            text: secret(&t),
            lang: l,
            model: Redacted::new(m),
            pinned,
            sound,
        },
    )
}

fn date() -> impl Strategy<Value = Date> {
    (1..=u16::MAX, 1..=12_u8, 1..=28_u8).prop_map(|(y, m, d)| Date::new(y, m, d).unwrap())
}

fn counts() -> impl Strategy<Value = Counts> {
    let cloud = |model: Option<&str>| Model::Cloud {
        service: "groq".into(),
        model: model.map(str::to_owned),
    };
    let model = prop_oneof![
        Just(Model::Other),
        Just(Model::Row(ROW.into())),
        Just(cloud(Some("whisper-large-v3"))),
        Just(cloud(None)),
    ];
    let device = prop_oneof![
        Just(Device::Cpu),
        card().prop_map(Device::Card),
        Just(Device::Cloud),
    ];
    let bucket = (date(), model, device).prop_map(|(date, model, device)| Bucket {
        date,
        model,
        device,
    });
    let tally =
        (any::<u32>(), any::<u64>(), any::<u64>()).prop_map(|(dictations, sound_ms, wait_ms)| {
            Tally {
                dictations,
                sound_ms,
                wait_ms,
            }
        });
    prop::collection::btree_map(bucket, tally, 0..6).prop_map(Counts::new)
}

fn verdict() -> impl Strategy<Value = Verdict> {
    prop_oneof![
        any::<u32>().prop_map(|g| Verdict::Passed {
            gain: Hundredths(g)
        }),
        (any::<u32>(), any::<u32>()).prop_map(|(g, n)| Verdict::Slow {
            gain: Hundredths(g),
            need: Hundredths(n)
        }),
        (any::<u16>(), any::<u8>())
            .prop_map(|(words, allowed)| Verdict::Differs { words, allowed }),
        Just(Verdict::NoRoom),
        Just(Verdict::Failed),
    ]
}

fn card() -> impl Strategy<Value = CardId> {
    (0..16_u8).prop_map(|n| CardId::new(&format!("Vulkan{n}")).unwrap())
}

fn checks() -> impl Strategy<Value = Checks> {
    let lang = prop::option::of(prop::sample::select(vec!["en", "fr", "ar"]));
    let one = (card(), words(), any::<u8>(), lang, verdict(), date());
    let row = one.prop_map(|(card, driver, m, lang, verdict, day)| Checked {
        card,
        driver,
        model: print(m),
        lang: lang.map(str::to_owned),
        verdict,
        day,
    });
    prop::collection::vec(row, 0..5).prop_map(Checks::new)
}

fn scores() -> impl Strategy<Value = Scores> {
    let score = (
        any::<u8>(),
        prop::option::of(any::<u16>()),
        any::<u16>(),
        any::<u32>(),
        any::<u32>(),
    )
        .prop_map(|(m, wrong, words, wait_ms, mb)| Score {
            model: print(m),
            wrong,
            words,
            wait_ms,
            mb,
        });
    let lang = prop::sample::select(vec!["en", "fr", "ar", "de"]);
    let board = (
        lang,
        date(),
        any::<bool>(),
        prop::option::of(card()),
        prop::collection::vec(score, 0..4),
    )
        .prop_map(|(lang, day, auto, card, scores)| Board {
            lang: lang.into(),
            day,
            auto,
            card,
            scores,
        });
    prop::collection::vec(board, 0..4).prop_map(Scores::new)
}

/// Every decoder, each reading `bytes`.
fn read_all(bytes: &[u8]) {
    let _ = text::decode(bytes);
    let _ = numbers::decode_counts(bytes);
    let _ = numbers::decode_checks(bytes);
    let _ = numbers::decode_langs(bytes);
    let _ = numbers::decode_scores(bytes);
}

proptest! {
    #[test]
    fn random_bytes_never_panic(bytes in prop::collection::vec(any::<u8>(), 0..256)) {
        read_all(&bytes);
        let forms = [
            (text::MARK, text::VERSION),
            (numbers::COUNTS_MARK, numbers::VERSION),
            (numbers::CHECKS_MARK, numbers::VERSION),
            (numbers::LANGS_MARK, numbers::VERSION),
            (numbers::SCORES_MARK, numbers::VERSION),
        ];
        for (mark, version) in forms {
            read_all(&[mark.as_slice(), &[version], &bytes].concat());
        }
    }

    #[test]
    fn any_text_comes_back_and_a_cut_or_changed_one_never_panics(e in entry(), at: prop::sample::Index, byte: u8) {
        let bytes = text::encode(&e);
        prop_assert_eq!(text::decode(&bytes), Ok(e));
        let at = at.index(bytes.len());
        prop_assert!(text::decode(&bytes[..at]).is_err());
        let mut changed = bytes.to_vec();
        changed[at] = byte;
        let _ = text::decode(&changed);
    }

    #[test]
    fn any_numbers_come_back(c in counts(), k in checks(), s in scores()) {
        prop_assert_eq!(numbers::decode_counts(&numbers::encode_counts(&c)), Ok(c));
        let plain = numbers::decode_checks(&numbers::encode_checks(&k)).unwrap();
        let langs = numbers::decode_langs(&numbers::encode_langs(&k)).unwrap();
        // The blob puts passes first, so the checks come back in another order.
        let back = numbers::named(plain, langs);
        let times = |rows: &[_], row| rows.iter().filter(|r| *r == row).count();
        prop_assert_eq!(back.rows().len(), k.rows().len());
        prop_assert!(k.rows().iter().all(|row| times(back.rows(), row) == times(k.rows(), row)));
        prop_assert_eq!(numbers::decode_scores(&numbers::encode_scores(&s)), Ok(s));
    }

    #[test]
    fn a_shelf_of_random_blobs_opens_without_panic(
        blobs in prop::collection::btree_map((0..3_usize, 0..7_usize), prop::collection::vec(any::<u8>(), 0..64), 0..12),
    ) {
        let folders = ["texts", "sounds", "numbers"];
        let names = ["1", "2", "1-0", "2-1", "counts", "checks", "check-langs"];
        let shelf = Arc::new(Mem::default());
        let planted: BTreeMap<_, _> = blobs.into_iter().collect();
        for ((f, n), bytes) in &planted {
            shelf.plant(folders[*f], names[*n], bytes);
        }
        let _ = Store::open(shelf, MAX_BLOB_BYTES);
    }
}
