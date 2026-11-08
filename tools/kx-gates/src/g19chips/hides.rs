//! When the strip hides (P10 design "When the chips hide" and "Keyboard states"): a press outside it, our keyboard's
//! Esc, our panel opening with its search aimed, and the keyboard closed to the tray, which is the run's last act; the
//! minimised keyboard follows `minimised`.
#![cfg(windows)]

use kx_mod_keyboard::{Nav, Role};
use serde_json::{Value, json};

use crate::apps::Opened;
use crate::board;
use crate::clippanel::Panel;
use crate::g19chips::docs::Line;
use crate::g19chips::gate::Gate;
use crate::g19chips::notepad::{Lines, by_keys};
use crate::g19chips::strip::Shown;
use crate::product::{self, Keyboard};
use crate::win::{self, sleep_ms};

/// The acts the step and the tray take: five selections by keys, a press and its release, Esc three times, the clip
/// key twice, the panel's two keys, three buttons pressed and the tray's selection.
pub const ACTS: usize = 24;
/// The cases that must show no strip: the aimed panel's and the tray's.
pub const QUIET: usize = 2;

/// The copy line selected by keys, as case `name`; the strip, if it showed.
fn select(gate: &Gate, app: &Opened, lines: &Lines, name: &str) -> Result<Option<Shown>, String> {
    Ok(by_keys(gate, app, lines, Line::Copy, name)?.1)
}

/// Presses the product's button `id` through UI Automation, then pauses `settle_ms`.
fn press(gate: &Gate, id: &str) -> Result<(), String> {
    product::invoke(&gate.reader, id)?;
    sleep_ms(gate.g().settle_ms);
    Ok(())
}

/// Our panel opens on the clip key, aimed at its search, and the strip goes; a selection made then shows none, and
/// the clip key shuts the panel. `cases` gathers the aimed case.
fn panel(
    gate: &Gate,
    app: &Opened,
    lines: &Lines,
    cases: &mut Vec<Value>,
) -> Result<Value, String> {
    let g = gate.g();
    let (clip, panel) = (
        board::id_of(Role::Clip)?,
        Panel::new(gate.ctx, g.gone_wait_ms)?,
    );
    let shown = select(gate, app, lines, "panel_before")?.is_some();
    gate.acts.key(&clip)?;
    let opened = panel.wait(true);
    let hid = shown && gate.strip.goes(|_| true)?;
    let before = Gate::before();
    gate.acts.keys(app, &g.caret_left)?;
    gate.acts.keys(app, &g.select_home)?;
    cases.push(gate.quiet("panel_aimed", &before)?);
    gate.acts.key(&clip)?;
    let shut = panel.wait(false);
    gate.acts.key(&board::nav_key(Nav::Esc)?)?;
    let ok = hid && opened.is_ok() && shut.is_ok();
    Ok(json!({ "hid": hid, "opened": opened.is_ok(), "shut": shut.is_ok(), "ok": ok }))
}

/// The keyboard minimised to its bubble: the strip shows as `minimised` says, then the bubble brings it back.
fn minimised(gate: &Gate, app: &Opened, lines: &Lines) -> Result<Value, String> {
    let (g9, g) = (&gate.ctx.cfg.g9, gate.g());
    press(gate, &g9.put_away)?;
    let shows = select(gate, app, lines, "minimised")?.is_some();
    press(gate, &g9.bring_back)?;
    let poll = gate.ctx.cfg.timing.poll_ms;
    let back = win::poll_until(g.strip_wait_ms, poll, || Keyboard::find().ok()).is_some();
    gate.acts.key(&board::nav_key(Nav::Esc)?)?;
    let ok = shows == gate.sel.minimised && back;
    Ok(json!({ "shows": shows, "minimised": gate.sel.minimised, "back": back, "ok": ok }))
}

/// A press outside the strip, our Esc, our aimed panel and the minimised keyboard; `cases` gathers the panel's.
pub fn run(
    gate: &Gate,
    app: &Opened,
    lines: &Lines,
    cases: &mut Vec<Value>,
) -> Result<Value, String> {
    let outside = match select(gate, app, lines, "outside")? {
        Some(_) => {
            let at = lines.end(Line::Arabic);
            gate.acts.down(app, at)?;
            let went = gate.strip.goes(|_| true);
            gate.acts.up(at)?;
            went?
        }
        None => false,
    };
    let esc = match select(gate, app, lines, "esc")? {
        Some(_) => gate
            .acts
            .key(&board::nav_key(Nav::Esc)?)
            .and_then(|()| gate.strip.goes(|_| true))?,
        None => false,
    };
    let panel = panel(gate, app, lines, cases)?;
    let minimised = minimised(gate, app, lines)?;
    let ok = outside && esc && panel["ok"] == true && minimised["ok"] == true;
    Ok(json!({ "outside": outside, "esc": esc, "panel": panel, "minimised": minimised, "ok": ok }))
}

/// The keyboard closed to the tray: the strip goes, and a selection made then shows none. The copy stays in the tray
/// until the run ends it.
pub fn tray(
    gate: &Gate,
    app: &Opened,
    lines: &Lines,
    cases: &mut Vec<Value>,
) -> Result<Value, String> {
    let shown = select(gate, app, lines, "tray_before")?.is_some();
    press(gate, &gate.g().tray)?;
    let hid = shown && gate.strip.goes(|_| true)?;
    let before = Gate::before();
    gate.acts.click(app, lines.end(Line::Copy))?;
    gate.acts.keys(app, &gate.g().select_home)?;
    let quiet = gate.quiet("tray", &before)?;
    let ok = hid && quiet["chips"] == true;
    cases.push(quiet);
    Ok(json!({ "hid": hid, "ok": ok }))
}
