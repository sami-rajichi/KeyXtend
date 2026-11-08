//! G19 in terminals (P10 design "In a terminal"): Windows Terminal and the classic console show Copy, Paste and
//! Translate on the gate's printed line; Copy copies it, and Paste types it back at the prompt with Num Lock on and
//! off, which the gate reads and puts back only through its own terminal in front. Copy during the gate's own `ping -t`
//! leaves the ping running.
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
use crate::ownkeys::{self, Desk, Real};
use crate::probe;
use crate::proc::Process;
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
    for _ in 0..script::taps(numlock_on(t.hwnd)?, on) {
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

/// Num Lock put back to `was` on the gate's own terminal `app`, read by `on` and tapped by `tap` only while `app` is
/// in front on `desk`, then `settle_ms` given; an error says why it is not back.
pub fn back_on(
    desk: &dyn Desk,
    app: &Opened,
    (was, settle_ms): (bool, u64),
    on: &dyn Fn() -> Result<bool, String>,
    tap: &dyn Fn() -> Result<(), String>,
) -> Result<(), String> {
    let not = |e: String| format!("Num Lock not put back: {e}");
    if on().map_err(not)? != was {
        ownkeys::click(desk, app, tap).map_err(not)?;
        desk.settle(settle_ms);
    }
    if on().map_err(not)? != was {
        return Err(not("it stayed changed".into()));
    }
    Ok(())
}

/// Num Lock put back to `was` in terminal `t`, never brought forward, as that would tap into the window in front.
fn put_back(gate: &Gate, t: &Term, was: bool) -> Result<(), String> {
    let g = gate.g();
    let tap = || crate::keys::restore_combo(&g.numlock);
    let on = || numlock_on(t.hwnd);
    back_on(&Real, &t.app(), (was, g.settle_ms), &on, &tap)
}

/// Terminal `kind` on the line script, read for Num Lock once in front, driven, then closed; Num Lock goes back as
/// it was, whatever happened.
fn one(
    gate: &Gate,
    kind: Kind,
    cases: &mut Vec<Value>,
    left: &mut Vec<String>,
) -> Result<Value, String> {
    let g = gate.g();
    let t = term::open(gate.ctx, kind, LINES, |ready, out| {
        script::lines(&g.term_line, ready, out)
    })?;
    let was = probe::front(gate.ctx, &t.app()).and_then(|()| numlock_on(t.hwnd));
    let got = was.and_then(|was| {
        let v = drive(gate, kind, &t, cases);
        let back = put_back(gate, &t, was);
        match v {
            Ok(v) => Ok(finished(v, back)),
            Err(e) => {
                left.extend(back.err());
                Err(e)
            }
        }
    });
    left.extend(term::close(gate.ctx, t));
    got
}

/// One terminal's result with Num Lock put back as `back` says: it fails, naming why, when Num Lock is not back.
pub fn finished(mut v: Value, back: Result<(), String>) -> Value {
    v["ok"] = json!(v["ok"] == true && back.is_ok());
    v["numlock_back"] = json!(back.is_ok());
    if let Err(e) = back {
        v["numlock_error"] = json!(e);
    }
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

/// The pings the gate's own script `t` started since `before`, held; none, and a note in `left`, when its ready file
/// names no process.
fn held(
    gate: &Gate,
    t: &Term,
    before: &[procs::Proc],
    left: &mut Vec<String>,
) -> Vec<(procs::Proc, Option<Process>)> {
    let Some(shell) = script::shell_of(&t.mark) else {
        left.push("the ping script named no process, so no ping of it is ended".into());
        return Vec::new();
    };
    procs::hold(procs::ours(
        before,
        &procs::now(),
        &gate.g().ping_exe,
        shell,
    ))
}

/// Copy during the gate's own ping in Windows Terminal, which then closes; the ping gets `ping_exit_ms` to end, and
/// one still running then is ended and reported.
fn ping(gate: &Gate, cases: &mut Vec<Value>, left: &mut Vec<String>) -> Result<Value, String> {
    let g = gate.g();
    let before = procs::now();
    let t = term::open(gate.ctx, Kind::Terminal, PING, |ready, out| {
        script::ping(g, ready, out)
    })?;
    let got = pinged(gate, &t, cases);
    let held = held(gate, &t, &before, left);
    left.extend(term::close(gate.ctx, t));
    left.extend(procs::let_end(
        &held,
        g.ping_exit_ms,
        gate.ctx.cfg.timing.poll_ms,
    ));
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

    use std::cell::RefCell;

    use crate::apps::AppKind;
    use crate::ownkeys::fake::{Fake, hwnd};

    /// The gate's terminal as the tests open it: window `hwnd(1)`, no tab to check.
    fn terminal() -> Opened {
        Opened {
            kind: AppKind::Terminal,
            hwnd: hwnd(1),
            child: None,
            file: None,
            title_has: None,
        }
    }

    /// Num Lock put back to on, from `reads` in turn, on a desk with window `front` in front; the result and clicks.
    fn back(front: usize, reads: &[Result<bool, String>]) -> (Result<(), String>, usize) {
        let desk = Fake::new(hwnd(front), &["Windows PowerShell"]);
        let reads = RefCell::new(reads.to_vec());
        let on = || reads.borrow_mut().remove(0);
        let got = back_on(&desk, &terminal(), (true, 0), &on, &|| Ok(()));
        (got, desk.clicks.get())
    }

    #[test]
    fn num_lock_is_tapped_back_only_into_our_own_terminal_in_front() {
        assert_eq!(back(1, &[Ok(false), Ok(true)]), (Ok(()), 1));
        assert_eq!(back(1, &[Ok(true), Ok(true)]), (Ok(()), 0), "already back");
        let (got, clicks) = back(2, &[Ok(false), Ok(false)]);
        assert!(
            clicks == 0 && got.is_err_and(|e| e.contains("Num Lock not put back")),
            "another window in front"
        );
        let (got, clicks) = back(1, &[Err("unread".into())]);
        assert!(clicks == 0 && got.is_err(), "Num Lock unread");
        let (got, _) = back(1, &[Ok(false), Ok(false)]);
        assert!(got.is_err_and(|e| e.contains("stayed changed")));
    }

    #[test]
    fn a_terminal_that_left_num_lock_changed_fails() {
        let v = finished(json!({ "ok": true }), Ok(()));
        assert_eq!(
            (v["ok"].as_bool(), v["numlock_back"].as_bool()),
            (Some(true), Some(true))
        );
        let v = finished(json!({ "ok": true }), Err("why".into()));
        assert_eq!(
            (&v["ok"], &v["numlock_error"]),
            (&json!(false), &json!("why"))
        );
        assert_eq!(finished(json!({ "ok": false }), Ok(()))["ok"], false);
    }
}
