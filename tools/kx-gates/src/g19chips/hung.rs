//! A hung app (P10 plan's edge cases): target-window takes the front, then sleeps in its hang mode, so no strip shows
//! on it, and the gate never asks the hung window anything; the gate's Notepad then shows the strip while the target
//! still sleeps, and the time from the last key to the strip is the lock's wait. After the hang, a new selection still
//! shows the strip, and the field watch still hears focus: a password box locks the lane.
#![cfg(windows)]

use std::time::Instant;

use kx_mod_keyboard::Nav;
use serde_json::{Value, json};

use crate::apps::Opened;
use crate::g5play::hang_target;
use crate::g19chips::act::Then;
use crate::g19chips::docs::Line;
use crate::g19chips::gate::{Gate, wait_ms};
use crate::g19chips::kept::{Kept, Target};
use crate::g19chips::notepad::{Lines, by_keys};
use crate::win::{self, ms_since, sleep_ms};
use crate::{board, launch, probe, typebox};

/// The acts the step pauses after: a selection by keys during the hang and one after it, each ended with our Esc;
/// the keys during the hang take no pause.
pub const ACTS: usize = 5;
/// What follows the keys during the hang: no pause, so the lock's wait runs from their input.
const LAST: Then = Then::Watch;
/// The hung app's own case, which must show no strip.
const HUNG: &str = "hung";
/// Notepad's case while the target sleeps.
const BESIDE_HUNG: &str = "beside_hung";
/// Notepad's case after the hang.
const AFTER_HUNG: &str = "after_hung";

/// True when the strip, first seen `lock_ms` after the act that ended at `sent`, came before the hang from `at` ended.
pub fn in_hang(at: Instant, sent: Instant, lock_ms: Option<u64>, hang_ms: u64) -> bool {
    lock_ms.is_some_and(|ms| wait_ms(at, sent).saturating_add(ms) < hang_ms)
}

/// The parts of the step's verdict that did not hold, by name: no strip on the hung app, one beside it within the
/// hang, one after it, and the field watch.
fn failed<'a>(parts: &[(&'a str, bool)]) -> Vec<&'a str> {
    parts
        .iter()
        .filter(|(_, held)| !held)
        .map(|(name, _)| *name)
        .collect()
}

/// The target brought forward and then hung, as a sleeping window cannot take the front, then Notepad's selection
/// while it sleeps; whether the strip came within the hang.
fn during(
    gate: &Gate,
    app: &Opened,
    lines: &Lines,
    target: &Opened,
    cases: &mut Vec<Value>,
) -> Result<bool, String> {
    let (ctx, g) = (gate.ctx, gate.g());
    if !win::front(target.hwnd, &ctx.cfg.timing, &ctx.cfg.keys) {
        return Err("could not bring target-window to the front before its hang".into());
    }
    hang_target(target.hwnd, g.hang_ms)?;
    let at = Instant::now();
    sleep_ms(g.hang_lead_ms);
    cases.push(gate.quiet_hung(HUNG, target.hwnd)?);
    probe::front(ctx, app)?;
    let before = Gate::before();
    gate.acts.click(app, lines.end(Line::Copy))?;
    gate.acts.keys_then(app, &g.select_home, LAST)?;
    let (mut case, _) =
        gate.case_within(BESIDE_HUNG, &before, &g.editable_chips, g.hung_wait_ms)?;
    let within = in_hang(at, gate.acts.sent(), case["ms"].as_u64(), g.hang_ms);
    case["during_hang"] = json!(within);
    cases.push(case);
    sleep_ms(g.hang_ms.saturating_sub(ms_since(at)));
    Ok(within)
}

/// A password box in a fresh target-window: the lane shows the lock, so the field watch heard its focus.
fn heard(gate: &Gate, left: &mut Vec<String>) -> Value {
    let ctx = gate.ctx;
    let args = typebox::password_args(ctx, true);
    let b = typebox::in_target(ctx, args, |t| Ok(t.lane_until(|l| l.locked).is_some()));
    left.extend(b.left);
    let locked = b.got.as_ref().is_ok_and(|l| *l);
    json!({ "locked": locked, "layout_back": b.back, "error": b.got.err(), "ok": locked && b.back })
}

