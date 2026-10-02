//! G2 no focus: random clicks on the product keyboard's keys; the target keeps focus and caret; click-to-character
//! time.
#![cfg(windows)]

use std::fmt::Write;

use serde_json::{Map, Value, json};

use crate::apps::{self, Ctx, Opened};
use crate::clicks::{self, Tally, Usable};
use crate::mouse;
use crate::out::say;
use crate::product::Keyboard;
use crate::stats::{self, Click};
use crate::tlog::{self, Unit};
use crate::win::sleep_ms;
use crate::{launch, out};

/// Suffix of the raw click file.
const CLICKS_FILE: &str = "-clicks.tsv";
/// First line of the raw click file.
const TSV_HEADER: &str = "index\tcode\tclick_us\tlatency_us\n";

/// What the target log shows after the clicks.
struct Logged {
    /// Units logged since target-window started.
    units: Vec<Unit>,
    /// Latency per click in µs; `None` when its character never came.
    lat: Vec<Option<i64>>,
    /// Times the text box lost the keyboard since the first click.
    focus_lost: usize,
    /// Why the log could not be read.
    error: Option<String>,
}

/// Brings `target` to the front and lists the uncovered keys it can type; cleans up on failure.
fn setup(ctx: &Ctx, kb: &Keyboard, target: Opened) -> Result<(Opened, Usable), String> {
    let target = launch::to_front(ctx, target)?;
    match clicks::usable(ctx, kb) {
        Ok(usable) => Ok((target, usable)),
        Err(e) => {
            apps::clean_up(ctx, target);
            Err(e)
        }
    }
}

/// Reads the target log: latencies, and focus losses since the first click.
fn measure(ctx: &Ctx, tally: &Tally) -> Logged {
    let path = &ctx.target.log;
    let (log, error) = match std::fs::read_to_string(path) {
        Ok(text) => (text, None),
        Err(e) => (
            String::new(),
            Some(format!("target log {}: {e}", path.display())),
        ),
    };
    let units = tlog::parse(&log);
    let window_us = ctx.cfg.g2.match_window_ms * stats::US_PER_MS_INT;
    let since = tally.clicks.first().map_or(i64::MAX, |c| c.us);
    Logged {
        lat: stats::latencies(&tally.clicks, &units, window_us),
        focus_lost: tlog::focus_lost(&log, since),
        units,
        error,
    }
}

/// Runs G2 on the product keyboard, called `name`, with `count` clicks from `seed`.
pub fn run(ctx: &Ctx, name: &str, count: usize, seed: u64) -> Result<Value, String> {
    let kb = Keyboard::find()?;
    let target = launch::start_target(ctx)?;
    on_target(ctx, name, &kb, target, (count, seed))
}

/// G2 on keyboard `kb` into `target`, already started (G26 starts it in a layout): `run.0` clicks from seed `run.1`.
/// The target is closed at the end.
pub fn on_target(
    ctx: &Ctx,
    name: &str,
    kb: &Keyboard,
    target: Opened,
    run: (usize, u64),
) -> Result<Value, String> {
    let place = match mouse::face(kb.hwnd) {
        Ok(place) => place,
        Err(e) => {
            apps::clean_up(ctx, target);
            return Err(e);
        }
    };
    // Keys another window already covers are left out and named, never clicked.
    let (target, usable) = setup(ctx, kb, target)?;
    let (count, seed) = run;
    say!(
        "G2 {name}: {count} clicks on {} keys, seed {seed}; keyboard dpi {}, client {:?}",
        usable.keys.len(),
        place.dpi,
        place.client
    );
    let tally = clicks::click_loop(ctx, &target, kb.hwnd, &usable.keys, run);
    sleep_ms(ctx.cfg.g2.drain_ms);
    let logged = measure(ctx, &tally);
    let left_open = apps::clean_up(ctx, target);
    let raw = ctx.file(CLICKS_FILE);
    out::write(&raw, &tsv(&tally.clicks, &logged.lat))?;
    let head = json!({
        "gate": "G2", "keyboard": name, "seed": seed, "planned": count, "dpi": place.dpi, "raw": raw,
        "hidden_keys": usable.hidden, "follow_ms": usable.follow_ms,
    });
    Ok(report(head, &tally, &logged, &left_open))
}

/// True when G2's result line `v` says every planned click typed its character and the target kept the focus and
/// its caret after each one (roadmap G2).
pub fn typed_intact(v: &Value) -> bool {
    let clicks = v["clicks"].as_u64().unwrap_or_default();
    let every = clicks > 0 && v["planned"].as_u64() == Some(clicks);
    let matched = v["matched"].as_u64() == Some(clicks);
    let caret = v["caret_kept"].as_u64() == Some(clicks);
    v.get("error").is_none() && every && matched && caret && v["focus_ok"] == true
}

/// True when the target kept the keyboard for every click and never lost it.
fn focus_ok(tally: &Tally, logged: &Logged) -> bool {
    let n = tally.clicks.len();
    n > 0 && tally.focus_kept == n && logged.focus_lost == 0
}

