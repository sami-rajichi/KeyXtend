//! G6's Explorer folder: enough files that no case meets its end on the way, counted in rows, as an icon view shows
//! several files a row; the files a row shows are read from the first items' boxes once the window is laid.
#![cfg(windows)]

use kx_module_api::MS_PER_S;
use spike_core::uia::{self, Uia};

use crate::apps::Ctx;
use crate::p5cfg::G6;
use crate::probe::{self, Doc};
use crate::win::sleep_ms;

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

/// How many files the first row shows, from the items' `tops` in order: those level with the first; at least one.
pub fn per_row(tops: &[i32]) -> usize {
    let level = tops.first().map_or(0, |&first| {
        tops.iter().take_while(|&&top| top == first).count()
    });
    level.max(1)
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

/// Reads how many files a row of the laid window shows, and adds files until the folder holds enough at that many a
/// row; the files a row shows.
pub fn fill(ctx: &Ctx, uia: &Uia, doc: &Doc, speeds: (u32, u32)) -> Result<usize, String> {
    let (pr, settle) = (&ctx.cfg.probes, ctx.cfg.pad.settle_ms);
    let had = files(pr.scroll_files, 1, speeds, &ctx.cfg.g6, settle);
    let row = per_row(&tops(ctx, uia, doc, had));
    let want = files(pr.scroll_files, row, speeds, &ctx.cfg.g6, settle);
    for i in had + 1..=want {
        let path = doc.path.join(name(&pr.scroll_prefix, i));
        std::fs::File::create(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    if want > had {
        sleep_ms(pr.settle_ms);
    }
    Ok(row)
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
        assert_eq!(per_row(&[10, 40]), 1, "a list: one a row");
        assert_eq!(per_row(&[10, 10, 10, 90]), 3, "an icon view");
        assert_eq!(per_row(&[10, 10]), 2, "the read stopped on the row");
        assert_eq!(per_row(&[]), 1, "no item read");
    }
}
