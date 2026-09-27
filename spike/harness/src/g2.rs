//! G2 no focus: random clicks on face keys; the target keeps focus and caret; click-to-character time.

use serde_json::{Map, Value, json};
use spike_core::clock::now_us;
use spike_core::window::foreground;
use spike_core::{layout, place};
use windows::Win32::Foundation::{HWND, POINT};

use crate::apps::{self, Ctx, Opened};
use crate::mouse::{self, Face};
use crate::rng::Rng;
use crate::stats::{self, Click};
use crate::tlog::{self, Unit};
use crate::win::{self, sleep_ms};
use crate::{launch, out};

/// Suffix of the raw click file.
const CLICKS_FILE: &str = "-clicks.tsv";
/// First line of the raw click file.
const TSV_HEADER: &str = "index\tcode\tclick_us\tlatency_us\n";

/// A clickable key: scan code, the character it types in the target's layout, screen centre.
struct Key {
    code: u32,
    text: String,
    at: POINT,
}

/// Counts and records of one click run.
#[derive(Default)]
struct Tally {
    clicks: Vec<Click>,
    focus_kept: usize,
    caret_kept: usize,
    failures: Vec<Value>,
    failed: usize,
    stopped: Option<String>,
    /// Clicks skipped because another window covered the key.
    covered: usize,
    /// The windows that covered keys, each named once.
    covered_by: Vec<String>,
}

impl Tally {
    /// Counts one click skipped because `who` covered the key; names at most `shown` windows.
    fn skip(&mut self, who: String, shown: usize) {
        self.covered += 1;
        if self.covered_by.len() < shown && !self.covered_by.contains(&who) {
            self.covered_by.push(who);
        }
    }
}

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

fn keys(ctx: &Ctx, face: &Face) -> Vec<Key> {
    let hkl = layout::foreground_layout();
    mouse::key_places(&ctx.spike.keyboard)
        .into_iter()
        .filter_map(|(code, place)| {
            let text = layout::character(code, hkl);
            layout::is_printable(&text).then(|| Key {
                code,
                text,
                at: mouse::centre(&place, face.origin, face.dpi),
            })
        })
        .collect()
}

/// Clicks one key; the character that arrives, not the cursor, shows where it landed.
fn click(key: &Key, tally: &mut Tally) -> Result<(), String> {
    let us = now_us();
    mouse::click_at(key.at)?;
    tally.clicks.push(Click {
        code: key.code,
        us,
        expect: key.text.encode_utf16().collect(),
    });
    Ok(())
}

/// Checks focus and caret after a click; `Err` stops the run.
fn check(
    ctx: &Ctx,
    target: &Opened,
    caret0: HWND,
    key: &Key,
    tally: &mut Tally,
) -> Result<(), String> {
    let (cfg, g) = (ctx.cfg, &ctx.cfg.g2);
    let front = foreground();
    let caret = win::caret(win::owner(target.hwnd).1);
    let (focus_ok, caret_ok) = (front == target.hwnd, caret == caret0);
    tally.focus_kept += usize::from(focus_ok);
    tally.caret_kept += usize::from(caret_ok);
    if focus_ok && caret_ok {
        return Ok(());
    }
    tally.failed += 1;
    if tally.failures.len() < g.failures_shown {
        tally.failures.push(json!({
            "index": tally.clicks.len() - 1, "key": format!("{:#X}", key.code), "label": key.text,
            "front": win::describe(front), "caret": win::describe(caret),
        }));
    }
    if !focus_ok && !win::front(target.hwnd, &cfg.timing, &cfg.keys) {
        return Err("target-window could not be brought back".to_string());
    }
    if tally.failed >= g.max_failures {
        return Err(format!("{} failures", tally.failed));
    }
    Ok(())
}

/// Clicks random keys of the face `face`; a key another window covers is skipped, never clicked.
fn click_loop(ctx: &Ctx, target: &Opened, face: HWND, keys: &[Key], run: (usize, u64)) -> Tally {
    let g = &ctx.cfg.g2;
    let caret0 = win::caret(win::owner(target.hwnd).1);
    let mut rng = Rng::new(run.1);
    let mut tally = Tally::default();
    while tally.clicks.len() < run.0 {
        let key = &keys[rng.below(keys.len())];
        let top = win::root_at(key.at);
        if top != face {
            tally.skip(win::describe(top), g.failures_shown);
            if tally.covered >= g.max_failures {
                tally.stopped = Some(format!("{} keys covered by other windows", tally.covered));
                break;
            }
            continue;
        }
        let step = click(key, &mut tally).and_then(|()| {
            sleep_ms(g.click_gap_ms);
            check(ctx, target, caret0, key, &mut tally)
        });
        if let Err(reason) = step {
            tally.stopped = Some(reason);
            break;
        }
    }
    tally
}

