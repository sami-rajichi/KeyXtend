//! The least time each P7 gate holds the mouse, from config; `fit::held_fits` adds the headroom and checks the guard.

use crate::config::GatesConfig;
use crate::copyend::END_WAITS;
use crate::fit::{times, total};
use crate::g8predict::RUNS;
use crate::g8words::READINGS;
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

/// The least time `g8 predict` holds the mouse: each copy with its warm-up and end, each typed word at its longest,
/// and the settle before each memory reading.
pub fn predict_ms(c: &GatesConfig) -> u64 {
    let (g, p) = (&c.g27, &c.g8_predict);
    let keys = g.cases.iter().map(|k| k.prefix.chars().count()).max();
    let word = word_ms(g, keys.unwrap_or_default());
    let words = times(p.warm_words.saturating_add(p.words), word);
    total(&[
        times(RUNS, copy_ms(c, c.g8.warm_ms)),
        words,
        times(READINGS, p.settle_ms),
    ])
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
