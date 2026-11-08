//! Our own inputs make selections that show the strip (P10 design "On a selection"): Shift latched and an arrow on our
//! keyboard, a Grab carry, and a drag in Right-click mode, whose selection comes late, on the app's event. The scroll
//! pad is left to the owner's check, as it needs a long document laid under the keyboard.
#![cfg(windows)]

use kx_mod_keyboard::{Assist, Latch, Nav, Role};
use serde_json::{Value, json};

use crate::apps::Opened;
use crate::board;
use crate::g19chips::docs::Line;
use crate::g19chips::gate::Gate;
use crate::g19chips::notepad::Lines;
use crate::modekey::{ModeKey, Wait};

/// The acts the step takes: Shift and an arrow after a click, a Grab's pick-up, glide and drop, a drag, and Esc thrice.
pub const ACTS: usize = 10;
/// What the result says of the scroll pad.
const SCROLL_PAD: &str = "the owner's check: the strip waits for the app's event after a scroll";

/// The wait for a mode key to read on or off.
fn lit(gate: &Gate) -> Wait {
    Wait {
        lit_ms: gate.ctx.cfg.gate_input.lit_ms,
        poll_ms: gate.ctx.cfg.timing.poll_ms,
    }
}

/// True when case `v` showed the strip untouched, with the box's keycaps.
fn shown(v: &Value) -> bool {
    v["seen"] == true && v["chips"] == true
}

/// Shift latched and Left on our keyboard, from the copy line's end.
fn keys(gate: &Gate, app: &Opened, lines: &Lines) -> Result<Value, String> {
    let before = Gate::before();
    gate.acts.click(app, lines.end(Line::Copy))?;
    gate.acts.key(&board::latch_id(Latch::Shift)?)?;
    gate.acts.key(&board::nav_key(Nav::Left)?)?;
    Ok(gate
        .case("keyboard_keys", &before, &gate.g().editable_chips)?
        .0)
}

/// Grab on: a click picks up at the copy line's start, the pointer glides to its end, and a click drops.
fn grab(gate: &Gate, app: &Opened, lines: &Lines) -> Result<Value, String> {
    let key = ModeKey::of(Role::Assist(Assist::Grab))?;
    key.set(true, lit(gate))?;
    let (from, to) = (lines.start(Line::Copy), lines.end(Line::Copy));
    let before = Gate::before();
    let made = gate
        .acts
        .click(app, from)
        .and_then(|()| gate.acts.glide(app, from, to));
    let dropped = made.and_then(|()| gate.acts.click(app, to));
    let case = dropped.and_then(|()| gate.case("grab", &before, &gate.g().editable_chips));
    let off = key.set(false, lit(gate));
    let case = case?.0;
    off?;
    Ok(case)
}

/// Right-click mode on: a drag over the copy line, whose selection the engine lets through late.
fn rclick(gate: &Gate, app: &Opened, lines: &Lines) -> Result<Value, String> {
    let key = ModeKey::of(Role::Assist(Assist::RClick))?;
    key.set(true, lit(gate))?;
    let before = Gate::before();
    let dragged = gate
        .acts
        .drag(app, lines.start(Line::Copy), lines.end(Line::Copy));
    let case = dragged.and_then(|()| gate.case("rclick", &before, &gate.g().editable_chips));
    let off = key.set(false, lit(gate));
    let case = case?.0;
    off?;
    Ok(case)
}

/// Each of our inputs' selections, each cleared with our Esc; `cases` gathers them.
pub fn run(
    gate: &Gate,
    app: &Opened,
    lines: &Lines,
    cases: &mut Vec<Value>,
) -> Result<Value, String> {
    let esc = board::nav_key(Nav::Esc)?;
    let mut ok = true;
    for step in [keys, grab, rclick] {
        let case = step(gate, app, lines)?;
        ok &= shown(&case);
        cases.push(case);
        gate.acts.key(&esc)?;
    }
    Ok(json!({ "scroll_pad": SCROLL_PAD, "ok": ok }))
}
