//! G19 in terminals (P10 design "In a terminal"): Windows Terminal and the classic console show Copy, Paste and
//! Translate on the gate's printed line; Copy copies it, and Paste types it back at the prompt with Num Lock on and
//! off, which the gate puts back as it found it. Copy during the gate's own `ping -t` leaves the ping running.
#![cfg(windows)]

use kx_module_api::ports::{Rect, SelChip};
use kx_platform_windows::clip::open;
use kx_platform_windows::uia;
use serde_json::{Value, json};
use windows::Win32::Foundation::POINT;

use crate::apps::Opened;
use crate::clipkeep;
use crate::g19chips::gate::Gate;
use crate::g19chips::notepad::{act, rows};
use crate::g19chips::strip::Shown;
use crate::g19chips::term::{self, Kind, Term, numlock_on};
use crate::g19chips::{place, procs, script};
use crate::win::{self, sleep_ms};

/// The acts the step takes at the least: in each terminal a drag and Copy, then a drag, Paste and Enter with Num Lock
/// on and off; and the ping's drag and Copy.
pub const ACTS: usize = 18;
/// The Num Lock states Paste is tried with.
const NUMLOCK: [bool; 2] = [true, false];
/// The tag of the line script's files.
const LINES: &str = "lines";
/// The tag of the ping script's files.
const PING: &str = "ping";

/// The first text row of the terminal's window, once UI Automation shows one.
fn row(gate: &Gate, app: &Opened) -> Result<Rect, String> {
    let t = &gate.ctx.cfg.timing;
    let read = || {
        let parts = uia::lines(&gate.uia().text(app.hwnd)?).ok()?;
        rows(&parts.iter().map(place::rect).collect::<Vec<_>>())
            .into_iter()
            .next()
    };
    win::poll_until(t.read_wait_ms, t.poll_ms, read)
        .ok_or_else(|| "the terminal shows no text row".to_string())
}

/// Row `r` dragged from its start to its end, as case `name`.
fn select(
    gate: &Gate,
    app: &Opened,
    r: Rect,
    name: &str,
) -> Result<(Value, Option<Shown>), String> {
    let y = r.y.saturating_add(r.h / 2);
    let (from, to) = (
        POINT { x: r.x, y },
        POINT {
            x: r.x.saturating_add(r.w),
            y,
        },
    );
    let before = Gate::before();
    gate.acts.drag(app, from, to)?;
    gate.case(name, &before, &gate.g().terminal_chips)
}

/// Num Lock turned `on`, the row selected again, Paste clicked and Enter pressed: the script's `at`th line must be the
/// terminal line.
fn paste(
    gate: &Gate,
    app: &Opened,
    t: &Term,
    r: Rect,
    (at, on): (usize, bool),
) -> Result<Value, String> {
    let g = gate.g();
    for _ in 0..script::taps(numlock_on(), on) {
        gate.acts.keys(app, &g.numlock)?;
    }
    let (_, shown) = select(gate, app, r, "paste")?;
    let pasted = act(gate, app, shown, SelChip::Paste, |_| true)?;
    gate.acts.keys(app, &gate.ctx.cfg.keys.enter)?;
    let poll = gate.ctx.cfg.timing.poll_ms;
    let arrived = win::poll_until(g.strip_wait_ms, poll, || t.lines().get(at).cloned());
    let line = arrived.is_some_and(|l| l == g.term_line);
    Ok(json!({ "numlock": on, "pasted": pasted, "line": line, "ok": pasted["ok"] == true && line }))
}

/// One terminal's case, Copy and both Pastes; `cases` gathers its case.
fn drive(gate: &Gate, kind: Kind, t: &Term, cases: &mut Vec<Value>) -> Result<Value, String> {
    let (g, app) = (gate.g(), t.app());
    let r = row(gate, &app)?;
    let (case, shown) = select(gate, &app, r, kind.name())?;
    cases.push(case);
    let copy = act(gate, &app, shown, SelChip::Copy, |seq| {
        gate.clip_holds(seq, &g.term_line)
    })?;
    let mut pastes = Vec::new();
    for step in NUMLOCK.into_iter().enumerate() {
        pastes.push(paste(gate, &app, t, r, step)?);
    }
    let ok = copy["ok"] == true && pastes.iter().all(|p| p["ok"] == true);
    Ok(json!({ "copy": copy, "pastes": pastes, "ok": ok }))
}

