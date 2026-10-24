//! The download budget and its helpers: the words budget cuts the rarest words, a build over the deflated total is
//! refused, and Tatoeba's overused tokens get a factor for the chains they end.

use std::collections::HashMap;

use super::super::WordsError;
use super::super::cands::candidates;
use super::super::download::{check, deflated};
use super::super::fit::size;
use super::super::fixture::{self, text::cand};
use super::super::list::{self, WordList};
use super::super::manifest::Packs;
use super::super::report::report;
use super::super::skew::Shares;
use super::super::split::Splitter;
use super::super::vocab::Vocab;
use super::super::wordfreq::share;
use super::{entries, plan, spellings};

/// Four words of a small list, most frequent first.
const WORDS: [(&str, u32); 4] = [("der", 120), ("die", 130), ("das", 140), ("und", 150)];

/// The small list of `WORDS` cut to a words budget of `budget` bytes.
fn cut_to(budget: usize) -> WordList {
    let rules = fixture::rules();
    let mut cands = candidates(entries(&WORDS), &plan(10), None, &rules);
    cands.words_budget = budget;
    list::finish(cands, None, &rules).unwrap()
}

#[test]
fn the_words_budget_cuts_the_rarest_words_until_the_pack_fits() {
    let all = cut_to(usize::MAX);
    assert_eq!((all.words.len(), all.cut.0), (4, 0));
    let two = size("fr", all.words.get(..2).unwrap()).unwrap();
    let cut = cut_to(two);
    assert_eq!(spellings(&cut), ["der", "die"]);
    assert_eq!(cut.cut, (2, two));
    assert_eq!(cut.of_cand, [Some(0), Some(1), None, None]);
    assert_eq!(spellings(&cut_to(two - 1)), ["der"], "one byte short");
    let kept = share(120) + share(130);
    let total: f64 = WORDS.iter().map(|w| share(w.1)).sum();
    assert!((cut.covers - kept / total).abs() < 1e-12, "{}", cut.covers);
    let lines = report("de", &cut);
    let budget = format!("budget de: the 2 rarest words cut to fit {two} bytes");
    assert!(lines.contains(&budget), "{lines:?}");
    let covers = format!(
        "covering {:.1}% of its frequency mass",
        100.0 * kept / total
    );
    assert!(
        lines.first().is_some_and(|l| l.ends_with(&covers)),
        "{lines:?}"
    );
}

#[test]
fn a_build_over_the_download_budget_is_refused() {
    let packs = Packs {
        download_kb: 1,
        level: 6,
    };
    let line = check(&[(5_000, 600), (100, 400)], &packs).unwrap();
    assert_eq!(
        line,
        "download: 2 files, 5100 bytes, 1000 deflated of the 1000 allowed"
    );
    let refused = check(&[(5_000, 601), (100, 400)], &packs);
    let why = match refused {
        Err(WordsError::Pack(why)) => why,
        other => panic!("{other:?}"),
    };
    assert!(why.ends_with("1001 deflated of the 1000 allowed; the build is refused"));
}

#[test]
fn deflate_shrinks_repeats_and_level_0_stores() {
    let bytes = vec![7_u8; 10_000];
    assert!(deflated(&bytes, 6).unwrap() < 100);
    assert!(deflated(&bytes, 0).unwrap() >= bytes.len());
}

#[test]
fn only_a_wordfreq_token_past_the_ratio_gets_a_factor_and_a_repeat_counts_once() {
    let cands = [
        cand(&["Tom"], 400, 0),
        cand(&["the"], 120, 1),
        cand(&["don't"], 400, 2),
    ];
    let tokens = HashMap::from([("tom".to_string(), 400), ("the".to_string(), 120)]);
    let vocab = Vocab::new(&cands);
    let split = Splitter {
        ends: &['.'],
        apostrophe: '\'',
    };
    let mut shares = Shares::new(&vocab, cands.len(), split, 10);
    shares.add("Tom saw the cat. Tom saw the cat. Tom don't.");
    let (factors, tally) = shares.factors(&cands, &tokens, 10.0, 2);
    let tom = 2.0 / 6.0 / share(400);
    assert_eq!(tally.first, [format!("tom {tom:.0}")]);
    let want = [10.0 / tom, 1.0, 1.0];
    let near = factors
        .iter()
        .zip(want)
        .all(|(f, w)| (f64::from(*f) - w).abs() < 1e-6);
    assert!(
        near,
        "{factors:?}: the is under the ratio, don't's bucket is borrowed"
    );
}
