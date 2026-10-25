//! Property tests: random learns, forgets, undos and purges keep every table within 1.2 × its cap and within the cap
//! after each clean-up, an undo right after a forget restores the learner, and no byte form panics the reader.

use proptest::prelude::*;

use super::clean::peak;
use super::tests::word;
use super::*;

/// Random runs tried.
const CASES: u32 = 256;
/// The longest run of steps.
const MAX_STEPS: usize = 96;
/// The words the steps pick from, half of them seed words when a pack is built.
const VOCAB: usize = 16;
/// The highest cap the settings take, small so clean-ups happen often.
const MAX_CAP: usize = 12;
/// The highest `min_uses` the settings take.
const MAX_MIN_USES: u32 = 3;
/// The most days one step moves the clock.
const MAX_DAYS: u32 = 4;
/// A day number to start from.
const DAY: u32 = 20_000;

/// One thing that can happen to the learner.
#[derive(Debug, Clone)]
enum Step {
    /// Learn word `n`, with a first capital when set, after the words `context`, `days` later.
    Learn(usize, bool, Vec<usize>, u32),
    /// Forget word `n`.
    Forget(usize),
    /// Undo the last forget.
    Undo,
    /// Purge, `days` later.
    Purge(u32),
}

/// Any step.
fn step() -> impl Strategy<Value = Step> {
    let context = prop::collection::vec(0..VOCAB, 0..=usize::from(MAX_CHAIN));
    prop_oneof![
        8 => (0..VOCAB, any::<bool>(), context, 0..MAX_DAYS).prop_map(|(n, c, x, d)| Step::Learn(n, c, x, d)),
        2 => (0..VOCAB).prop_map(Step::Forget),
        2 => Just(Step::Undo),
        1 => (0..MAX_DAYS * MAX_DAYS).prop_map(Step::Purge),
    ]
}

/// Settings with small caps.
fn config() -> impl Strategy<Value = LearnConfig> {
    let chain = usize::from(MAX_CHAIN);
    (
        1..=MAX_MIN_USES,
        1..=MAX_CAP,
        1..=MAX_CAP,
        0..=MAX_DAYS * MAX_DAYS,
        1..chain,
    )
        .prop_map(|(u, w, c, f, x)| LearnConfig {
            min_uses: u,
            max_words: w,
            max_chains: c,
            forget_days: f,
            context_words: x,
        })
}

/// A pack holding the even words of the vocabulary, when the pack writer is built.
#[cfg(feature = "build")]
fn seed_bytes() -> Option<Vec<u8>> {
    use crate::pack::{Flags, PackWriter};
    let mut writer = PackWriter::new("en", Flags::default(), 0).ok()?;
    for n in (0..VOCAB).step_by(2) {
        writer.add_word(&word(n), u8::MAX).ok()?;
    }
    writer.finish().ok()
}

/// No pack without the pack writer.
#[cfg(not(feature = "build"))]
fn seed_bytes() -> Option<Vec<u8>> {
    None
}

/// Applies `step` on `day`; true when it was a forget that took something.
fn apply(l: &mut Learner, seed: Option<&Pack<'_>>, step: &Step, day: u32) -> bool {
    match step {
        Step::Learn(n, capital, context, _) => {
            let typed = word(*n);
            let typed = if *capital {
                l.casing().capital(&typed)
            } else {
                typed
            };
            let context: Vec<String> = context.iter().map(|&c| word(c)).collect();
            let context: Vec<&str> = context.iter().map(String::as_str).collect();
            l.learn(seed, &typed, &context, day);
            false
        }
        Step::Forget(n) => l.forget(seed, &word(*n), day),
        Step::Undo => {
            l.undo_forget();
            false
        }
        Step::Purge(_) => {
            l.purge(day);
            false
        }
    }
}

/// Every table's length: the words, then each chain table.
fn lengths(l: &Learner) -> Vec<usize> {
    let chains = l.chains.iter().map(Chains::len);
    std::iter::once(l.words.len()).chain(chains).collect()
}

