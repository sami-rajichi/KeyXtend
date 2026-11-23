//! The numbers (plan task 18): counts add up per day, model and card and hold numbers only; a card's check is found
//! for its driver and file alone, and only that file's goes when the card fails (edges 25, 26); Compare's numbers
//! replace a language's, and only an ISO code's are kept.

use super::*;
use kx_module_api::Redacted;
use kx_module_api::ports::{CardId, Fingerprint, Hundredths, SpeechDone, Verdict};

mod checks;
mod cloud;

/// A row of the models table.
const ROW: &str = "parakeet-v3-q8";

fn day(n: u8) -> Date {
    Date::new(2026, 11, n).unwrap()
}

/// A dictation by `model` of `sound_ms` heard and `wait_ms` waited, on card `card` or the processor.
fn count(model: &str, sound_ms: u32, wait_ms: u32, card: Option<&str>) -> Count {
    let done = SpeechDone {
        pieces: 1,
        sound_ms,
        wait_ms,
        load_ms: 400,
        memory_mb: 900,
        card: card.is_some(),
    };
    Count {
        model: Redacted::new(model.into()),
        card: card.map(self::card),
        done,
    }
}

fn bucket(n: u8, model: Model, device: Device) -> Bucket {
    Bucket {
        date: day(n),
        model,
        device,
    }
}

fn card(id: &str) -> CardId {
    CardId::new(id).unwrap()
}

fn print(n: u8) -> Fingerprint {
    Fingerprint::new([n; Fingerprint::BYTES])
}

/// `card`'s check of file `model` under `driver`, on the English clip.
fn checked(card_id: &str, driver: &str, model: u8, verdict: Verdict) -> Checked {
    Checked {
        card: card(card_id),
        driver: driver.into(),
        model: print(model),
        lang: Some("en".into()),
        verdict,
        day: day(14),
    }
}

fn passed(gain: u32) -> Verdict {
    Verdict::Passed {
        gain: Hundredths(gain),
    }
}

#[test]
fn counts_add_up_per_day_model_and_device() {
    let mut counts = Counts::default();
    counts.add(day(14), &count(ROW, 4_000, 900, None));
    counts.add(day(14), &count(ROW, 2_500, 600, None));
    counts.add(day(14), &count(ROW, 1_000, 100, Some("Vulkan0")));
    counts.add(day(14), &count(ROW, 500, 50, Some("Vulkan1")));
    counts.add(day(15), &count(ROW, 3_000, 300, None));
    let row = Model::Row(ROW.into());
    let one = |sound_ms, wait_ms| Tally {
        dictations: 1,
        sound_ms,
        wait_ms,
    };
    let want = BTreeMap::from([
        (
            bucket(14, row.clone(), Device::Cpu),
            Tally {
                dictations: 2,
                sound_ms: 6_500,
                wait_ms: 1_500,
            },
        ),
        (
            bucket(14, row.clone(), Device::Card(card("Vulkan0"))),
            one(1_000, 100),
        ),
        (
            bucket(14, row.clone(), Device::Card(card("Vulkan1"))),
            one(500, 50),
        ),
        (bucket(15, row, Device::Cpu), one(3_000, 300)),
    ]);
    assert_eq!(counts.rows(), &want);
}

#[test]
fn a_file_not_in_the_table_counts_as_other_and_its_name_is_never_kept() {
    let mut counts = Counts::default();
    counts.add(day(14), &count("my secret plan.gguf", 1_000, 100, None));
    counts.add(day(14), &count("another.bin", 1_000, 100, None));
    let only = counts
        .rows()
        .keys()
        .map(|b| b.model.clone())
        .collect::<Vec<_>>();
    assert_eq!(only, [Model::Other]);
    assert_eq!(counts.rows().values().next().unwrap().dictations, 2);
    assert!(!format!("{counts:?}").contains("secret"));
}

#[test]
fn a_count_saturates_rather_than_wraps() {
    let key = bucket(14, Model::Other, Device::Cpu);
    let full = Tally {
        dictations: u32::MAX,
        sound_ms: u64::MAX,
        wait_ms: u64::MAX,
    };
    let mut counts = Counts::new(BTreeMap::from([(key.clone(), full)]));
    counts.add(day(14), &count("x.gguf", 5, 5, None));
    assert_eq!(counts.rows()[&key], full);
}

