//! The speed figure at the spec's caps (report only): `guesses` with a full learner, `context_words` 4, over packs
//! sized like the real ones by `xtask/words.toml`, for an empty prefix after a word and for a one-letter prefix.

use std::time::{Duration, Instant};

use toml::{Table, Value};

use super::super::clean::peak;
use super::size::{INC, MUL, SHIFT, filled};
use super::*;
use crate::pack::layout::{STEPS_PER_ZIPF, ZIPF_OFFSET};
use crate::pack::{Flags, PackWriter};

/// The pack build settings, which size each fixture pack like the real one.
const WORDS_TOML: &str = include_str!("../../../../../xtask/words.toml");
/// The languages timed, each with its fixture letters as first-last pairs: a–z, and 28 Arabic letters no fold
/// changes.
const LANGS: [(&str, &[(char, char)]); 2] = [
    ("en", &[('a', 'z')]),
    (
        "ar",
        &[
            ('\u{0627}', '\u{0628}'),
            ('\u{062A}', '\u{063A}'),
            ('\u{0641}', '\u{0648}'),
            ('\u{064A}', '\u{064A}'),
        ],
    ),
];
/// One learned word in this many is a pack word; the others are new to the pack.
const SEED_EVERY: usize = 2;
/// Queries timed per kind.
const SAMPLES: usize = 400;
/// The percentiles reported.
const PERCENTS: [usize; 2] = [50, 99];
/// Percent in a whole.
const WHOLE: usize = 100;
/// Bytes in a KB, as `words.toml` counts its budgets.
const KB: usize = 1024;
/// Contexts in the probe pack that measures a chain's bytes.
const PROBE: usize = 4_096;
/// Letters before a word's id code, at most: so first letters spread as in real text.
const MAX_LEAD: u64 = 4;
/// Letters of a word's id code: four hold every fixture id in 26 letters.
const CODE: usize = 4;

/// One language's real-pack sizes from `words.toml`.
struct Sizes {
    /// Words in the pack.
    words: usize,
    /// Bytes the pack's chains take.
    chain_bytes: usize,
    /// Followers per context.
    followers: usize,
}

/// The sizes of `iso`'s pack.
fn sizes(iso: &str) -> Sizes {
    let toml: Table = toml::from_str(WORDS_TOML).unwrap();
    let lang = &toml["lang"][iso];
    let int = |v: &Value| usize::try_from(v.as_integer().unwrap()).unwrap();
    Sizes {
        words: int(&lang["max_words"]),
        chain_bytes: int(&lang["text"]["chain_budget_kb"]) * KB,
        followers: int(&toml["text"]["followers"]),
    }
}

/// Word `n` in `letters`: a few repeatable letters, then `n` in four of them, so every word differs.
fn fixture_word(letters: &[char], n: usize) -> String {
    let base = letters.len();
    let mut state = u64::try_from(n)
        .unwrap()
        .wrapping_mul(MUL)
        .wrapping_add(INC);
    let mut out = String::new();
    for _ in 0..(state >> SHIFT) % (MAX_LEAD + 1) {
        state = state.wrapping_mul(MUL).wrapping_add(INC);
        out.push(letters[usize::try_from(state >> SHIFT).unwrap() % base]);
    }
    (0..CODE).fold(n, |rest, _| {
        out.push(letters[rest % base]);
        rest / base
    });
    out
}

/// A pack of `words` fixture words, most frequent first, and `contexts` one-word contexts with their followers.
fn fixture_pack(iso: &str, abc: &[char], s: &Sizes, contexts: usize) -> Vec<u8> {
    let flags = Flags {
        checked: false,
        chains: contexts > 0,
    };
    let followers = u8::try_from(s.followers).unwrap();
    let mut w = PackWriter::new(iso, flags, followers).unwrap();
    let words: Vec<String> = (0..s.words).map(|n| fixture_word(abc, n)).collect();
    let zipf = zipf_bytes(s.words);
    for (word, freq) in words.iter().zip(zipf) {
        w.add_word(word, freq).unwrap();
    }
    for c in 0..contexts {
        for f in 1..=s.followers {
            let next = &words[(c * s.followers + f) % s.words];
            w.add_chain(&[&words[c]], next, u8::MAX).unwrap();
        }
    }
    w.finish().unwrap()
}

/// The frequency bytes of `words` words by Zipf's law, as in real lists: the word at rank r has a share of
/// 1 / (r × H), with H the harmonic number of `words`, which puts the top English word near Zipf 7.9 as wordfreq does.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "Ranks stay far below 2^52, and each byte is clamped to 1..=255 before the cast."
)]
fn zipf_bytes(words: usize) -> Vec<u8> {
    let harmonic: f64 = (1..=words).map(|r| 1.0 / r as f64).sum();
    let byte = |rank: usize| {
        let zipf = f64::from(ZIPF_OFFSET) - (rank as f64 * harmonic).log10();
        (zipf * f64::from(STEPS_PER_ZIPF))
            .round()
            .clamp(1.0, f64::from(u8::MAX)) as u8
    };
    (1..=words).map(byte).collect()
}