/// The step's verdict over `cases`, with the strip seen within the hang and after it, and the field watch `field`.
fn verdict(cases: &[Value], (within, after): (bool, bool), field: &Value) -> Value {
    let told = |name: &str| {
        cases
            .iter()
            .any(|c| c["case"] == name && c["chips"] == true)
    };
    let lock_ms = cases
        .iter()
        .find(|c| c["case"] == BESIDE_HUNG)
        .map(|c| c["ms"].clone());
    let parts = [
        ("quiet", told(HUNG)),
        ("beside", told(BESIDE_HUNG)),
        ("during_hang", within),
        ("after", after),
        ("field_watch", field["ok"] == true),
    ];
    let failed = failed(&parts);
    json!({ "lock_ms": lock_ms, "field_watch": field, "failed": failed, "ok": failed.is_empty() })
}

/// The hung target's look, Notepad beside it and after it, then the field watch; the target closes first.
pub fn run(
    gate: &Gate,
    app: &Opened,
    lines: &Lines,
    cases: &mut Vec<Value>,
    left: &mut Vec<String>,
) -> Result<Value, String> {
    let cap = gate.ctx.target.hang_max_ms;
    if gate.g().hang_ms > cap {
        return Err(format!(
            "g19_product.hang_ms passes target-window's hang_max_ms ({cap} ms)"
        ));
    }
    let target = Kept::new(Target::new(gate.ctx, launch::start_target(gate.ctx)?));
    let got = during(gate, app, lines, &target.get().app, cases);
    left.extend(target.close());
    let within = got?;
    probe::front(gate.ctx, app)?;
    gate.acts.key(app, &board::nav_key(Nav::Esc)?)?;
    let (after, _) = by_keys(gate, app, lines, Line::Copy, AFTER_HUNG)?;
    let shown = after["chips"] == true;
    cases.push(after);
    gate.acts.key(app, &board::nav_key(Nav::Esc)?)?;
    let field = heard(gate, left);
    Ok(verdict(cases, (within, shown), &field))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn the_strip_counts_as_within_the_hang_only_when_first_seen_before_it_ends() {
        let at = Instant::now();
        let sent = at + Duration::from_millis(2_000);
        assert!(in_hang(at, sent, Some(400), 3_000));
        assert!(!in_hang(at, sent, Some(1_000), 3_000), "at its end");
        assert!(!in_hang(at, sent, None, 3_000), "no strip");
    }

    #[test]
    fn the_locks_wait_starts_at_the_last_keys_input() {
        use crate::g19chips::act::pause_ms;
        let settle = crate::config::load().expect("loads").g19_product.settle_ms;
        assert_eq!(pause_ms(LAST, settle), 0, "no settle before the watch");
        assert_eq!(pause_ms(Then::Settle, settle), settle);
    }

    #[test]
    fn the_step_fails_on_a_strip_after_the_hang_or_a_deaf_field_watch() {
        let all = [
            ("quiet", true),
            ("during_hang", true),
            ("field_watch", true),
        ];
        assert!(failed(&all).is_empty());
        let late = [
            ("quiet", true),
            ("during_hang", false),
            ("field_watch", true),
        ];
        assert_eq!(
            failed(&late),
            ["during_hang"],
            "the strip came after the hang"
        );
        let deaf = [
            ("quiet", true),
            ("during_hang", true),
            ("field_watch", false),
        ];
        assert_eq!(failed(&deaf), ["field_watch"]);
    }

    #[test]
    fn the_verdict_reads_the_hung_and_beside_cases_by_name() {
        let cases = [
            json!({ "case": HUNG, "chips": true }),
            json!({ "case": BESIDE_HUNG, "chips": true, "ms": 400 }),
        ];
        let field = json!({ "ok": true });
        let v = verdict(&cases, (true, true), &field);
        assert_eq!((&v["ok"], &v["lock_ms"]), (&json!(true), &json!(400)));
        let v = verdict(&cases[..1], (true, true), &field);
        assert_eq!(v["failed"], json!(["beside"]), "no case beside the hang");
        let v = verdict(&cases[1..], (true, false), &field);
        assert_eq!(v["failed"], json!(["quiet", "after"]));
    }
}
