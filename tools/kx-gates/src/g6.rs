//! G6 scroll on the product's test build: its pad scrolls Notepad, Chrome and Explorer at the last point outside the
//! keyboard, while the simulated user's pointer rests on the pad. It judges the aim, both speeds, every stop and the
//! end, and records what P5 asked to know; `g6rec` takes the target-window records.
#![cfg(windows)]

use kx_mod_scroll::ScrollSettings;
use kx_module_api::ports::Dir;
use serde_json::{Value, json};
use spike_core::uia::Uia;

use crate::apps::{AppKind, Ctx};
use crate::g5::RCLICK;
use crate::g6cases::{self, Case};
use crate::g6judge::{self, Pair};
use crate::g6rows::Grown;
use crate::g6scene::Scene;
use crate::out::say;
use crate::probe::{self, Doc};
use crate::scroll::along;
use crate::scrolllog::{self, ScrollLine};
use crate::testbuild::{Closed, TestBuild};
use crate::{g6rec, g6rows};
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
    /// Explorer's files a row and how the grown folder settled.
    grown: Option<Grown>,
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

/// Opens the app's long document: Explorer's folder, sized for the test build's `speeds`, else the numbered lines.
fn open(ctx: &Ctx, kind: AppKind, speeds: (u32, u32)) -> Result<Doc, String> {
    if kind == AppKind::Explorer {
        return g6rows::open(ctx, speeds);
    }
    let pr = &ctx.cfg.probes;
    let text = probetext::scroll_text(pr.scroll_lines, pr.scroll_cols);
    probe::open_text(ctx, kind, &text)
}

/// Every case on `kind`'s document, in order, with the ratio `want` at the test build's `speeds`, and the records
/// taken on the way; Explorer's folder grows first to the files a row shows.
fn drive(
    ctx: &Ctx,
    uia: &Uia,
    (doc, kind): (&Doc, AppKind),
    (want, speeds): (f64, (u32, u32)),
) -> Result<Driven, String> {
    probe::front(ctx, &doc.app)?;
    let s = Scene::new(ctx, uia, &doc.app)?;
    let vertical = || s.pair().and_then(|p| along(Dir::Down, p));
    let grown = if kind == AppKind::Explorer {
        Some(g6rows::fill(ctx, uia, doc, speeds, vertical)?)
    } else {
        None
    };
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
        grown,
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
    let (want, speeds) = (wanted(&scroll)?, (scroll.slow, scroll.fast));
    let doc = open(ctx, kind, speeds)?;
    say!("G6 {name}: {}", win::describe(doc.app.hwnd));
    let result = probe::run_on(ctx, doc, |d| drive(ctx, &uia, (d, kind), (want, speeds)));
    let closed = build.close();
    let (driven, notes) = result.map_err(|e| closed.failed(&e, &[]))?;
    Ok(report(name, &driven, &notes, &counts(&log), &closed))
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
    let short = g6cases::sizing_faults(&d.cases);
    if !short.is_empty() {
        say!("G6 {name}: sizing fault, the pad met the document's end in {short:?}");
    }
    say!("G6 {name}: pass {pass}; stops {stops}; {}", json!(cases));
    json!({
        "gate": "G6", "app": name, "pass": pass, "cases": cases, "stops_counted": stops,
        "scroll_stats": line, "scroll_stats_error": counts.as_ref().err(), "outside_clicks": d.clicks,
        "end_percents": { "top": d.top, "bottom": d.bottom }, "percent_lag_ms": d.lag_ms,
        "routing": g6rec::routing(), "files_per_row": d.grown.and_then(|g| g.per_row),
        "grown_settled": d.grown.and_then(|g| g.settled), "sizing_faults": short, "clean_up": notes,
        "test_build": closed.json(),
    })
}