/// Prints the counts and latencies of the run.
fn print(tally: &Tally, logged: &Logged) {
    say!(
        "clicks {}; focus kept {}; caret kept {}; focus lost {}; chars logged {}",
        tally.clicks.len(),
        tally.focus_kept,
        tally.caret_kept,
        logged.focus_lost,
        logged.units.len()
    );
    let lat: Vec<String> = stats::summary_ms(&logged.lat)
        .iter()
        .map(|(name, ms)| format!("{name} {ms:?}"))
        .collect();
    let matched = logged.lat.iter().flatten().count();
    say!("latency ms {}; matched {matched}", lat.join(" "));
    let yes = |b: bool| if b { "yes" } else { "no" };
    let caret = tally.caret_kept == tally.clicks.len();
    say!(
        "focus ok: {}; caret ok: {}",
        yes(focus_ok(tally, logged)),
        yes(caret)
    );
    if tally.covered > 0 {
        say!(
            "skipped {} covered keys; covered by {}",
            tally.covered,
            tally.covered_by.join(", ")
        );
    }
    if let Some(s) = &tally.stopped {
        say!("stopped: {s}");
    }
    if let Some(e) = &logged.error {
        say!("{e}");
    }
}

/// Prints the run and adds its numbers and its verdict to `head`, the result line.
fn report(mut head: Value, tally: &Tally, logged: &Logged, left_open: &[String]) -> Value {
    print(tally, logged);
    let latency: Map<String, Value> = stats::summary_ms(&logged.lat)
        .iter()
        .map(|(name, ms)| (name.to_string(), json!(ms)))
        .collect();
    let body = json!({
        "clicks": tally.clicks.len(), "focus_kept": tally.focus_kept,
        "caret_kept": tally.caret_kept, "focus_lost": logged.focus_lost,
        "focus_ok": focus_ok(tally, logged), "units_logged": logged.units.len(),
        "chars_logged": tlog::decode(&logged.units).chars().count(),
        "matched": logged.lat.iter().flatten().count(), "latency_ms": latency,
        "stopped": tally.stopped, "failures": tally.failures, "left_open": left_open,
        "covered_skips": tally.covered, "covered_by": tally.covered_by,
    });
    if let (Some(all), Value::Object(more)) = (head.as_object_mut(), body) {
        all.extend(more);
    }
    if let Some(e) = &logged.error {
        head["error"] = json!(e);
    }
    head["pass"] = json!(typed_intact(&head));
    head
}

/// One line per click: index, scan code, click time and latency in µs.
fn tsv(clicks: &[Click], lat: &[Option<i64>]) -> String {
    let mut s = String::from(TSV_HEADER);
    for (i, (c, l)) in clicks.iter().zip(lat).enumerate() {
        let l = l.map(|v| v.to_string()).unwrap_or_default();
        // Writing to a `String` cannot fail.
        let _ = writeln!(s, "{i}\t{:#X}\t{}\t{l}", c.code, c.us);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(planned: u64, clicks: u64, matched: u64, focus_ok: bool) -> Value {
        json!({
            "gate": "G2", "planned": planned, "clicks": clicks, "matched": matched, "focus_ok": focus_ok,
            "caret_kept": clicks, "left_open": [],
        })
    }

    #[test]
    fn typing_is_intact_only_when_every_planned_click_matched_and_the_focus_stayed() {
        assert!(typed_intact(&run(100, 100, 100, true)));
        assert!(!typed_intact(&run(100, 100, 99, true)), "a lost character");
        assert!(
            !typed_intact(&run(100, 100, 100, false)),
            "the target lost the focus"
        );
        assert!(
            !typed_intact(&run(0, 0, 0, true)),
            "no click checks nothing"
        );
        assert!(
            !typed_intact(&run(100, 40, 40, true)),
            "the clicks stopped short"
        );
        let mut broke = run(5, 5, 5, true);
        broke["error"] = json!("target log");
        assert!(!typed_intact(&broke));
        let mut moved = run(100, 100, 100, true);
        moved["caret_kept"] = json!(99);
        assert!(!typed_intact(&moved), "the caret moved once");
    }

    /// A tally of `clicks` clicks after which the focus stayed every time and the caret `caret` times.
    fn tally(clicks: usize, caret: usize) -> Tally {
        let click = Click {
            code: 0x1E,
            us: 0,
            expect: vec![],
        };
        Tally {
            clicks: vec![click; clicks],
            focus_kept: clicks,
            caret_kept: caret,
            ..Default::default()
        }
    }

    #[test]
    fn the_result_line_carries_g2s_verdict_on_focus_caret_and_text() {
        let logged = |n: usize| Logged {
            units: Vec::new(),
            lat: vec![Some(10); n],
            focus_lost: 0,
            error: None,
        };
        let head = || json!({ "gate": "G2", "planned": 3 });
        let good = report(head(), &tally(3, 3), &logged(3), &[]);
        assert_eq!(good["pass"], true);
        let caret = report(head(), &tally(3, 2), &logged(3), &[]);
        assert_eq!(caret["pass"], false, "a lost caret fails G2 on its own");
        assert!(out::failed(&caret));
    }

    #[test]
    fn tsv_has_a_header_and_one_line_per_click() {
        let click = |code, us| Click {
            code,
            us,
            expect: vec![],
        };
        let got = tsv(&[click(0x1E, 100), click(0xE01D, 250)], &[Some(30), None]);
        let want = "index\tcode\tclick_us\tlatency_us\n0\t0x1E\t100\t30\n1\t0xE01D\t250\t\n";
        assert_eq!(got, want);
    }
}
