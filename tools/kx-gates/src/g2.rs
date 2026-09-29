//! G2 no focus: random clicks on face keys; the target keeps focus and caret; click-to-character time.
#![cfg(windows)]

use std::fmt::Write;

use serde_json::{Map, Value, json};
use spike_core::place;
use windows::Win32::Foundation::HWND;

use crate::apps::{self, Ctx, Opened};
use crate::clicks::{self, Key, Tally};
use crate::mouse::{self, Face};
use crate::out::say;
use crate::stats::{self, Click};
use crate::tlog::{self, Unit};
use crate::win::{self, sleep_ms};
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

/// Starts target-window in front and lists the uncovered keys it can type; cleans up on failure.
fn setup(ctx: &Ctx, face: &Face, hwnd: HWND) -> Result<(Opened, Vec<Key>, Vec<String>), String> {
    let target = launch::start_target(ctx)?;
    if !win::front(target.hwnd, &ctx.cfg.timing, &ctx.cfg.keys) {
        apps::clean_up(ctx, target);
        return Err("could not bring target-window to the front".to_string());
    }
    match clicks::usable(ctx, face, hwnd) {
        Ok((keys, hidden)) => Ok((target, keys, hidden)),
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

/// Runs G2 on the face called `name` with `count` clicks from `seed`.
pub fn run(ctx: &Ctx, name: &str, count: usize, seed: u64) -> Result<Value, String> {
    let face_hwnd = mouse::find_face(ctx.spike, name)?;
    let face = mouse::face(face_hwnd)?;
    // Keys another window already covers are left out and named, never clicked.
    let (target, keys, hidden) = setup(ctx, &face, face_hwnd)?;
    let block = place::block_size(&ctx.spike.keyboard);
    say!(
        "G2 {name}: {count} clicks on {} keys, seed {seed}; face dpi {}, client {:?}, block {block:?} logical",
        keys.len(),
        face.dpi,
        face.client
    );
    let tally = clicks::click_loop(ctx, &target, face_hwnd, &keys, (count, seed));
    sleep_ms(ctx.cfg.g2.drain_ms);
    let logged = measure(ctx, &tally);
    let left_open = apps::clean_up(ctx, target);
    let raw = ctx.file(CLICKS_FILE);
    out::write(&raw, &tsv(&tally.clicks, &logged.lat))?;
    let head = json!({
        "gate": "G2", "face": name, "seed": seed, "planned": count, "dpi": face.dpi, "raw": raw,
        "hidden_keys": hidden,
    });
    Ok(report(head, &tally, &logged, &left_open))
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
    let ok = if focus_ok(tally, logged) { "yes" } else { "no" };
    say!("focus ok: {ok}");
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

/// Prints the run and adds its numbers to `head`, the result line.
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
