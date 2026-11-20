//! The numbers on the shelf: counts, card checks and Compare runs come back, a damaged blob starts empty and goes,
//! and counts past one blob drop their oldest days.

use super::*;

/// A dictation's end, on a card or not.
fn done(card: bool) -> SpeechDone {
    SpeechDone {
        pieces: 2,
        sound_ms: 4_000,
        wait_ms: 700,
        load_ms: 300,
        memory_mb: 900,
        card,
    }
}

/// A dictation by the row, on card `Vulkan0` or the processor.
fn count(card: bool) -> Count {
    let model = Redacted::new(ROW.into());
    let done = done(card);
    let card = card.then(|| CardId::new("Vulkan0").unwrap());
    Count { model, card, done }
}

/// Puts one count, one card check and one Compare run in `o`.
fn fill(o: &mut Opened) {
    o.counts.add(day(14), &count(true));
    o.counts.add(day(14), &count(false));
    let card = CardId::new("Vulkan0").unwrap();
    let verdict = Verdict::Passed {
        gain: Hundredths(900),
    };
    let (driver, model, day) = ("32.0.15.6094".into(), print(1), day(14));
    let checked = Checked {
        card: card.clone(),
        driver,
        model,
        lang: Some("ar".into()),
        verdict,
        day,
    };
    o.checks.put(checked);
    let (wrong, words, wait_ms, mb) = (Some(3), 70, 1_200, 800);
    let model = print(2);
    let scores = vec![Score {
        model,
        wrong,
        words,
        wait_ms,
        mb,
    }];
    let (lang, card, auto) = ("fr".into(), Some(card), true);
    o.scores.put(Board {
        lang,
        day,
        auto,
        card,
        scores,
    });
}

#[test]
fn numbers_come_back_and_a_damaged_one_starts_empty() {
    let shelf = Arc::new(Mem::default());
    let mut o = open(&shelf, MAX_BLOB_BYTES);
    fill(&mut o);
    o.store.save_counts(&mut o.counts).unwrap();
    o.store.save_checks(&o.checks).unwrap();
    o.store.save_scores(&o.scores).unwrap();
    let back = open(&shelf, MAX_BLOB_BYTES);
    let numbers = |o: &Opened| (o.counts.clone(), o.checks.clone(), o.scores.clone());
    assert_eq!(numbers(&back), numbers(&o));
    shelf.plant("numbers", "checks", b"junk");
    let damaged = open(&shelf, MAX_BLOB_BYTES);
    let numbers_only = Damage {
        texts: 0,
        numbers: 1,
    };
    assert_eq!(damaged.damaged, numbers_only, "no text is named");
    assert!(damaged.checks.rows().is_empty());
    assert_eq!(damaged.counts, o.counts);
    let after = open(&shelf, MAX_BLOB_BYTES).damaged;
    assert_eq!(after, Damage::default(), "the junk was removed");
}

#[test]
fn counts_past_one_blob_drop_their_oldest_days() {
    let shelf = Arc::new(Mem::default());
    let mut o = open(&shelf, 256);
    for n in 1..=20 {
        o.counts.add(day(n), &count(false));
    }
    o.store.save_counts(&mut o.counts).unwrap();
    let back = open(&shelf, 256);
    let days: Vec<Date> = back.counts.rows().keys().map(|b| b.date).collect();
    assert!(!days.is_empty() && days.len() < 20);
    assert_eq!(days.last(), Some(&day(20)), "the newest day stays");
    assert_eq!(back.counts, o.counts);
}