/// Fails unless every table is within its peak, the blocklist within `max_words`, tables a clean-up shrank within
/// their caps, and every chain names a held word.
fn check_caps(l: &Learner, before: &[usize], learned: bool) -> Result<(), TestCaseError> {
    let after = lengths(l);
    let caps = std::iter::once(l.cfg.max_words).chain(l.chains.iter().map(|_| l.cfg.max_chains));
    let words_shrank = after.first() < before.first();
    for (at, ((&len, &was), cap)) in after.iter().zip(before).zip(caps).enumerate() {
        prop_assert!(len <= peak(cap), "table {at}: {len} past {}", peak(cap));
        let cleaned = learned && len < was && (at == 0 || !words_shrank);
        prop_assert!(
            !cleaned || len <= cap,
            "table {at}: {len} past {cap} after a clean-up"
        );
    }
    prop_assert!(l.blocked.len() <= l.cfg.max_words.max(1));
    let total: u64 = l.words.iter().map(|(_, w)| u64::from(w.count)).sum();
    prop_assert_eq!(l.words.total(), total);
    prop_assert!(
        l.blocked
            .iter()
            .all(|(text, _)| l.words.find(text).is_none())
    );
    for table in &l.chains {
        let mut keys = (0..table.len()).map(|at| table.entry(at).0);
        let held = |key: &[Id]| key.iter().all(|&id| l.words.get(id).is_some());
        prop_assert!(keys.all(held), "a chain names a word not held");
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn random_use_keeps_the_caps_and_an_undo_restores_the_forget(
        cfg in config(),
        steps in prop::collection::vec(step(), 0..MAX_STEPS),
    ) {
        let bytes = seed_bytes();
        let pack = bytes.as_deref().map(Pack::parse).transpose().ok().flatten();
        let mut l = Learner::new("en", cfg).unwrap();
        let (mut day, mut snapshot) = (DAY, None);
        for step in &steps {
            if let Step::Learn(.., days) | Step::Purge(days) = step {
                day += days;
            }
            let before = (lengths(&l), l.clone());
            let forgot = apply(&mut l, pack.as_ref(), step, day);
            check_caps(&l, &before.0, matches!(step, Step::Learn(..)))?;
            if let (Step::Undo, Some(was)) = (step, snapshot.take()) {
                prop_assert_eq!(&l, &was);
            }
            snapshot = forgot.then_some(before.1);
        }
        prop_assert_eq!(Learner::from_bytes(&l.to_bytes(), cfg), Ok(l.clone()));
        let bytes = l.to_bytes();
        for end in 0..bytes.len() {
            prop_assert!(Learner::from_bytes(&bytes[..end], cfg).is_err());
        }
    }

    #[test]
    fn random_or_altered_bytes_never_panic_the_reader(
        noise in prop::collection::vec(any::<u8>(), 0..MAX_STEPS),
        flips in prop::collection::vec((any::<prop::sample::Index>(), 1..=u8::MAX), 1..MAX_CAP),
    ) {
        let cfg = LearnConfig { min_uses: 2, max_words: MAX_CAP, max_chains: MAX_CAP, forget_days: 1, context_words: 2 };
        let _ = Learner::from_bytes(&noise, cfg);
        let mut l = Learner::new("en", cfg).unwrap();
        for n in 0..VOCAB {
            l.learn(None, &word(n), &[&word(n / 2)], DAY);
        }
        let mut headed = l.to_bytes();
        headed.truncate(MAGIC.len());
        headed.extend(VERSION.to_le_bytes());
        headed.extend(&noise);
        let _ = Learner::from_bytes(&headed, cfg);
        let mut bytes = l.to_bytes();
        for (at, flip) in &flips {
            let at = at.index(bytes.len());
            bytes[at] ^= flip;
        }
        if let Ok(back) = Learner::from_bytes(&bytes, cfg) {
            prop_assert_eq!(Learner::from_bytes(&back.to_bytes(), cfg), Ok(back));
        }
    }
}
