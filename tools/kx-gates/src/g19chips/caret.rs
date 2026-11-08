//! The caret's Paste (P10 design "At a caret"): a click on the empty line, with the clipboard holding the cut line,
//! shows Paste alone; it goes on our keyboard's key, on a press anywhere, on a caret step the app makes, and after
//! `caret_s`. A double click's brief Paste is recorded, never failed.
#![cfg(windows)]

use std::time::Instant;

use kx_mod_keyboard::Nav;
use serde_json::{Value, json};
use windows::Win32::Foundation::POINT;

use crate::apps::Opened;
use crate::board;
use crate::g19chips::docs::Line;
use crate::g19chips::gate::Gate;
use crate::g19chips::notepad::Lines;
use crate::g19chips::plan::on_time;
use crate::g19chips::strip::{Shown, ids};
use crate::g23::live::poll_read;
use crate::p10cfg::caret_ms;
use crate::win::ms_since;

/// The acts the step takes: four clicks on the caret line, our key, a press and its release, the app's caret step, a
/// double click and our Esc.
pub const ACTS: usize = 10;

/// True when `s` shows the caret's keycaps alone.
fn caret_like(gate: &Gate, s: &Shown) -> bool {
    ids(&s.chips(), gate.g()) == gate.g().caret_chips
}

/// The caret's Paste once a click on the empty line shows it.
fn shows(gate: &Gate, app: &Opened, lines: &Lines) -> Result<Option<Shown>, String> {
    gate.acts.click(app, lines.caret())?;
    Ok(gate.strip.shows()?.filter(|s| caret_like(gate, s)))
}

/// True once the caret's Paste went within `gone_wait_ms`.
fn goes(gate: &Gate) -> Result<bool, String> {
    gate.strip.goes(|s| caret_like(gate, s))
}

/// The Paste times out: it goes `caret_s` after it showed, within `caret_slack_ms`.
fn timer(gate: &Gate, app: &Opened, lines: &Lines) -> Result<Value, String> {
    let g = gate.g();
    let Some(s) = shows(gate, app, lines)? else {
        return Ok(json!({ "ok": false, "error": "no Paste" }));
    };
    let (r, p) = (s.rect, lines.caret());
    let (shown_at, stays_ms) = (Instant::now(), caret_ms(&gate.sel));
    let longest = stays_ms
        .saturating_add(g.caret_slack_ms)
        .saturating_add(g.gone_wait_ms);
    let poll = gate.ctx.cfg.timing.poll_ms;
    let gone = || {
        let now = gate.strip.now()?;
        Ok(now.is_none_or(|s| !caret_like(gate, &s)).then_some(()))
    };
    let after = poll_read(longest, poll, gone)?.map(|()| ms_since(shown_at));
    let ok = after.is_some_and(|ms| on_time(ms, stays_ms, g.caret_slack_ms));
    let near = json!({ "strip": [r.left, r.top, r.right, r.bottom], "click": [p.x, p.y] });
    Ok(json!({ "ok": ok, "went_ms": after, "caret_ms": stays_ms, "caret_note": near }))
}

/// A double click on the copy line: every strip seen, and whether a brief Paste showed before the word's strip.
fn double(gate: &Gate, app: &Opened, lines: &Lines) -> Result<Value, String> {
    let g = gate.g();
    let (from, to) = (lines.start(Line::Copy), lines.end(Line::Copy));
    let p = POINT {
        x: i32::midpoint(from.x, to.x),
        y: from.y,
    };
    gate.acts.double(app, p)?;
    let seen = gate.strip.seen(g.strip_wait_ms)?;
    let named: Vec<Vec<String>> = seen.iter().map(|c| ids(c, g)).collect();
    let brief = named.contains(&g.caret_chips);
    gate.acts.key(&board::nav_key(Nav::Esc)?)?;
    Ok(json!({ "seen": named, "brief_paste": brief }))
}

/// The caret's Paste, each way it goes; the double click is a note.
pub fn run(gate: &Gate, app: &Opened, lines: &Lines) -> Result<Value, String> {
    let g = gate.g();
    let key = match shows(gate, app, lines)? {
        Some(_) => gate
            .acts
            .key(&board::nav_key(Nav::Right)?)
            .and_then(|()| goes(gate))?,
        None => false,
    };
    let click = match shows(gate, app, lines)? {
        Some(_) => {
            let at = lines.end(Line::Copy);
            gate.acts.down(app, at)?;
            let went = goes(gate);
            gate.acts.up(at)?;
            went?
        }
        None => false,
    };
    let moved = match shows(gate, app, lines)? {
        Some(_) => gate
            .acts
            .keys(app, &g.caret_left)
            .and_then(|()| goes(gate))?,
        None => false,
    };
    let timer = timer(gate, app, lines)?;
    let double = double(gate, app, lines)?;
    let ok = gate.sel.caret_paste && key && click && moved && timer["ok"] == true;
    Ok(
        json!({ "key": key, "click": click, "moved": moved, "timer": timer, "double": double, "ok": ok }),
    )
}
