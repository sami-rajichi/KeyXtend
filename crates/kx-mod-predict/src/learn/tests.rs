//! The learner: the two-use gate, capitals, chains and their context; the pack's spellings live in `seed`.

use kx_module_api::Redacted;
use kx_module_api::ports::TypingMode;

use super::*;
use crate::score::{Learned, ScoreConfig, guesses};
use crate::track::{Query, Reading};

mod bytes;
mod clean;
mod forget;
mod load;
#[cfg(feature = "build")]
mod seed;
mod size;
#[cfg(feature = "build")]
mod speed;

/// A day number, as the module counts them.
pub(crate) const DAY: u32 = 20_000;
/// The settings' defaults as Task 12 fills them.
pub(crate) const CFG: LearnConfig = LearnConfig {
    min_uses: 2,
    max_words: 10_000,
    max_chains: 30_000,
    forget_days: 300,
    context_words: 2,
};
/// The scorer's defaults as Task 12 fills them.
const SCORE: ScoreConfig = ScoreConfig {
    backoff: 0.4,
    learn_weight: 0.6,
    learn_ramp: 2000,
    min_uses: 2,
};
/// No latch.
const PLAIN: TypingMode = TypingMode {
    shift: false,
    caps: false,
    shortcut: false,
};
/// The chips a test asks for.
const LIMIT: usize = 5;

/// An empty learner for `iso` with the default settings.
fn learner(iso: &str) -> Learner {
    Learner::new(iso, CFG).unwrap()
}

/// The `n`th test word, in letters only, as a learned word holds no digit; the property tests share it.
pub(super) fn word(n: usize) -> String {
    let letters: Vec<char> = ('a'..='z').collect();
    let (mut n, mut out) = (n, String::from("w"));
    loop {
        out.push(letters[n % letters.len()]);
        n /= letters.len();
        if n == 0 {
            return out;
        }
    }
}

/// Learns each word of `text` after the words before it, at most `context_words`, as the tracker hands them out.
fn teach(l: &mut Learner, seed: Option<&Pack<'_>>, text: &str, day: u32) {
    let words: Vec<&str> = text.split_whitespace().collect();
    for (at, word) in words.iter().enumerate() {
        let from = at.saturating_sub(l.cfg.context_words);
        l.learn(seed, word, &words[from..at], day);
    }
}

/// The texts guessed for `prefix` after `context`.
fn guessed(l: &Learner, seed: Option<&Pack<'_>>, context: &[&str], prefix: &str) -> Vec<String> {
    let reading = Reading {
        context: Redacted::new(context.iter().map(|w| (*w).to_owned()).collect()),
        lead: Redacted::default(),
        prefix: Redacted::new(prefix.to_owned()),
    };
    let query = Query {
        word: Redacted::new(prefix.to_owned()),
        readings: vec![reading],
    };
    let found = guesses(l.iso(), seed, &l.view(seed), &query, PLAIN, LIMIT, &SCORE);
    found.iter().map(|g| g.text.expose().clone()).collect()
}

/// Each chain table's length, shortest chains first.
fn lengths(l: &Learner) -> Vec<usize> {
    l.chains.iter().map(Chains::len).collect()
}

/// The learned words `starting` lists for `folded_prefix`.
fn listed(l: &Learner, folded_prefix: &str) -> Vec<String> {
    let mut out = Vec::new();
    l.view(None)
        .starting(folded_prefix, &mut |w| out.push(w.to_owned()));
    out
}

/// The learned words `following` lists after `context`.
fn followers(l: &Learner, seed: Option<&Pack<'_>>, context: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    l.view(seed)
        .following(context, &mut |w| out.push(w.to_owned()));
    out
}

