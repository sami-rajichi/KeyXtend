//! G8's hook part (spec §10, "Mouse hook callback"): `g8 hook` and `g8 hook-weak` drive the product's test build with
//! G5's cases and judge the p99 its hook logs when it stops. Only the test build's engine hears automated input.
#![cfg(windows)]

use serde_json::{Value, json};
use spike_core::weak::Job;

use crate::apps::Ctx;
use crate::budget::{self, Part};
use crate::g5play::Played;
use crate::g8frames::GATE;
use crate::hooklog::{self, HookLine};
use crate::out::say;
use crate::p4cfg::G8Hook;
use crate::testbuild::{Closed, TestBuild};
use crate::{g5, g8};
use crate::{probe, product};

/// The name that judges the hook on this PC.
pub const HOOK: &str = "hook";
/// The name that judges it in weak mode, capped as `g8 weak` caps the product.
pub const HOOK_WEAK: &str = "hook-weak";
/// The part's name in the result.
const PART: &str = "hook";
/// The budget's name in the result.
const BUDGET: &str = "hook_p99_ms";

/// Runs `g8 hook` or `g8 hook-weak`; none for another name.
pub fn run(ctx: &Ctx, name: &str) -> Option<Result<Value, String>> {
    match name {
        HOOK => Some(hook(ctx, false)),
        HOOK_WEAK => Some(hook(ctx, true)),
        _ => None,
    }
}

/// The hook's budget: the summed lines' p99 in ms, not measured when they hold fewer than `min_events` events.
fn part(sum: &HookLine, g: &G8Hook) -> Part {
    let got = (sum.events >= g.min_events).then(|| sum.p99_ms());
    Part::at_most(BUDGET, got, g.p99_ms)
}

/// The result line: the budget decides, once the test build quit in time; the cases' own results are reported only.
fn result(weak: bool, played: &Played, closed: &Closed, g: &G8Hook, left_open: &[String]) -> Value {
    let lines = closed.lines.as_deref().unwrap_or_default();
    let parts = [part(&hooklog::sum(lines), g)];
    json!({
        "gate": GATE, "keyboard": product::NAME, "part": PART, "weak": weak, "hook": hooklog::summed(lines),
        "hook_lines": lines.len(), "cases": { "played": played.count, "matched": played.ok },
        "test_build": closed.json(), "left_open": left_open, "parts": parts,
        "pass": closed.pass(budget::all_pass(&parts)),
    })
}

/// Closes `job`, which ends a test build that failed to start, then lets go of anything left down; what it let go.
fn ended(job: Option<Job>) -> Vec<String> {
    drop(job);
    probe::released()
}

/// Opens the test build, in weak mode inside `g8 weak`'s job, plays G5's cases `[g8_hook] rounds` times on
/// target-window, then closes it and judges its stats lines. When it does not quit, or fails to start, closing the job
/// ends it and any button it left down is let go.
fn hook(ctx: &Ctx, weak: bool) -> Result<Value, String> {
    let (job, _) = g8::weak_job(ctx, weak)?;
    let build = match TestBuild::open(ctx, job.as_ref()) {
        Ok(build) => build,
        Err(e) => return Err(format!("{e}; left open: {:?}", ended(job))),
    };
    let g = &ctx.cfg.g8_hook;
    let (driven, mut left_open) = match build.mouse() {
        Ok(mouse) => g5::on_target(ctx, &build, &mouse, g.rounds),
        Err(e) => (Err(e), Vec::new()),
    };
    let (closed, released) = build.close_in(job);
    left_open.extend(released);
    let (_, played) = driven.map_err(|e| closed.failed(&e, &left_open))?;
    let v = result(weak, &played, &closed, g, &left_open);
    say!("G8 hook {}; pass {}", v["hook"], v["pass"]);
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hook() -> G8Hook {
        crate::config::load().expect("kx-gates.toml loads").g8_hook
    }

    fn line(events: u64, p99_us: u32) -> HookLine {
        HookLine {
            events,
            p99_us,
            ..HookLine::default()
        }
    }

    #[test]
    fn too_few_events_fail_as_not_measured() {
        let g = hook();
        let few = part(&line(g.min_events - 1, 10), &g);
        assert_eq!((few.got, few.pass), (None, false));
        let fast = part(&line(g.min_events, 10), &g);
        assert_eq!((fast.name, fast.pass), (BUDGET, true));
        assert!(
            !part(&line(g.min_events, u32::MAX), &g).pass,
            "past the budget"
        );
    }

    #[test]
    fn the_verdict_is_the_hooks_p99_once_the_test_build_quit_and_the_cases_are_reported_only() {
        let g = hook();
        let played = Played {
            lines: Vec::new(),
            ok: 5,
            count: 6,
        };
        let closed = Closed {
            quit: true,
            owner_back: None,
            lines: Ok(vec![line(g.min_events, 10)]),
        };
        let v = result(true, &played, &closed, &g, &[]);
        assert_eq!(
            v["pass"],
            json!(true),
            "a failed case does not fail the part"
        );
        let head = (v["gate"].as_str(), v["part"].as_str(), v["weak"].as_bool());
        assert_eq!(head, (Some("G8"), Some(PART), Some(true)));
        assert_eq!(
            (v["cases"]["matched"].as_u64(), v["hook_lines"].as_u64()),
            (Some(5), Some(1))
        );
        let hung = Closed {
            quit: false,
            ..closed.clone()
        };
        let v = result(true, &played, &hung, &g, &[]);
        assert_eq!(v["pass"], json!(false), "the test build did not quit");
        let unread = Closed {
            lines: Err("no log".into()),
            ..closed
        };
        let v = result(false, &played, &unread, &g, &[]);
        assert_eq!(v["pass"], json!(false), "no line, not measured");
    }
}