/// A pack of words in `abc` sized like `iso`'s real one: its words, and as many contexts as fill its chain budget.
fn sized_pack(iso: &str, abc: &[char]) -> Vec<u8> {
    let s = sizes(iso);
    let bare = fixture_pack(iso, abc, &s, 0).len();
    let per = (fixture_pack(iso, abc, &s, PROBE).len() - bare) / PROBE;
    fixture_pack(iso, abc, &s, (s.chain_bytes / per.max(1)).min(s.words))
}

/// The `PERCENTS` of `times`, sorted first.
fn percentiles(mut times: Vec<Duration>) -> Vec<Duration> {
    times.sort_unstable();
    let at = |p: usize| (times.len() * p / WHOLE).min(times.len() - 1);
    PERCENTS.iter().map(|&p| times[at(p)]).collect()
}

/// The time `guesses` takes for `prefix` after `context`.
fn time(l: &Learner, pack: &Pack<'_>, context: &[String], prefix: &str) -> Duration {
    let reading = Reading {
        context: Redacted::new(context.to_vec()),
        lead: Redacted::default(),
        prefix: Redacted::new(prefix.to_owned()),
    };
    let query = Query {
        word: Redacted::new(prefix.to_owned()),
        readings: vec![reading],
    };
    let view = l.view(Some(pack));
    let start = Instant::now();
    let found = guesses(Some(pack), &view, &query, PLAIN, LIMIT, &SCORE);
    let spent = start.elapsed();
    assert!(!found.is_empty());
    spent
}

#[test]
#[ignore = "builds packs sized like the real ones; run with --release for the speed report"]
fn the_speed_report_times_guesses_with_a_full_learner() {
    for (iso, ranges) in LANGS {
        let abc: Vec<char> = ranges
            .iter()
            .flat_map(|&(first, last)| first..=last)
            .collect();
        let bytes = sized_pack(iso, &abc);
        let pack = Pack::parse(&bytes).unwrap();
        let words = sizes(iso).words;
        let cfg = LearnConfig {
            context_words: usize::from(MAX_CHAIN) - 1,
            ..CFG
        };
        let vocab: Vec<String> = (0..peak(cfg.max_words))
            .map(|n| fixture_word(&abc, if n % SEED_EVERY == 0 { n } else { words + n }))
            .collect();
        let start = Instant::now();
        let (l, order) = filled(iso, cfg, Some(&pack), &vocab);
        let learn = start.elapsed() / u32::try_from(order.len()).unwrap();
        let (empty, letter) = sample(&l, &pack, &vocab, &order);
        let chains: Vec<usize> = l.chains.iter().map(Chains::len).collect();
        eprintln!(
            "{iso}: pack {} B, {words} words; learner {} words, chains {chains:?}; empty p50/p99 {:?}; one letter p50/p99 {:?}; a learn {learn:?}; reconcile {:?}",
            bytes.len(),
            l.words.len(),
            percentiles(empty),
            percentiles(letter),
            reconciled(iso, cfg, &pack, &vocab),
        );
    }
}

/// The time `reconcile` takes on a learner with `cfg` filled from `vocab` without `pack`, then shown it.
fn reconciled(iso: &str, cfg: LearnConfig, pack: &Pack<'_>, vocab: &[String]) -> Duration {
    let (mut bare, _) = filled(iso, cfg, None, vocab);
    let start = Instant::now();
    bare.reconcile(Some(pack));
    start.elapsed()
}

/// The times of `SAMPLES` queries after runs of `order`, the words learned: an empty prefix, then the next word's
/// first letter.
fn sample(
    l: &Learner,
    pack: &Pack<'_>,
    vocab: &[String],
    order: &[usize],
) -> (Vec<Duration>, Vec<Duration>) {
    let width = l.cfg.context_words;
    let step = (order.len() - width - 1) / SAMPLES;
    let (mut empty, mut letter) = (Vec::new(), Vec::new());
    for at in (0..SAMPLES).map(|s| s * step) {
        let context: Vec<String> = order[at..at + width]
            .iter()
            .map(|&n| vocab[n].clone())
            .collect();
        let next = vocab[order[at + width]].chars().take(1).collect::<String>();
        empty.push(time(l, pack, &context, ""));
        letter.push(time(l, pack, &context, &next));
    }
    (empty, letter)
}