#[test]
fn a_min_uses_past_the_headroom_is_refused() {
    let most = LearnConfig {
        min_uses: MAX_MIN_USES,
        ..CFG
    };
    assert!(Learner::new("en", most).is_ok());
    let over = LearnConfig {
        min_uses: MAX_MIN_USES + 1,
        ..CFG
    };
    let refused = Some(LearnFormatError::BadMinUses);
    assert_eq!(Learner::new("en", over).err(), refused);
    assert_eq!(
        Learner::from_bytes(&learner("en").to_bytes(), over).err(),
        refused
    );
}

#[test]
fn one_use_is_not_offered_and_two_are() {
    let mut l = learner("en");
    l.learn(None, "zork", &[], DAY);
    assert_eq!(l.view(None).count("zork"), 1);
    assert!(guessed(&l, None, &[], "zo").is_empty());
    l.learn(None, "zork", &[], DAY);
    assert_eq!(guessed(&l, None, &[], "zo"), ["zork"]);
    assert_eq!(l.view(None).uses(), 2);
}

#[test]
fn a_name_keeps_its_capital_and_a_sentence_start_does_not_split_a_word() {
    let mut l = learner("en");
    teach(&mut l, None, "Zork zork Sami Sami", DAY);
    assert_eq!(listed(&l, "zo"), ["zork"]);
    assert_eq!(listed(&l, "sa"), ["Sami"]);
    let view = l.view(None);
    assert_eq!((view.count("zork"), view.count("Zork")), (2, 2));
    let mut lower_first = learner("en");
    teach(&mut lower_first, None, "zork Zork ZORK", DAY);
    assert_eq!(listed(&lower_first, "zo"), ["zork"]);
    assert_eq!(lower_first.view(None).count("zork"), 3);
}

#[test]
fn words_are_keyed_with_the_language_casing() {
    let mut tr = learner("tr");
    teach(&mut tr, None, "Ilık ılık", DAY);
    assert_eq!(listed(&tr, "ıl"), ["ılık"]);
    assert!(listed(&tr, "il").is_empty());
    assert_eq!(tr.view(None).count("ılık"), 2);
    let mut en = learner("en");
    teach(&mut en, None, "ılık", DAY);
    assert_eq!(listed(&en, "il"), ["ılık"]);
}

#[test]
fn chains_count_each_context_and_stop_at_an_unknown_word() {
    let mut l = learner("en");
    teach(&mut l, None, "in my park", DAY);
    let view = l.view(None);
    assert_eq!(view.chain(&["in"], "my"), 1);
    assert_eq!(view.chain(&["in", "my"], "park"), 1);
    assert_eq!(followers(&l, None, &["in", "my"]), ["park"]);
    l.learn(None, "park", &["unread", "my"], DAY);
    l.learn(None, "park", &["my", "unread"], DAY);
    let view = l.view(None);
    assert_eq!(view.chain(&["my"], "park"), 2);
    assert_eq!(view.chain(&["unread", "my"], "park"), 0);
    assert_eq!(view.count("unread"), 0);
    assert_eq!(view.count("park"), 3);
}

#[test]
fn context_past_context_words_is_cut() {
    let mut l = learner("en");
    teach(&mut l, None, "a b c", DAY);
    l.learn(None, "d", &["a", "b", "c"], DAY);
    let view = l.view(None);
    assert_eq!(view.chain(&["c"], "d"), 1);
    assert_eq!(view.chain(&["b", "c"], "d"), 1);
    assert_eq!(view.chain(&["a", "b", "c"], "d"), 0);
    assert!(followers(&l, None, &["a", "b", "c"]).is_empty());
    assert_eq!(followers(&l, None, &["c"]), ["d"]);
}

#[test]
fn by_count_lists_the_words_of_a_prefix_most_used_first() {
    let mut l = learner("en");
    teach(&mut l, None, "park pat pat pan pat pan zoo", DAY);
    let view = l.view(None);
    let ranked: Vec<(&str, u32)> = view.by_count("pa").collect();
    assert_eq!(ranked, [("pat", 3), ("pan", 2), ("park", 1)]);
    assert_eq!(view.by_count("").count(), 4);
    assert_eq!(view.by_count("q").count(), 0);
}
