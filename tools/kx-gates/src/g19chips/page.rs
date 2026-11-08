//! G19 on the gate's own Chrome page: the paragraph shows Copy and Translate, the text box all four, the password box
//! and the box of unknown role none; Copy with Shift latched reaches the page as Ctrl+C, never Ctrl+Shift+C, as the
//! page's title counts.
#![cfg(windows)]

use kx_mod_keyboard::{Latch, Nav};
use kx_module_api::ports::SelChip;
use kx_platform_windows::uia;
use serde_json::{Value, json};
use windows::Win32::Foundation::{POINT, RECT};

use crate::apps::Opened;
use crate::g19chips::docs::{counts, page};
use crate::g19chips::gate::Gate;
use crate::g19chips::notepad::act;
use crate::modekey::ModeKey;
use crate::{board, probe, win};

/// The acts the step takes: a drag, Shift latched and let go, Copy, three clicks with Shift+Home, and Esc.
pub const ACTS: usize = 11;
/// The boxes that must show no strip: the password box and the unknown one.
pub const QUIET: usize = 2;

/// The screen box of the page's element named `name`, once UI Automation shows it.
fn named(gate: &Gate, app: &Opened, name: &str) -> Result<RECT, String> {
    let t = &gate.ctx.cfg.timing;
    let found = || uia::rect(&gate.uia().named(app.hwnd, name)?).ok();
    win::poll_until(t.read_wait_ms, t.poll_ms, found)
        .ok_or_else(|| format!("the page shows no {name}"))
}

/// The middle height of `r`, `inset` in from its left or right edge.
fn inside(r: &RECT, inset: i32, right: bool) -> POINT {
    POINT {
        x: if right {
            r.right.saturating_sub(inset)
        } else {
            r.left.saturating_add(inset)
        },
        y: i32::midpoint(r.top, r.bottom),
    }
}

/// The paragraph dragged, then Copy with Shift latched: the page's counts of Ctrl+C and Ctrl+Shift+C.
fn paragraph(gate: &Gate, app: &Opened, cases: &mut Vec<Value>) -> Result<Value, String> {
    let (g, inset) = (gate.g(), gate.ctx.cfg.probes.inset_px);
    let r = named(gate, app, &g.page_text)?;
    let before = Gate::before();
    gate.acts
        .drag(app, inside(&r, inset, false), inside(&r, inset, true))?;
    let (case, shown) = gate.case("page_drag", &before, &g.page_chips)?;
    cases.push(case);
    let shift = board::latch_id(Latch::Shift)?;
    gate.acts.key(&shift)?;
    let copy = act(gate, app, shown, SelChip::Copy, |seq| {
        gate.clip_holds(seq, &g.page_text)
    })?;
    let latched = ModeKey::by_id(shift.clone())?.on()?;
    if latched {
        gate.acts.key(&shift)?;
    }
    let (ctrl_c, ctrl_shift_c) = counts(&win::title(app.hwnd), g).unwrap_or_default();
    let ok = copy["ok"] == true && latched && ctrl_c > 0 && ctrl_shift_c == 0;
    Ok(
        json!({ "copy": copy, "shift_latched": latched, "ctrl_c": ctrl_c, "ctrl_shift_c": ctrl_shift_c, "ok": ok }),
    )
}

/// The box named `name` clicked near its right end and Shift+Home pressed, as case `name`: it shows `want`, or none.
fn boxed(gate: &Gate, app: &Opened, name: &str, want: Option<&[String]>) -> Result<Value, String> {
    let r = named(gate, app, name)?;
    let before = Gate::before();
    gate.acts
        .click(app, inside(&r, gate.ctx.cfg.probes.inset_px, true))?;
    gate.acts.keys(app, &gate.g().select_home)?;
    match want {
        Some(chips) => Ok(gate.case(name, &before, chips)?.0),
        None => gate.quiet(name, &before),
    }
}

/// The page's four boxes, in their order; `cases` gathers each.
fn drive(gate: &Gate, app: &Opened, cases: &mut Vec<Value>) -> Result<Value, String> {
    let g = gate.g();
    probe::front(gate.ctx, app)?;
    let copy = paragraph(gate, app, cases)?;
    cases.push(boxed(gate, app, &g.box_name, Some(&g.editable_chips))?);
    gate.acts.key(&board::nav_key(Nav::Esc)?)?;
    cases.push(boxed(gate, app, &g.pass_name, None)?);
    cases.push(boxed(gate, app, &g.unknown_name, None)?);
    Ok(copy)
}

/// Opens the page, runs its cases, then closes only the gate's own tab; what was left open joins `left`.
pub fn run(gate: &Gate, cases: &mut Vec<Value>, left: &mut Vec<String>) -> Result<Value, String> {
    let doc = probe::open_page(gate.ctx, &page(gate.g()))?;
    let (v, notes) = probe::run_on(gate.ctx, doc, |d| drive(gate, &d.app, cases))?;
    left.extend(notes);
    Ok(v)
}
