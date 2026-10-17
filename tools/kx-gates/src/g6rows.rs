//! G6's Explorer folder: enough files that no case meets its end on the way, counted in rows, as an icon view shows
//! several files a row; the files a row shows are read from the first items' boxes once the window is laid.
#![cfg(windows)]

use std::cell::Cell;

use kx_module_api::MS_PER_S;
use serde::Serialize;
use spike_core::uia::{self, Uia};

use crate::apps::Ctx;
use crate::p5cfg::G6;
use crate::probe::{self, Doc};
use crate::win;

/// Files for the folder at `per_row` a row: `spare` rows past the lines the pad runs down at `(slow, fast)` lines a
/// second through G6's speeds, up and press-stop cases, each tap's `settle_ms` included.
pub fn files(
    spare: usize,
    per_row: usize,
    (slow, fast): (u32, u32),
    g: &G6,
    settle_ms: u64,
) -> usize {
    let both = u64::from(slow.saturating_add(fast)).saturating_mul(settle_ms + g.phase_ms);
    let again = u64::from(slow).saturating_mul(settle_ms + g.move_ms);
    let lines = both.saturating_add(again).div_ceil(u64::from(MS_PER_S));
    let rows = spare.saturating_add(usize::try_from(lines).unwrap_or(usize::MAX));
    rows.saturating_mul(per_row.max(1))
}

/// How many files the first row shows, from the items' `tops` in order: those level with the first; none from fewer
/// than two tops, as one file alone cannot tell.
pub fn per_row(tops: &[i32]) -> Option<usize> {
    let [first, _, ..] = tops else {
        return None;
    };
    Some(tops.iter().take_while(|&top| top == first).count())
}

/// True when `now` reads the same percent as the read before it, which `last` holds and then takes `now` for.
fn same_twice(last: &Cell<Option<f64>>, now: Option<f64>) -> bool {
    let before = last.replace(now);
    matches!((before, now), (Some(a), Some(b)) if (a - b).abs() <= f64::EPSILON)
}

/// What growing the folder found: the files a row shows, none when fewer than two files read, and whether the
/// vertical percent then read the same twice in time, none when nothing grew or no percent reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Grown {
    /// The files a row shows.
    pub per_row: Option<usize>,
    /// The list's extent stopped growing within `[g6] grow_wait_ms`.
    pub settled: Option<bool>,
}

/// File number `i`'s name, as the folder's files sort in order.
fn name(prefix: &str, i: usize) -> String {
    format!("{prefix}{i:04}")
}

/// Opens Explorer on a folder sized for one file a row at the test build's `speeds`.
pub fn open(ctx: &Ctx, speeds: (u32, u32)) -> Result<Doc, String> {
    let pr = &ctx.cfg.probes;
    let n = files(
        pr.scroll_files,
        1,
        speeds,
        &ctx.cfg.g6,
        ctx.cfg.pad.settle_ms,
    );
    let names: Vec<String> = (1..=n).map(|i| name(&pr.scroll_prefix, i)).collect();
    probe::open_folder(ctx, &names, &[])
}

/// The tops of the first files `doc` shows, in order, up to the first one off the first row.
fn tops(ctx: &Ctx, uia: &Uia, doc: &Doc, n: usize) -> Vec<i32> {
    let prefix = &ctx.cfg.probes.scroll_prefix;
    let mut tops = Vec::new();
    for i in 1..=n {
        let item = uia.item(doc.app.hwnd, &name(prefix, i));
        let Some(top) = item.and_then(|el| uia::rect(&el).ok()).map(|r| r.top) else {
            break;
        };
        let level = tops.first().is_none_or(|&first| first == top);
        tops.push(top);
        if !level {
            break;
        }
    }
    tops
}

/// Reads how many files a row of the laid window shows, adds files until the folder holds enough at that many a row,
/// or at `[g6] max_per_row` when it cannot tell, then waits for the `vertical` percent to settle.
pub fn fill(
    ctx: &Ctx,
    uia: &Uia,
    doc: &Doc,
    speeds: (u32, u32),
    vertical: impl Fn() -> Option<f64>,
) -> Result<Grown, String> {
    let (pr, g, settle) = (&ctx.cfg.probes, &ctx.cfg.g6, ctx.cfg.pad.settle_ms);
    let had = files(pr.scroll_files, 1, speeds, g, settle);
    let per_row = per_row(&tops(ctx, uia, doc, had));
    let want = files(
        pr.scroll_files,
        per_row.unwrap_or(g.max_per_row),
        speeds,
        g,
        settle,
    );
    for i in had + 1..=want {
        let path = doc.path.join(name(&pr.scroll_prefix, i));
        std::fs::File::create(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    let settled = (want > had).then(|| steady(g, vertical)).flatten();
    Ok(Grown { per_row, settled })
}

/// Waits up to `grow_wait_ms` for `vertical` to read the same percent twice, `sample_ms` apart; none when it reads
/// no percent, as nothing then rests on the extent.
fn steady(g: &G6, vertical: impl Fn() -> Option<f64>) -> Option<bool> {
    vertical()?;
    let last = Cell::new(None);
    let held = win::poll_until(g.grow_wait_ms, g.sample_ms, || {
        same_twice(&last, vertical()).then_some(())
    });
    Some(held.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g6() -> G6 {
        let mut g = crate::config::load().expect("loads").g6;
        (g.phase_ms, g.move_ms) = (3_000, 1_500);
        g
    }

    #[test]
    fn the_folder_outlasts_every_run_down_before_the_end_case() {
        let (g, spare, settle) = (g6(), 80, 500);
        // Slow and fast through settle and phase, 13 × 3.5 s, and one more slow run of settle and move, 3 × 2 s.
        assert_eq!(files(spare, 1, (3, 10), &g, settle), spare + 52);
        assert_eq!(files(spare, 1, (0, 0), &g, settle), spare, "nothing runs");
    }

    #[test]
    fn an_icon_view_needs_as_many_more_files_as_a_row_shows() {
        let (g, spare, settle) = (g6(), 80, 500);
        assert_eq!(files(spare, 4, (3, 10), &g, settle), 4 * (spare + 52));
    }

    #[test]
    fn a_row_holds_the_files_level_with_the_first() {
        assert_eq!(per_row(&[10, 40]), Some(1), "a list: one a row");
        assert_eq!(per_row(&[10, 10, 10, 90]), Some(3), "an icon view");
        assert_eq!(per_row(&[10, 10]), Some(2), "the read stopped on the row");
    }

    #[test]
    fn fewer_than_two_files_read_leave_the_row_unknown() {
        assert_eq!(per_row(&[]), None, "no file read");
        assert_eq!(per_row(&[10]), None, "the second file never read");
    }

    #[test]
    fn the_percent_has_settled_once_it_reads_the_same_twice_in_a_row() {
        let last = Cell::new(None);
        let reads = [None, Some(10.0), Some(12.0), Some(12.0), None, None];
        let held: Vec<bool> = reads.into_iter().map(|r| same_twice(&last, r)).collect();
        assert_eq!(held, [false, false, false, true, false, false]);
    }
}
