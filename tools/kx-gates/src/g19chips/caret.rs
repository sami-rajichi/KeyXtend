//! The caret's Paste (P10 design "At a caret"): a click on the empty line, with the clipboard holding the cut line,
//! shows Paste alone; it goes on our keyboard's key, on a press anywhere, on a caret step the app makes with no input,
//! and after `caret_s`. A double click's brief Paste is recorded, never failed.
#![cfg(windows)]

use std::time::Instant;

use kx_mod_keyboard::Nav;
use serde_json::{Value, json};
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::Graphics::Gdi::MapWindowPoints;

use crate::apps::Opened;
use crate::board;
use crate::g19chips::docs::Line;
use crate::g19chips::gate::Gate;
use crate::g19chips::notepad::Lines;
use crate::g19chips::plan::on_time;
use crate::g19chips::select;
use crate::g19chips::strip::{Shown, ids};
use crate::g23::live::poll_read;
use crate::p10cfg::caret_ms;
use crate::win::{self, ms_since};

/// The acts the step takes: four clicks on the caret line, our key, a press and its release, a double click and our
/// Esc; the app's caret step takes no input.
pub const ACTS: usize = 9;

/// The system caret's screen box in the thread that owns `hwnd`, for the caret note; none when it shows no caret.
#[allow(unsafe_code, reason = "A plain point mapping into a local.")]
fn caret_box(hwnd: HWND) -> Option<RECT> {
    let info = win::thread_info(win::owner(hwnd).1);
    if info.hwndCaret.is_invalid() {
        return None;
    }
    let c = info.rcCaret;
    let mut at = [
        POINT {
            x: c.left,
            y: c.top,
        },
        POINT {
            x: c.right,
            y: c.bottom,
        },
    ];
    // SAFETY: `at` is a local array the call maps in place; a dead window maps nothing.
    let _ = unsafe { MapWindowPoints(Some(info.hwndCaret), None, &mut at) };
    let [from, to] = at;
    Some(RECT {
        left: from.x,
        top: from.y,
        right: to.x,
        bottom: to.y,
    })
}

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
    let (r, p, caret) = (s.rect, lines.caret(), caret_box(app.hwnd));
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
    let at = caret.map(|c| [c.left, c.top, c.right, c.bottom]);
    let near =
        json!({ "strip": [r.left, r.top, r.right, r.bottom], "caret": at, "click": [p.x, p.y] });
    Ok(json!({ "ok": ok, "went_ms": after, "caret_ms": stays_ms, "caret_note": near }))
}

/// The app moves the caret one character back with no input, and the caret's Paste goes.
fn moved(gate: &Gate, app: &Opened, lines: &Lines) -> Result<Value, String> {
    if shows(gate, app, lines)?.is_none() {
        return Ok(json!({ "ok": false, "error": "no Paste" }));
    }
    let max = gate.ctx.cfg.g1.max_read_chars;
    let from = select::caret(gate.uia(), app.hwnd, max)?;
    select::back(gate.uia(), app.hwnd)?;
    let to = select::caret(gate.uia(), app.hwnd, max)?;
    let went = goes(gate)?;
    let ok = select::stepped(from, to) && went;
    Ok(json!({ "stepped": select::stepped(from, to), "went": went, "ok": ok }))
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
    let moved = moved(gate, app, lines)?;
    let timer = timer(gate, app, lines)?;
    let double = double(gate, app, lines)?;
    let ok = gate.sel.caret_paste && key && click && moved["ok"] == true && timer["ok"] == true;
    Ok(
        json!({ "key": key, "click": click, "moved": moved, "timer": timer, "double": double, "ok": ok }),
    )
}
