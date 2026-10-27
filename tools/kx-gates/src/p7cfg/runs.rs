//! The least time each P7 gate holds the mouse, from config; `fit::held_fits` adds the headroom and checks the guard.

use crate::config::GatesConfig;
use crate::copyend::END_WAITS;
use crate::fit::{times, total};
use crate::g8predict::RUNS;
use crate::g8words::{Plan, READINGS};
use crate::g21prod::BOXES;
use crate::g27::COPIES;
use crate::p7cfg::G27;

/// One typed word at its longest: the switch, `keys` key clicks, the chip, the settle and the box's text.
fn word_ms(g: &G27, keys: usize) -> u64 {
    total(&[
        g.switch_wait_ms,
        times(keys, g.key_gap_ms),
        g.chip_wait_ms,
        g.settle_ms,
        g.text_wait_ms,
    ])
}

/// One churn: each of its words at its longest prefix, and the settle before each memory reading.
fn churn_ms(g: &G27, plan: &Plan, settle_ms: u64) -> u64 {
    let words = times(plan.words(), word_ms(g, plan.keys()));
    total(&[words, times(READINGS, settle_ms)])
}

/// The least time `g8 predict` holds the mouse: each copy with its warm-up and end, then the judged and the mixed
/// churn. A `lang` with no case adds no judged churn, as load refuses it first.
pub fn predict_ms(c: &GatesConfig) -> u64 {
    let (g, p) = (&c.g27, &c.g8_predict);
    let judged = Plan::judged(g, p).map_or(0, |k| churn_ms(g, &k, p.settle_ms));
    let mixed = churn_ms(g, &Plan::mixed(g, p), p.settle_ms);
    total(&[times(RUNS, copy_ms(c, c.g8.warm_ms)), judged, mixed])
}

/// The longest a copy's end may take: every Quit try, the wait for its end, then the wait after an end.
pub(crate) fn end_ms(c: &GatesConfig) -> u64 {
    let tries = u64::from(c.g8.quit_tries).saturating_mul(c.g8.quit_try_ms);
    total(&[tries, times(END_WAITS, c.timing.quit_wait_ms)])
}

/// One copy a gate starts: its start, the wait `warm_ms` until it is ready, and its end.
fn copy_ms(c: &GatesConfig, warm_ms: u64) -> u64 {
    total(&[c.g8.start_wait_ms, warm_ms, end_ms(c)])
}

/// The characters of `lang`'s case prefix; none for no case.
fn keys(g: &G27, lang: &str) -> usize {
    g.case(lang).map_or(0, |k| k.prefix.chars().count())
}

/// The least time G21's product half holds the mouse: its copy, then each box's switch, prefix and whole chip wait.
pub fn g21_ms(c: &GatesConfig) -> u64 {
    let g = &c.g27;
    let typed = times(keys(g, &c.g21_product.case), g.key_gap_ms);
    let boxed = total(&[g.switch_wait_ms, typed, g.chip_wait_ms]);
    total(&[copy_ms(c, g.ready_ms), times(BOXES, boxed)])
}

/// The least time G27 holds the mouse: its two copies, each case's word and edge 6's.
pub fn g27_ms(c: &GatesConfig) -> u64 {
    let g = &c.g27;
    let cases: Vec<u64> = g
        .cases
        .iter()
        .map(|k| word_ms(g, keys(g, &k.lang)))
        .collect();
    total(&[
        times(COPIES, copy_ms(c, g.ready_ms)),
        total(&cases),
        word_ms(g, keys(g, &g.edge)),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::load;
    use crate::fit::held_fits;

    /// The longest a copy's end takes: every Quit try and both end waits.
    fn end(c: &GatesConfig) -> u64 {
        u64::from(c.g8.quit_tries) * c.g8.quit_try_ms + 2 * c.timing.quit_wait_ms
    }

    /// One word of `keys` characters at its longest.
    fn word(g: &G27, keys: usize) -> u64 {
        let typed = keys as u64 * g.key_gap_ms;
        g.switch_wait_ms + typed + g.chip_wait_ms + g.settle_ms + g.text_wait_ms
    }

    /// The longest case prefix's characters.
    fn longest(g: &G27) -> usize {
        let all = g.cases.iter().map(|k| keys(g, &k.lang));
        all.max().expect("cases")
    }

    /// `g8 predict` worked out apart from `predict_ms`: both copies, the judged churn at its own prefix, the mixed
    /// churn at the longest, and a settle before each churn's two readings.
    fn predict(c: &GatesConfig) -> u64 {
        let (g, p) = (&c.g27, &c.g8_predict);
        let copy = c.g8.start_wait_ms + c.g8.warm_ms + end(c);
        let judged = (p.warm_words + p.words) as u64 * word(g, keys(g, &p.lang));
        let mixed = (g.cases.len() + p.mixed_words) as u64 * word(g, longest(g));
        2 * copy + judged + mixed + 2 * 2 * p.settle_ms
    }

    #[test]
    fn g21_and_g27_count_every_copy_and_every_word() {
        let c = load().expect("loads");
        let g = &c.g27;
        let copy = c.g8.start_wait_ms + g.ready_ms + end(&c);
        let typed = keys(g, &c.g21_product.case) as u64 * g.key_gap_ms;
        let boxed = g.switch_wait_ms + typed + g.chip_wait_ms;
        assert_eq!(g21_ms(&c), copy + 3 * boxed);
        let cases: u64 = g.cases.iter().map(|k| word(g, keys(g, &k.lang))).sum();
        assert_eq!(g27_ms(&c), 2 * copy + cases + word(g, keys(g, &g.edge)));
    }

    #[test]
    fn g8_predict_counts_both_copies_and_both_churns() {
        let c = load().expect("loads");
        let g = &c.g27;
        assert_eq!(predict_ms(&c), predict(&c));
        let short = g.cases.iter().min_by_key(|k| keys(g, &k.lang));
        let short = short.expect("cases");
        assert!(keys(g, &short.lang) < longest(g), "a shorter prefix");
        let mut other = c.clone();
        other.g8_predict.lang.clone_from(&short.lang);
        assert_eq!(predict_ms(&other), predict(&other), "its own prefix");
        let mut more = c.clone();
        more.g8_predict.mixed_words += 1;
        assert_eq!(predict_ms(&more), predict_ms(&c) + word(g, longest(g)));
    }

    #[test]
    fn todays_g8_predict_fits_the_mouse_guard() {
        let c = load().expect("loads");
        let need = predict_ms(&c) + c.fit.guard_headroom_ms;
        assert!(c.mouse_guard.on, "the shipped guard is on");
        assert!(need <= c.mouse_guard.limit_ms, "{need} ms");
        assert_eq!(held_fits(&c, "g8 predict", predict_ms(&c)), Ok(()));
    }
}
