//! The early stop against the full scan: on random packs, learners and queries, both give the same guesses.

use proptest::prelude::*;

use super::*;
use crate::learn::Learner;
use crate::learn::tests::{CFG as LEARN, DAY};

/// Random runs tried.
const CASES: u32 = 256;
/// The letters words are made of: few, so words share prefixes.
const LETTERS: [char; 3] = ['a', 'b', 'c'];
/// The words of up to three letters.
const VOCAB: usize = 39;
/// The most pack words or chains one run makes.
const MOST: usize = 32;
/// The most learns one run makes.
const MOST_LEARNS: usize = MOST * 2;
/// The most words a pack chain's context holds: as many as the learner keeps.
const MOST_CONTEXT: usize = LEARN.context_words;
/// The most words a query's context holds: one more than the learner keeps.
const QUERY_CONTEXT: usize = LEARN.context_words + 1;
/// The most words one run forgets.
const MOST_FORGETS: usize = 2;
/// Typed prefixes: up to two letters, a capital among them.
const PREFIX: &str = "[aAbc]{0,2}";
/// The learned ramps tried: none, short, and the default.
const RAMPS: [u32; 3] = [0, 10, CFG.learn_ramp];
/// The highest `min_uses` tried.
const MOST_GATE: u32 = 3;
/// The language of every run.
const ISO: &str = "en";

/// Word `n` of the vocabulary, with a first capital when `capital` is set.
fn word(n: usize, capital: bool) -> String {
    let base = LETTERS.len();
    let (mut n, mut out) = (n % VOCAB, String::new());
    loop {
        out.push(LETTERS[n % base]);
        if n < base {
            break;
        }
        n = n / base - 1;
    }
    if capital {
        Casing::of(ISO).capital(&out)
    } else {
        out
    }
}

/// One random run: pack words and chains, learns, forgets, and a query.
#[derive(Debug, Clone)]
struct Run {
    /// Pack words: vocabulary place, capital, frequency.
    words: Vec<(usize, bool, u8)>,
    /// Pack chains: context places and next place among the pack words, score.
    chains: Vec<(Vec<usize>, usize, u8)>,
    /// Learns: vocabulary place, capital, context places, with the pack.
    learns: Vec<(usize, bool, Vec<usize>, bool)>,
    /// Forgotten vocabulary places.
    forgets: Vec<usize>,
    /// The query's context places, prefix, Caps Lock, Shift and chip limit.
    query: (Vec<usize>, String, bool, bool, usize),
    /// The scorer's ramp and gate.
    score: (u32, u32),
}

/// Any run.
fn run_of() -> impl Strategy<Value = Run> {
    let words = prop::collection::vec((0..VOCAB, any::<bool>(), 1..=u8::MAX), 1..MOST);
    let chain = (
        prop::collection::vec(0..MOST, 1..=MOST_CONTEXT),
        0..MOST,
        1..=u8::MAX,
    );
    let learn = (
        0..VOCAB,
        any::<bool>(),
        prop::collection::vec(0..VOCAB, 0..=LEARN.context_words),
        any::<bool>(),
    );
    let score = (prop::sample::select(RAMPS.to_vec()), 1..=MOST_GATE);
    (
        words,
        prop::collection::vec(chain, 0..MOST),
        prop::collection::vec(learn, 0..MOST_LEARNS),
    )
        .prop_flat_map(move |(w, c, l)| {
            (
                Just((w, c, l)),
                prop::collection::vec(0..VOCAB, 0..=MOST_FORGETS),
                query_of(),
                score.clone(),
            )
        })
        .prop_map(|((words, chains, learns), forgets, query, score)| Run {
            words,
            chains,
            learns,
            forgets,
            query,
            score,
        })
}

/// Any query: context places, prefix, Caps Lock, Shift and chip limit.
fn query_of() -> impl Strategy<Value = (Vec<usize>, String, bool, bool, usize)> {
    (
        prop::collection::vec(0..VOCAB, 0..=QUERY_CONTEXT),
        PREFIX,
        any::<bool>(),
        any::<bool>(),
        1..=LIMIT,
    )
}

/// The pack of `run`'s words and chains.
fn pack_of(run: &Run) -> Vec<u8> {
    let spelt: Vec<String> = run.words.iter().map(|&(n, c, _)| word(n, c)).collect();
    let words: Vec<(&str, u8)> = spelt
        .iter()
        .zip(&run.words)
        .map(|(s, w)| (s.as_str(), w.2))
        .collect();
    let at = |n: usize| spelt[n % spelt.len()].as_str();
    let contexts: Vec<Vec<&str>> = run
        .chains
        .iter()
        .map(|c| c.0.iter().map(|&n| at(n)).collect())
        .collect();
    let chains: Vec<(&[&str], &str, u8)> = (contexts.iter().zip(&run.chains))
        .map(|(x, c)| (x.as_slice(), at(c.1), c.2))
        .collect();
    pack(ISO, &words, &chains)
}

/// The learner `run` makes, reconciled with `pack`; no run comes near the default caps.
fn learner_of(run: &Run, pack: &Pack<'_>) -> Learner {
    let mut l = Learner::new(ISO, LEARN).unwrap();
    for (n, capital, context, seeded) in &run.learns {
        let context: Vec<String> = context.iter().map(|&c| word(c, false)).collect();
        let context: Vec<&str> = context.iter().map(String::as_str).collect();
        l.learn(seeded.then_some(pack), &word(*n, *capital), &context, DAY);
    }
    for &n in &run.forgets {
        l.forget(Some(pack), &word(n, false), DAY);
    }
    l.reconcile(Some(pack));
    l
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn the_early_stop_gives_what_the_full_scan_gives(run in run_of()) {
        let bytes = pack_of(&run);
        let pack = Pack::parse(&bytes).unwrap();
        let l = learner_of(&run, &pack);
        let (context, prefix, caps, shift, limit) = &run.query;
        let context: Vec<String> = context.iter().map(|&c| word(c, false)).collect();
        let context: Vec<&str> = context.iter().map(String::as_str).collect();
        let query = ask(&context, prefix);
        let mode = TypingMode { shift: *shift, caps: *caps, shortcut: false, dead: false };
        let cfg = ScoreConfig { learn_ramp: run.score.0, min_uses: run.score.1, ..CFG };
        for seed in [Some(&pack), None] {
            let view = l.view(seed);
            let by = |scan| {
                let ask = Ask { scan, ..Ask::new(l.iso(), seed, &view, &query, mode, &cfg) };
                texts(&ask.guesses(*limit))
            };
            prop_assert_eq!(by(Scan::Ranked), by(Scan::Every));
        }
    }
}
