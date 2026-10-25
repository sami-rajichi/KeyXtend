//! The speed figure (`--speed`): `guesses` on each real pack with a learner past its caps and the longest context,
//! timed for an empty prefix after a word and for one letter, with the scorer's word sort buffer per query.

use std::cmp::Reverse;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::hint::black_box;
use std::time::Instant;

use kx_mod_predict::learn::{LearnConfig, Learner};
use kx_mod_predict::pack::layout::MAX_CHAIN;
use kx_mod_predict::score::{Learned, ScoreConfig, guesses};
use kx_mod_predict::track::{Query, Reading};
use kx_mod_predict::{Pack, PredictSettings, WordId, fold};
use kx_module_api::Redacted;
use kx_module_api::ports::TypingMode;

use super::tally::percentile;
use super::{DAY, Paths, each_lang};
use crate::words::WordsError;
use crate::words::manifest::MeterRules;

/// The query kinds timed, with the letters of the next word each types: none, then one.
const KINDS: [(&str, usize); 2] = [("empty prefix", 0), ("one letter", 1)];

/// What every query of one language is asked against.
struct Bench<'a> {
    /// The language code.
    iso: &'a str,
    /// Its pack.
    pack: &'a Pack<'a>,
    /// The full learner, as the scorer reads it.
    learned: &'a dyn Learned,
    /// The scorer's settings.
    score: ScoreConfig,
    /// The chips asked for.
    chips: usize,
    /// The learned words.
    vocab: &'a [&'a str],
    /// The order they were learned in, as places in `vocab`.
    order: &'a [usize],
    /// The words of context before each query.
    width: usize,
}

/// Times each language with held sentences, printing one line for each, and gives each query kind over the ceiling.
pub(crate) fn run(
    rules: &MeterRules,
    settings: &PredictSettings,
    paths: &Paths,
    asked: &[String],
    say: &mut dyn FnMut(&str),
) -> Result<Vec<String>, WordsError> {
    let mut slow = Vec::new();
    each_lang(paths, asked, &mut |iso, pack, _| {
        say(&timed(iso, pack, settings, rules, &mut slow)?);
        Ok(())
    })?;
    Ok(slow)
}

/// The line of `iso`: its full learner's sizes and each query kind's times, adding to `slow` each kind over the
/// ceiling.
fn timed(
    iso: &str,
    pack: &Pack<'_>,
    settings: &PredictSettings,
    rules: &MeterRules,
    slow: &mut Vec<String>,
) -> Result<String, WordsError> {
    let cfg = LearnConfig {
        context_words: usize::from(MAX_CHAIN) - 1,
        ..settings.learn_config()
    };
    let vocab = vocab(pack, cfg.max_words);
    let (learner, order) = filled(iso, pack, cfg, &vocab, rules.speed_learns)?;
    let view = learner.view(Some(pack));
    let bench = Bench {
        iso,
        pack,
        learned: &view,
        score: settings.score_config(),
        chips: usize::from(settings.chips),
        vocab: &vocab,
        order: &order,
        width: cfg.context_words,
    };
    let mut parts = vec![format!("{iso} speed: {learner:?}")];
    for (kind, letters) in KINDS {
        let (times, buffer) = bench.sample(letters, rules.speed_samples);
        parts.push(kind_line(kind, &times, buffer, rules));
        slow.extend(over(iso, kind, &times, rules));
    }
    Ok(parts.join("; "))
}

/// The line that says every query kind stayed under the ceiling.
pub(crate) fn passed(rules: &MeterRules) -> String {
    let p = rules.percentiles.last().copied().unwrap_or_default();
    format!(
        "speed passed: every query kind's p{p} is at most {} us",
        rules.ceiling_us
    )
}

