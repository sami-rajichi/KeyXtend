//! G19 in terminals (P10 design "In a terminal"): Windows Terminal and the classic console show Copy, Paste and
//! Translate on the gate's printed line; Copy copies it, and Paste types it back at the prompt with Num Lock on and
//! off, which the gate reads and puts back only through its own terminal in front. Copy during the gate's own `ping -t`
//! leaves the ping running.
#![cfg(windows)]

use kx_module_api::ports::{Rect, SelChip};
use kx_platform_windows::clip::open;
use serde_json::{Value, json};
use windows::Win32::Foundation::POINT;

use crate::apps::Opened;
use crate::clipkeep;
use crate::g19chips::gate::Gate;
use crate::g19chips::notepad::act;
use crate::g19chips::strip::Shown;
use crate::g19chips::term::{self, Kind, Term, numlock_on};
use crate::g19chips::termrow::row;
use crate::g19chips::{numlock, procs, script};
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
    let r = row(gate, kind, &app)?;
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

/// Terminal `kind` on the line script, driven once in front, then closed. Num Lock goes back, whatever happened, to
/// its state before the run: the one `carried` from the first terminal, else read now; the state used comes back.
fn one(
    gate: &Gate,
    (kind, carried): (Kind, Option<bool>),
    cases: &mut Vec<Value>,
    left: &mut Vec<String>,
) -> Result<(Value, bool), String> {
    let g = gate.g();
    let t = term::open(gate.ctx, kind, LINES, |ready, out| {
        script::lines(&g.term_line, ready, out)
    })?;
    let front = probe::front(gate.ctx, &t.app());
    let was = front.and_then(|()| numlock::first(carried, || numlock_on(t.hwnd)));
    let got = was.and_then(|was| {
        let (v, back) = numlock::then_back(
            || drive(gate, kind, &t, cases),
            || numlock::put_back(gate, &t, was),
        );
        numlock::settled(v, back, left).map(|v| (v, was))
    });
    left.extend(term::close(gate.ctx, t));
    got
}

/// The ping's reply row copied: the clipboard takes it, and the ping writes on for `ping_ms`.
fn pinged(gate: &Gate, t: &Term, cases: &mut Vec<Value>) -> Result<Value, String> {
    let app = t.app();
    let r = row(gate, Kind::Terminal, &app)?;
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

/// Windows Terminal, then the console, through `one`; the console gets the Num Lock state the first found, so both
/// put back the run's first state.
fn both<V>(
    mut one: impl FnMut(Kind, Option<bool>) -> Result<(V, bool), String>,
) -> Result<(V, V), String> {
    let (terminal, was) = one(Kind::Terminal, None)?;
    let (console, _) = one(Kind::Console, Some(was))?;
    Ok((terminal, console))
}

/// Both terminals, then the ping; `cases` gathers each case, `left` what stayed open.
pub fn run(gate: &Gate, cases: &mut Vec<Value>, left: &mut Vec<String>) -> Result<Value, String> {
    let (terminal, console) = both(|kind, carried| one(gate, (kind, carried), cases, left))?;
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
    fn the_console_puts_back_the_num_lock_state_the_first_terminal_found() {
        let mut seen = Vec::new();
        let got = both(|kind, carried| {
            seen.push((kind, carried));
            Ok((kind.name(), kind == Kind::Terminal))
        });
        assert_eq!(got, Ok((Kind::Terminal.name(), Kind::Console.name())));
        assert_eq!(seen, [(Kind::Terminal, None), (Kind::Console, Some(true))]);
        let mut runs = 0;
        let failed = both(|_, _| -> Result<((), bool), String> {
            runs += 1;
            Err("no row".into())
        });
        assert!(
            failed.is_err() && runs == 1,
            "a failed terminal stops the step"
        );
    }
}
