//! G6's Explorer folder: enough files that no case meets its end on the way, counted in rows, as an icon view shows
//! several files a row; the files a row shows are read from the first items' boxes once the window is laid, and the
//! grown list's extent from UI Automation's vertical view size.
#![cfg(windows)]

use std::cell::Cell;

use kx_module_api::MS_PER_S;
use kx_platform_windows::uia::{self, Uia};
use kx_target_window::fill::NUMBER;
use serde::Serialize;

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

/// True when `a` and `b` read the same.
fn same(a: f64, b: f64) -> bool {
    (a - b).abs() <= f64::EPSILON
}

/// The view size the grown list should show: `before`, read at `had` files, scaled to `want` files, as the size is the
/// share of the list shown.
#[allow(
    clippy::cast_precision_loss,
    reason = "File counts are far below 2^52."
)]
fn grown_size(before: Option<f64>, had: usize, want: usize) -> Option<f64> {
    before.map(|b| b * had as f64 / want.max(1) as f64)
}

/// True when the view size has settled after the folder grew: `now` reads as the read before it, which `last` holds
/// and then takes `now` for, and lies within `slack` of `aim`, the grown size, as a share of it, when that is known.
fn settled(aim: Option<f64>, slack: f64, last: &Cell<Option<f64>>, now: Option<f64>) -> bool {
    let was = last.replace(now);
    let twice = matches!((was, now), (Some(a), Some(b)) if same(a, b));
    let near = aim.is_none_or(|a| now.is_some_and(|n| (n - a).abs() <= a * slack));
    twice && near
}

/// What growing the folder found: the files a row shows, none when fewer than two files read, and whether the
/// vertical view size then settled in time, none when nothing grew.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Grown {
    /// The files a row shows.
    pub per_row: Option<usize>,
    /// The list's extent stopped growing within `[g6] grow_wait_ms`.
    pub settled: Option<bool>,
}

/// File number `i`'s name, as the folder's files sort in order: numbered as target-window's fill lines.
fn name(prefix: &str, i: usize) -> String {
    format!("{prefix}{i:0NUMBER$}")
}

/// The files the folder needs at `per_row` a row at the test build's `speeds`, with `ctx`'s settings.
fn needed(ctx: &Ctx, per_row: usize, speeds: (u32, u32)) -> usize {
    let (spare, g, settle) = (
        ctx.cfg.probes.scroll_files,
        &ctx.cfg.g6,
        ctx.cfg.pad.settle_ms,
    );
    files(spare, per_row, speeds, g, settle)
}

/// Opens Explorer on a folder sized for one file a row at the test build's `speeds`.
pub fn open(ctx: &Ctx, speeds: (u32, u32)) -> Result<Doc, String> {
    let prefix = &ctx.cfg.probes.scroll_prefix;
    let names: Vec<String> = (1..=needed(ctx, 1, speeds))
        .map(|i| name(prefix, i))
        .collect();
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
/// or at `[g6] max_per_row` when it cannot tell, then waits for the `view` size to settle at the grown extent.
pub fn fill(
    ctx: &Ctx,
    uia: &Uia,
    doc: &Doc,
    speeds: (u32, u32),
    view: impl Fn() -> Option<f64>,
) -> Result<Grown, String> {
    let g = &ctx.cfg.g6;
    let had = needed(ctx, 1, speeds);
    let per_row = per_row(&tops(ctx, uia, doc, had));
    let want = needed(ctx, per_row.unwrap_or(g.max_per_row), speeds);
    let aim = grown_size(view(), had, want);
    for i in had + 1..=want {
        let path = doc.path.join(name(&ctx.cfg.probes.scroll_prefix, i));
        std::fs::File::create(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    let settled = (want > had).then(|| steady(g, aim, view));
    Ok(Grown { per_row, settled })
}

/// Waits up to `grow_wait_ms`, reading `view` every `sample_ms` from the start, for it to settle within `ratio_slack`
/// of `aim`; false when it never did or never read.
fn steady(g: &G6, aim: Option<f64>, view: impl Fn() -> Option<f64>) -> bool {
    let last = Cell::new(None);
    let held = win::poll_until(g.grow_wait_ms, g.sample_ms, || {
        settled(aim, g.ratio_slack, &last, view()).then_some(())
    });
    held.is_some()
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

    /// A fixture `[g6] ratio_slack`.
    const SLACK: f64 = 0.3;
    /// A list of 100 files that showed 40 % of itself, grown to 400 files: it should show 10 %.
    const GROWN: (Option<f64>, usize, usize) = (Some(40.0), 100, 400);

    /// Whether each of `reads` found the view size settled at `aim`.
    fn settles(aim: Option<f64>, reads: &[Option<f64>]) -> Vec<bool> {
        let last = Cell::new(None);
        reads
            .iter()
            .map(|&r| settled(aim, SLACK, &last, r))
            .collect()
    }

    #[test]
    fn the_grown_size_is_the_size_before_scaled_by_the_files() {
        let (before, had, want) = GROWN;
        assert_eq!(grown_size(before, had, want), Some(10.0));
        assert_eq!(grown_size(None, had, want), None, "no size before");
    }

    #[test]
    fn the_view_size_has_settled_once_it_reads_the_same_twice_near_the_grown_size() {
        let aim = grown_size(GROWN.0, GROWN.1, GROWN.2);
        let reads = [Some(40.0), Some(40.0), Some(12.0), Some(9.0), Some(9.0)];
        assert_eq!(
            settles(aim, &reads),
            [false, false, false, false, true],
            "the list had not grown at first"
        );
        let unread = [None, None, Some(9.0), None];
        assert_eq!(settles(aim, &unread), [false; 4]);
    }

    #[test]
    fn a_pause_between_explorers_batches_is_no_settle() {
        let aim = grown_size(GROWN.0, GROWN.1, GROWN.2);
        let reads = [Some(25.0), Some(25.0), Some(10.0), Some(10.0)];
        assert_eq!(settles(aim, &reads), [false, false, false, true]);
    }

    #[test]
    fn with_no_size_before_the_files_were_added_two_same_reads_after_settle_it() {
        let reads = [None, Some(9.0), Some(9.0)];
        assert_eq!(settles(None, &reads), [false, false, true]);
    }

    #[test]
    fn file_names_are_numbered_as_target_windows_fill_lines() {
        assert_eq!(name("kx-scroll-", 7), "kx-scroll-0007");
        assert_eq!(name("f", 12_345), "f12345", "a longer number stays whole");
    }
}