impl Bench<'_> {
    /// The time of `samples` queries spread over the learned order, each typing `letters` letters of the next word
    /// after the words before it, and the largest sort buffer one took.
    fn sample(&self, letters: usize, samples: usize) -> (Vec<u64>, usize) {
        let step = (self.order.len().saturating_sub(self.width) / samples.max(1)).max(1);
        let (mut times, mut buffer) = (Vec::new(), 0);
        for at in (self.width..self.order.len()).step_by(step).take(samples) {
            let word = |n: &usize| self.vocab.get(*n).copied().unwrap_or_default();
            let context = self.order.get(at - self.width..at).unwrap_or_default();
            let context: Vec<String> = context.iter().map(|n| word(n).to_owned()).collect();
            let next = self.order.get(at).map(word).unwrap_or_default();
            let prefix: String = next.chars().take(letters).collect();
            times.push(self.time(context, &prefix));
            buffer = buffer.max(self.buffer(&prefix));
        }
        (times, buffer)
    }

    /// The time `guesses` takes for `prefix` after `context`, in microseconds.
    fn time(&self, context: Vec<String>, prefix: &str) -> u64 {
        let reading = Reading {
            context: Redacted::new(context),
            lead: Redacted::default(),
            prefix: Redacted::new(prefix.to_owned()),
        };
        let query = Query {
            word: Redacted::new(prefix.to_owned()),
            readings: vec![reading],
        };
        let (pack, mode) = (Some(self.pack), TypingMode::default());
        let start = Instant::now();
        let found = guesses(
            self.iso,
            pack,
            self.learned,
            &query,
            mode,
            self.chips,
            &self.score,
        );
        let spent = start.elapsed();
        drop(black_box(found));
        u64::try_from(spent.as_micros()).unwrap_or(u64::MAX)
    }

    /// The bytes of the word sort `guesses` makes for `prefix`: one id per pack word the prefix starts.
    fn buffer(&self, prefix: &str) -> usize {
        let span = self.pack.span(&fold(prefix, self.pack.casing()));
        let words = usize::try_from(span.end.saturating_sub(span.start)).unwrap_or(usize::MAX);
        words.saturating_mul(size_of::<WordId>())
    }
}

/// The pack's `n` most frequent words, as most of a user's learned words would be.
fn vocab<'p>(pack: &Pack<'p>, n: usize) -> Vec<&'p str> {
    let mut ids: Vec<WordId> = (0..pack.word_count()).collect();
    ids.sort_by_key(|&id| (Reverse(pack.freq(id)), id));
    let words = ids.into_iter().map(|id| pack.spelling(id));
    words.filter(|w| !w.is_empty()).take(n).collect()
}

/// A learner for `iso` after `learns` words of `vocab`, each after the words before it: every word once in order,
/// then a repeatable spread. Gives the order as places in `vocab`.
fn filled(
    iso: &str,
    pack: &Pack<'_>,
    cfg: LearnConfig,
    vocab: &[&str],
    learns: usize,
) -> Result<(Learner, Vec<usize>), WordsError> {
    let mut learner =
        Learner::new(iso, cfg).map_err(|e| WordsError::Meter(format!("{iso} learner: {e}")))?;
    let mut order: Vec<usize> = Vec::with_capacity(learns);
    for n in 0..learns {
        let next = if n < vocab.len() {
            n
        } else {
            spread(n, vocab.len())
        };
        let from = order.len().saturating_sub(cfg.context_words);
        let before = order.get(from..).unwrap_or_default();
        let context: Vec<&str> = before
            .iter()
            .filter_map(|&at| vocab.get(at).copied())
            .collect();
        if let Some(word) = vocab.get(next) {
            learner.learn(Some(pack), word, &context, DAY);
            order.push(next);
        }
    }
    Ok((learner, order))
}

/// A repeatable place for step `n` among `len`: the standard hasher of `n`, so no constant of our own is needed.
fn spread(n: usize, len: usize) -> usize {
    let mut hasher = DefaultHasher::new();
    n.hash(&mut hasher);
    let len = u64::try_from(len).unwrap_or(u64::MAX).max(1);
    usize::try_from(hasher.finish() % len).unwrap_or_default()
}

/// One query kind's part of the line: each percentile and the largest sort buffer.
fn kind_line(kind: &str, times: &[u64], buffer: usize, rules: &MeterRules) -> String {
    let parts: Vec<String> = (rules.percentiles.iter())
        .map(|&p| format!("p{p} {} us", percentile(times, p)))
        .collect();
    format!("{kind} {}, sort buffer up to {buffer} B", parts.join(", "))
}

/// Why `iso`'s `kind` failed: its highest percentile over the ceiling; none when it stayed under.
fn over(iso: &str, kind: &str, times: &[u64], rules: &MeterRules) -> Option<String> {
    let p = rules.percentiles.last().copied().unwrap_or_default();
    let took = percentile(times, p);
    (took > rules.ceiling_us).then(|| {
        format!(
            "{iso} {kind}: p{p} {took} us, over the ceiling of {} us",
            rules.ceiling_us
        )
    })
}