/// Terminal `kind` on the line script, driven, then closed; Num Lock goes back as it was, whatever happened.
fn one(
    gate: &Gate,
    kind: Kind,
    cases: &mut Vec<Value>,
    left: &mut Vec<String>,
) -> Result<Value, String> {
    let (g, was) = (gate.g(), numlock_on());
    let t = term::open(gate.ctx, kind, LINES, |ready, out| {
        script::lines(&g.term_line, ready, out)
    })?;
    let got = drive(gate, kind, &t, cases);
    if numlock_on() != was {
        let _ = crate::keys::restore_combo(&g.numlock);
        sleep_ms(g.settle_ms);
    }
    let back = numlock_on() == was;
    left.extend(term::close(gate.ctx, t));
    Ok(finished(got?, back))
}

/// One terminal's result with Num Lock put back as `back` says: it fails when Num Lock stayed changed.
pub fn finished(mut v: Value, back: bool) -> Value {
    v["ok"] = json!(v["ok"] == true && back);
    v["numlock_back"] = json!(back);
    v
}

/// The ping's reply row copied: the clipboard takes it, and the ping writes on for `ping_ms`.
fn pinged(gate: &Gate, t: &Term, cases: &mut Vec<Value>) -> Result<Value, String> {
    let app = t.app();
    let r = row(gate, &app)?;
    let (case, shown) = select(gate, &app, r, PING)?;
    cases.push(case);
    let wait = clipkeep::wait(&gate.ctx.cfg.timing);
    let copy = act(gate, &app, shown, SelChip::Copy, |seq| {
        open::wait_change(seq, wait)
    })?;
    let before = t.lines().len();
    sleep_ms(gate.g().ping_ms);
    let after = t.lines().len();
    let ok = copy["ok"] == true && after > before;
    Ok(json!({ "copy": copy, "replies_before": before, "replies_after": after, "ok": ok }))
}

/// Copy during the gate's own ping in Windows Terminal, which then closes; a ping still running after is left.
fn ping(gate: &Gate, cases: &mut Vec<Value>, left: &mut Vec<String>) -> Result<Value, String> {
    let g = gate.g();
    let before = procs::now();
    let t = term::open(gate.ctx, Kind::Terminal, PING, |ready, out| {
        script::ping(g, ready, out)
    })?;
    let got = pinged(gate, &t, cases);
    left.extend(term::close(gate.ctx, t));
    let stayed = procs::stayed(&before, &procs::now(), &g.ping_exe);
    left.extend(
        stayed
            .iter()
            .map(|p| format!("still running: {} {}", p.exe, p.pid)),
    );
    got
}

/// Both terminals, then the ping; `cases` gathers each case, `left` what stayed open.
pub fn run(gate: &Gate, cases: &mut Vec<Value>, left: &mut Vec<String>) -> Result<Value, String> {
    let terminal = one(gate, Kind::Terminal, cases, left)?;
    let console = one(gate, Kind::Console, cases, left)?;
    let ping = ping(gate, cases, left)?;
    let ok = [&terminal, &console, &ping].iter().all(|v| v["ok"] == true);
    Ok(
        json!({ Kind::Terminal.name(): terminal, Kind::Console.name(): console, PING: ping, "ok": ok }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_terminal_that_left_num_lock_changed_fails() {
        let v = finished(json!({ "ok": true }), true);
        assert_eq!(
            (v["ok"].as_bool(), v["numlock_back"].as_bool()),
            (Some(true), Some(true))
        );
        assert_eq!(finished(json!({ "ok": true }), false)["ok"], false);
        assert_eq!(finished(json!({ "ok": false }), true)["ok"], false);
    }
}
