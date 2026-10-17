//! G6 scroll on the product's test build: its pad scrolls Notepad, Chrome and Explorer at the last point outside the
//! keyboard, while the simulated user's pointer rests on the pad. It judges the aim, both speeds, every stop and the
//! end, and records what P5 asked to know; `g6rec` takes the target-window records.
#![cfg(windows)]

use kx_mod_scroll::ScrollSettings;
use kx_module_api::MS_PER_S;
use serde_json::{Value, json};
use spike_core::uia::Uia;

use crate::apps::{AppKind, Ctx};
use crate::g5::RCLICK;
use crate::g6cases::{self, Case};
use crate::g6judge::{self, Pair};
use crate::g6rec;
use crate::g6scene::Scene;
use crate::out::say;
use crate::p5cfg::G6;
use crate::probe::{self, Doc};
use crate::scrolllog::{self, ScrollLine};
use crate::testbuild::{Closed, TestBuild};
use crate::{probetext, win};

/// The apps G6 probes; never Word (roadmap G6, owner, 2026-10-16).
pub const APPS: [AppKind; 3] = [AppKind::Notepad, AppKind::Chrome, AppKind::Explorer];
/// The apps whose document always overflows sideways, so sideways is always judged: Chrome shows the long lines in a
/// `<pre>`, unwrapped.
const WIDE: [AppKind; 1] = [AppKind::Chrome];

/// What one app's run gave: its cases, its records, and how many clicks it made outside our windows.
struct Driven {
    cases: Vec<Case>,
    top: Pair,
    bottom: Pair,
    lag_ms: Option<u64>,
    clicks: u64,
}

/// The fast ÷ slow ratio G6 judges against, once the stops it judges are on in the test build's settings.
fn wanted(s: &ScrollSettings) -> Result<f64, String> {
    if !(s.stop_at_end && s.stop_on_press) {
        return Err(
            "G6 needs the test build's [scroll] stop_at_end and stop_on_press on".to_string(),
        );
    }
    Ok(f64::from(s.fast) / f64::from(s.slow))
}

/// Opens the app's long document: Explorer's folder, sized for the test build's speeds `s`, else the numbered lines.
fn open(ctx: &Ctx, kind: AppKind, s: &ScrollSettings) -> Result<Doc, String> {
    let pr = &ctx.cfg.probes;
    if kind == AppKind::Explorer {
        let n = files(
            pr.scroll_files,
            (s.slow, s.fast),
            &ctx.cfg.g6,
            ctx.cfg.pad.settle_ms,
        );
        let files: Vec<String> = (1..=n)
            .map(|i| format!("{}{i:04}", pr.scroll_prefix))
            .collect();
        return probe::open_folder(ctx, &files, &[]);
    }
    let text = probetext::scroll_text(pr.scroll_lines, pr.scroll_cols);
    probe::open_text(ctx, kind, &text)
}

/// Every case on `kind`'s document, in order, and the records taken on the way.
fn drive(ctx: &Ctx, uia: &Uia, (doc, kind): (&Doc, AppKind), want: f64) -> Result<Driven, String> {
    probe::front(ctx, &doc.app)?;
    let s = Scene::new(ctx, uia, &doc.app)?;
    let top = s.pair();
    let lag_ms = g6rec::lag(&s)?;
    let mut cases = g6cases::speeds(&s, want)?;
    cases.extend(g6cases::up_and_stop(&s)?);
    cases.push(g6cases::press_stop(&s)?);
    cases.push(g6cases::sideways(&s, WIDE.contains(&kind))?);
    let (end, bottom) = g6cases::end(&s)?;
    cases.push(end);
    let clicks = s.clicks();
    Ok(Driven {
        cases,
        top,
        bottom,
        lag_ms,
        clicks,
    })
}

/// The test build's scroll counts since its start, read after it quit.
fn counts(log: &(std::path::PathBuf, i64)) -> Result<Option<ScrollLine>, String> {
    let (path, since) = log;
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    scrolllog::since(&text, *since)
}

/// Runs G6 on the app called `name`, or takes the target-window record called so, under the test build with
/// Right-click off; an app passes when every judged case did, its stops counted right and the test build quit.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    if g6rec::NAMES.contains(&name) {
        return g6rec::run(ctx, name);
    }
    let kind = probe::kind_of(name, &APPS)?;
    let uia = Uia::new()?;
    let build = TestBuild::open(ctx, None)?;
    build.set(RCLICK, false)?;
    let (scroll, log) = (build.scroll()?, build.log()?);
    let want = wanted(&scroll)?;
    let doc = open(ctx, kind, &scroll)?;
    say!("G6 {name}: {}", win::describe(doc.app.hwnd));
    let result = probe::run_on(ctx, doc, |d| drive(ctx, &uia, (d, kind), want));
    let closed = build.close();
    let (driven, notes) = result.map_err(|e| closed.failed(&e, &[]))?;
    Ok(report(name, &driven, &notes, &counts(&log), &closed))
}

/// Files for Explorer's folder: `spare` past the lines the pad runs down at `(slow, fast)` lines a second through
/// G6's speeds, up and press-stop cases, each tap's `settle_ms` included, so no case meets the end on the way.
fn files(spare: usize, (slow, fast): (u32, u32), g: &G6, settle_ms: u64) -> usize {
    let both = u64::from(slow.saturating_add(fast)).saturating_mul(settle_ms + g.phase_ms);
    let again = u64::from(slow).saturating_mul(settle_ms + g.move_ms);
    let lines = both.saturating_add(again).div_ceil(u64::from(MS_PER_S));
    spare.saturating_add(usize::try_from(lines).unwrap_or(usize::MAX))
}

/// The app's result line.
fn report(
    name: &str,
    d: &Driven,
    notes: &[String],
    counts: &Result<Option<ScrollLine>, String>,
    closed: &Closed,
) -> Value {
    let verdicts: Vec<Option<bool>> = d.cases.iter().map(|c| c.ok).collect();
    let line = counts.as_ref().ok().and_then(Option::as_ref);
    let stops = line.is_some_and(|l| g6judge::stops_ok(l, d.clicks));
    let pass = closed.pass(g6judge::pass(&verdicts) && stops);
    let cases: Vec<Value> = d.cases.iter().map(Case::json).collect();
    say!("G6 {name}: pass {pass}; stops {stops}; {}", json!(cases));
    json!({
        "gate": "G6", "app": name, "pass": pass, "cases": cases, "stops_counted": stops,
        "scroll_stats": line, "scroll_stats_error": counts.as_ref().err(), "outside_clicks": d.clicks,
        "end_percents": { "top": d.top, "bottom": d.bottom }, "percent_lag_ms": d.lag_ms,
        "routing": g6rec::routing(), "clean_up": notes, "test_build": closed.json(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explorers_folder_outlasts_every_run_down_before_the_end_case() {
        let mut g = crate::config::load().expect("loads").g6;
        (g.phase_ms, g.move_ms) = (3_000, 1_500);
        let (spare, settle) = (80, 500);
        // Slow and fast through settle and phase, 13 × 3.5 s, and one more slow run of settle and move, 3 × 2 s.
        assert_eq!(files(spare, (3, 10), &g, settle), spare + 52);
        assert_eq!(files(spare, (0, 0), &g, settle), spare, "nothing runs");
    }
}
