//! A hung app (P10 plan's edge cases): target-window asleep in its hang mode takes the focus, so the module's look at
//! it times out and no strip shows; the gate's Notepad then shows the strip while the target still sleeps, and the
//! time it took is the lock's wait; after the hang, a new selection still shows it.
#![cfg(windows)]

use std::time::Instant;

use kx_mod_keyboard::Nav;
use serde_json::{Value, json};

use crate::apps::{self, Opened};
use crate::g5play::hang_target;
use crate::g19chips::docs::Line;
use crate::g19chips::gate::Gate;
use crate::g19chips::notepad::{Lines, by_keys};
use crate::win::{self, ms_since, sleep_ms};
use crate::{board, launch, probe};

/// The acts the step takes: a selection by keys during the hang, our Esc, and one after it.
pub const ACTS: usize = 5;
/// The case that must show no strip: the hung target's.
pub const QUIET: usize = 1;

/// The hung target in front, then Notepad's selection while it sleeps; whether that came within the hang.
fn during(
    gate: &Gate,
    app: &Opened,
    lines: &Lines,
    target: &Opened,
    cases: &mut Vec<Value>,
) -> Result<bool, String> {
    let (ctx, g) = (gate.ctx, gate.g());
    probe::front(ctx, app)?;
    hang_target(target.hwnd, g.hang_ms)?;
    let at = Instant::now();
    sleep_ms(g.hang_lead_ms);
    let before = Gate::before();
    if !win::front(target.hwnd, &ctx.cfg.timing, &ctx.cfg.keys) {
        return Err("could not bring the hung target-window to the front".into());
    }
    cases.push(gate.quiet("hung", &before)?);
    probe::front(ctx, app)?;
    let (mut case, _) = by_keys(gate, app, lines, Line::Copy, "beside_hung")?;
    let within = ms_since(at) < g.hang_ms;
    case["during_hang"] = json!(within);
    cases.push(case);
    sleep_ms(g.hang_ms.saturating_sub(ms_since(at)));
    Ok(within)
}

/// The hung target's look, Notepad beside it and after it; the target is closed before the last selection.
pub fn run(
    gate: &Gate,
    app: &Opened,
    lines: &Lines,
    cases: &mut Vec<Value>,
    left: &mut Vec<String>,
) -> Result<Value, String> {
    let target = launch::start_target(gate.ctx)?;
    let got = during(gate, app, lines, &target, cases);
    left.extend(apps::clean_up(gate.ctx, target));
    let within = got?;
    gate.acts.key(&board::nav_key(Nav::Esc)?)?;
    probe::front(gate.ctx, app)?;
    let (after, _) = by_keys(gate, app, lines, Line::Copy, "after_hung")?;
    let told = |name: &str, key: &str| cases.iter().any(|c| c["case"] == name && c[key] == true);
    let (quiet, beside) = (told("hung", "chips"), told("beside_hung", "chips"));
    let ok = quiet && beside && after["chips"] == true;
    let lock_ms = cases
        .iter()
        .find(|c| c["case"] == "beside_hung")
        .map(|c| c["ms"].clone());
    cases.push(after);
    Ok(
        json!({ "quiet": quiet, "beside": beside, "lock_ms": lock_ms, "during_hang": within, "ok": ok }),
    )
}
