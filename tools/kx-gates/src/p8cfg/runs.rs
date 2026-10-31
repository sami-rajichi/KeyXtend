//! The least time each P8 gate holds the mouse, from config; `fit::held_fits` adds the headroom and checks the guard.

use kx_platform_windows::clip;

use crate::config::GatesConfig;
use crate::fit::{times, total};
use crate::g8clip::READINGS;
use crate::g20prod::COPIES;
use crate::p7cfg::runs::end_ms;

/// G20's clipboard writes besides its plain and marked copies: the cleared copy, its clear and the skipped app's copy.
const MORE_WRITES: usize = 3;
/// G20's panel waits: open by the clip key, shut by Insert, open by Win and V, and shut again.
const PANEL_WAITS: usize = 4;
/// G20's pauses besides its searches' and ticks': after the panel opens, and before the Win and V check looks.
const MORE_SETTLES: usize = 2;
/// The waits of one Win and V part: for a window, for it in front, and for it to close (`g25wins::closes`).
const HISTORY_WAITS: usize = 3;
/// A G20 search's pauses after its typing: before the read, and after the clear.
const SEARCH_PAUSES: usize = 2;
/// A `g8 clip` round's panel waits: open, then shut.
const ROUND_WAITS: usize = 2;
/// A `g8 clip` round's pauses after its search: after the typing, and after the clear.
const ROUND_PAUSES: usize = 2;

/// One copy a gate starts: its start, the wait `warm_ms` until it is ready, and its end.
fn copy_ms(c: &GatesConfig, warm_ms: u64) -> u64 {
    total(&[c.g8.start_wait_ms, warm_ms, end_ms(c)])
}

/// `text` typed with G27's typist, then `pauses` pauses of `settle_ms`: the switch and a click per character.
fn typed_ms(c: &GatesConfig, text: &str, pauses: usize, settle_ms: u64) -> u64 {
    let g = &c.g27;
    let keys = times(text.chars().count(), g.key_gap_ms);
    total(&[g.switch_wait_ms, keys, times(pauses, settle_ms)])
}

/// The least time G20 on the product holds the mouse: its two copies, its writes, its searches and ticks, the panel
/// waits, both Win and V parts and the read-back.
pub fn g20_ms(c: &GatesConfig) -> u64 {
    let g = &c.g20_product;
    let writes = g.copies.len() + clip::markers().len() + MORE_WRITES;
    let search = |text: &str| typed_ms(c, text, SEARCH_PAUSES, g.settle_ms);
    let searches: Vec<u64> = g.searches.iter().map(|s| search(&s.typed)).collect();
    let ticks = g.searches.iter().filter(|s| s.tick).count();
    total(&[
        times(COPIES, copy_ms(c, g.ready_ms)),
        times(writes, g.copy_gap_ms),
        total(&searches),
        times(ticks + MORE_SETTLES, g.settle_ms),
        times(PANEL_WAITS, g.panel_wait_ms),
        times(COPIES * HISTORY_WAITS, c.g25.switch_wait_ms),
        c.timing.read_wait_ms,
    ])
}

/// The least time `g8 clip` holds the mouse: its copy after G8's warm-up, every fill copy, the pause before each
/// reading, and every round's panel waits, page turns and search.
pub fn clip_ms(c: &GatesConfig) -> u64 {
    let (g, p) = (&c.g8_clip, &c.g20_product);
    let round = total(&[
        times(ROUND_WAITS, p.panel_wait_ms),
        times(g.pages, p.settle_ms),
        typed_ms(c, &g.search, ROUND_PAUSES, p.settle_ms),
    ]);
    total(&[
        copy_ms(c, c.g8.warm_ms),
        times(g.entries, g.copy_gap_ms),
        times(READINGS + g.rounds, g.settle_ms),
        times(g.warm_rounds + g.rounds, round),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::load;
    use crate::fit::held_fits;

    #[test]
    fn todays_p8_runs_fit_the_mouse_guard() {
        let c = load().expect("loads");
        assert!(c.mouse_guard.on, "the shipped guard is on");
        assert_eq!(held_fits(&c, "g20", g20_ms(&c)), Ok(()));
        assert_eq!(held_fits(&c, "g8 clip", clip_ms(&c)), Ok(()));
    }

    #[test]
    fn each_more_copy_round_or_search_letter_counts() {
        let c = load().expect("loads");
        let mut more = c.clone();
        more.g8_clip.entries += 1;
        assert_eq!(clip_ms(&more), clip_ms(&c) + c.g8_clip.copy_gap_ms);
        let mut longer = c.clone();
        longer.g20_product.searches[0].typed.push('x');
        assert_eq!(g20_ms(&longer), g20_ms(&c) + c.g27.key_gap_ms);
        let mut round = c.clone();
        round.g8_clip.rounds += 1;
        assert!(clip_ms(&round) > clip_ms(&c) + c.g8_clip.settle_ms);
    }
}