/// Splits keys into those on top and those another window covers, named by that window.
fn visible(keys: Vec<Key>, face: HWND) -> (Vec<Key>, Vec<String>) {
    let (shown, hidden): (Vec<Key>, Vec<Key>) =
        keys.into_iter().partition(|k| win::root_at(k.at) == face);
    let hidden = hidden
        .iter()
        .map(|k| format!("{:#X} under {}", k.code, win::describe(win::root_at(k.at))))
        .collect();
    (shown, hidden)
}

/// Starts target-window in front and lists the uncovered keys it can type; cleans up on failure.
fn setup(ctx: &Ctx, face: &Face, hwnd: HWND) -> Result<(Opened, Vec<Key>, Vec<String>), String> {
    let target = launch::start_target(ctx)?;
    if !win::front(target.hwnd, &ctx.cfg.timing, &ctx.cfg.keys) {
        apps::clean_up(ctx, target);
        return Err("could not bring target-window to the front".to_string());
    }
    let (keys, hidden) = visible(keys(ctx, face), hwnd);
    if keys.is_empty() {
        apps::clean_up(ctx, target);
        return Err(format!(
            "no printable key on top; covered: {}",
            hidden.join("; ")
        ));
    }
    if !hidden.is_empty() {
        println!(
            "left out {} covered keys: {}",
            hidden.len(),
            hidden.join("; ")
        );
    }
    Ok((target, keys, hidden))
}

/// Reads the target log: latencies, and focus losses since the first click.
fn measure(ctx: &Ctx, tally: &Tally) -> Logged {
    let path = ctx.spike.resolve(&ctx.spike.target.log);
    let (log, error) = match std::fs::read_to_string(&path) {
        Ok(text) => (text, None),
        Err(e) => (
            String::new(),
            Some(format!("target log {}: {e}", path.display())),
        ),
    };
    let units = tlog::parse(&log);
    let window_us = ctx.cfg.g2.match_window_ms * stats::US_PER_MS as i64;
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
    println!(
        "G2 {name}: {count} clicks on {} keys, seed {seed}; face dpi {}, client {:?}, block {block:?} logical",
        keys.len(),
        face.dpi,
        face.client
    );
    let tally = click_loop(ctx, &target, face_hwnd, &keys, (count, seed));
    sleep_ms(ctx.cfg.g2.drain_ms);
    let logged = measure(ctx, &tally);
    let left_open = apps::clean_up(ctx, target);
    let raw = ctx.file(CLICKS_FILE);
    out::write(&raw, &tsv(&tally.clicks, &logged.lat))?;
    let head = json!({
        "gate": "G2", "face": name, "seed": seed, "planned": count, "dpi": face.dpi, "raw": raw,
        "hidden_keys": hidden,
    });
    Ok(report(head, &tally, &logged, left_open))
}

/// True when the target kept the keyboard for every click and never lost it.
fn focus_ok(tally: &Tally, logged: &Logged) -> bool {
    let n = tally.clicks.len();
    n > 0 && tally.focus_kept == n && logged.focus_lost == 0
}

/// Prints the counts and latencies of the run.
fn print(tally: &Tally, logged: &Logged) {
    println!(
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
    println!("latency ms {}; matched {matched}", lat.join(" "));
    let ok = if focus_ok(tally, logged) { "yes" } else { "no" };
    println!("focus ok: {ok}");
    if tally.covered > 0 {
        println!(
            "skipped {} covered keys; covered by {}",
            tally.covered,
            tally.covered_by.join(", ")
        );
    }
    if let Some(s) = &tally.stopped {
        println!("stopped: {s}");
    }
    if let Some(e) = &logged.error {
        println!("{e}");
    }
}

/// Prints the run and adds its numbers to `head`, the result line.
fn report(mut head: Value, tally: &Tally, logged: &Logged, left_open: Vec<String>) -> Value {
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
        s.push_str(&format!("{i}\t{:#X}\t{}\t{l}\n", c.code, c.us));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covered_keys_are_counted_and_each_window_named_once() {
        let mut t = Tally::default();
        for who in ["osk", "osk", "menu", "tip"] {
            t.skip(who.to_string(), 2);
        }
        assert_eq!(t.covered, 4);
        assert_eq!(t.covered_by, vec!["osk".to_string(), "menu".to_string()]);
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