#[test]
fn drop_oldest_takes_the_whole_oldest_day() {
    let mut counts = Counts::default();
    counts.add(day(12), &count(ROW, 1, 1, None));
    counts.add(day(12), &count(ROW, 1, 1, Some("Vulkan0")));
    counts.add(day(13), &count(ROW, 1, 1, None));
    assert!(counts.drop_oldest());
    let days: Vec<Date> = counts.rows().keys().map(|b| b.date).collect();
    assert_eq!(days, [day(13)]);
    assert!(counts.drop_oldest());
    assert!(!counts.drop_oldest(), "nothing left");
}

#[test]
fn a_check_is_found_for_its_card_driver_and_file_only() {
    let mut checks = Checks::default();
    checks.put(checked("Vulkan0", "32.0.1", 1, passed(900)));
    assert_eq!(
        checks.verdict(&card("Vulkan0"), "32.0.1", &print(1), "en"),
        Some(passed(900))
    );
    assert_eq!(
        checks.verdict(&card("Vulkan0"), "32.0.2", &print(1), "en"),
        None,
        "a driver update"
    );
    assert_eq!(
        checks.verdict(&card("Vulkan0"), "32.0.1", &print(2), "en"),
        None,
        "a new model file"
    );
    assert_eq!(
        checks.verdict(&card("Vulkan1"), "32.0.1", &print(1), "en"),
        None,
        "another card"
    );
}

#[test]
fn a_new_check_replaces_the_cards_result_for_that_file_whatever_its_driver() {
    let mut checks = Checks::default();
    checks.put(checked("Vulkan0", "32.0.1", 1, passed(900)));
    checks.put(checked("Vulkan0", "32.0.1", 2, Verdict::NoRoom));
    let slow = Verdict::Slow {
        gain: Hundredths(120),
        need: Hundredths(150),
    };
    checks.put(checked("Vulkan0", "32.0.2", 1, slow));
    assert_eq!(checks.rows().len(), 2);
    assert_eq!(
        checks.verdict(&card("Vulkan0"), "32.0.2", &print(1), "en"),
        Some(slow)
    );
    assert_eq!(
        checks.verdict(&card("Vulkan0"), "32.0.1", &print(2), "en"),
        Some(Verdict::NoRoom)
    );
}

#[test]
fn edge_25_a_card_that_failed_mid_run_loses_that_files_result_alone() {
    let mut checks = Checks::default();
    checks.put(checked("Vulkan0", "32.0.1", 1, passed(900)));
    checks.put(checked("Vulkan0", "32.0.1", 2, passed(400)));
    checks.put(checked("Vulkan1", "1.2", 1, passed(300)));
    assert!(checks.fell(&card("Vulkan0"), &print(1)));
    assert!(!checks.fell(&card("Vulkan0"), &print(1)));
    assert_eq!(
        checks.verdict(&card("Vulkan0"), "32.0.1", &print(1), "en"),
        None
    );
    assert_eq!(
        checks.verdict(&card("Vulkan0"), "32.0.1", &print(2), "en"),
        Some(passed(400)),
        "another file on that card"
    );
    assert_eq!(
        checks.verdict(&card("Vulkan1"), "1.2", &print(1), "en"),
        Some(passed(300)),
        "that file on another card"
    );
}

#[test]
fn loaded_checks_keep_one_result_per_card_and_file() {
    let rows = vec![
        checked("Vulkan0", "1", 1, passed(200)),
        checked("Vulkan0", "2", 1, passed(300)),
    ];
    let checks = Checks::new(rows);
    assert_eq!(checks.rows(), [checked("Vulkan0", "2", 1, passed(300))]);
}

#[test]
fn compare_numbers_replace_the_languages_last_run() {
    let score = |m: u8, wrong: Option<u16>| Score {
        model: print(m),
        wrong,
        words: 70,
        wait_ms: 900,
        mb: 800,
    };
    let board = |lang: &str, n: u8, scores: Vec<Score>| Board {
        lang: lang.into(),
        day: day(n),
        auto: false,
        card: None,
        scores,
    };
    let mut scores = Scores::default();
    assert!(scores.put(board("fr", 12, vec![score(1, Some(3))])));
    scores.put(board("ar", 12, vec![score(2, Some(0))]));
    scores.put(board("fr", 14, vec![score(1, Some(2)), score(3, None)]));
    for odd in ["", "fr-FR", "EN"] {
        assert!(!scores.put(board(odd, 14, vec![])), "{odd:?}");
    }
    assert_eq!(scores.boards().len(), 2);
    let fr = scores.get("fr").unwrap();
    assert_eq!((fr.day, fr.scores.len()), (day(14), 2));
    assert_eq!(scores.get("ar").unwrap().day, day(12));
    assert!(scores.get("en").is_none());
}
