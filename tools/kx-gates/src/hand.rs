//! The owner's hand try: the owner turns a mode on with the running keyboard's key and tries holds, then this reads the
//! stats lines its hook logged meanwhile. It has no engine of its own and gives no verdict.
#![cfg(windows)]

use std::path::PathBuf;
use std::time::Duration;

use serde_json::{Value, json};

use crate::apps::Ctx;
use crate::budget::now_us;
use crate::hooklog;
use crate::out::say;
use crate::product::{self, NAME};
use crate::testbuild::holds;

/// Hand-try mode: Right-click.
pub const RIGHT: &str = "right";
/// Hand-try mode: Grab.
pub const GRAB: &str = "grab";
/// Each mode's name on the command line and in the line that tells the owner what to do.
const MODES: [(&str, &str); 2] = [(RIGHT, "Right-click"), (GRAB, "Grab")];
/// The note when no stats line came.
const NO_LINE: &str =
    "no stats line came: a mode still on logs its line only once it is turned off";

/// The label of the mode called `name`.
fn label(name: &str) -> Result<&'static str, String> {
    let found = MODES.iter().find(|(n, _)| *n == name).map(|m| m.1);
    found.ok_or_else(|| format!("the hand try runs {RIGHT} or {GRAB}, not {name}"))
}

/// The running copy's program; refused when none runs, or when it is a test build, whose engine never hears the hand.
fn owners_copy(ctx: &Ctx) -> Result<PathBuf, String> {
    let (_, exe) =
        product::running().ok_or_else(|| format!("no {NAME} is running; start it first"))?;
    let bytes = std::fs::read(&exe).map_err(|e| format!("{exe}: {e}"))?;
    if holds(&bytes, &ctx.cfg.gate_input.flag) {
        return Err(format!(
            "the running {NAME} is the test build, whose engine never hears the hand; start the installed copy"
        ));
    }
    Ok(PathBuf::from(exe))
}

/// The result: the mode, the seconds, each stats line of `log` stamped since `since_us`, their sum, and a note when
/// none came.
fn report(
    mode: &str,
    secs: u64,
    log: &str,
    since_us: i64,
    hook_line: &str,
) -> Result<Value, String> {
    let lines = hooklog::since(log, since_us, hook_line)?;
    let note = lines.is_empty().then_some(NO_LINE);
    Ok(json!({
        "gate": "assist", "mode": mode, "secs": secs, "lines": lines, "sum": hooklog::sum(&lines), "note": note,
    }))
}

/// Waits `secs` seconds while the owner tries mode `name` on the running copy, then reports its stats lines.
pub fn run(ctx: &Ctx, name: &str, secs: u64) -> Result<Value, String> {
    let mode = label(name)?;
    let log = product::files(&owners_copy(ctx)?)?.log;
    let since_us = now_us();
    say!(
        "hand try, {secs} s: turn {mode} on with the keyboard's key, try holds, then turn it off, as its hook logs its \
         numbers when it stops"
    );
    std::thread::sleep(Duration::from_secs(secs));
    let text = std::fs::read_to_string(&log).map_err(|e| format!("{}: {e}", log.display()))?;
    report(mode, secs, &text, since_us, &ctx.cfg.gate_input.hook_line)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::budget::stamp_us;

    /// A stamp of the kernel's log format, and one a second later.
    const EARLY: &str = "2026-10-14T08:00:01.000000Z";
    const LATE: &str = "2026-10-14T08:00:02.000000Z";

    fn hook() -> String {
        crate::config::load()
            .expect("kx-gates.toml loads")
            .gate_input
            .hook_line
    }

    /// A stats line stamped `at` with `events`, as the kernel writes it.
    fn line(at: &str, events: u64) -> String {
        format!(
            "{at}  INFO kx_mod_mouse::actor: {} events={events} p99_us=40 max_us=90 max_wait_ms=3 reinstalls=0 \
             send_errors=0 lost_echoes=0",
            hook()
        )
    }

    #[test]
    fn the_lines_logged_since_the_start_and_their_sum_make_the_result() {
        let log = [line(EARLY, 99), line(LATE, 12), line(LATE, 5)].join("\n");
        let since = stamp_us(LATE).expect("a stamp");
        let v = report("Grab", 30, &log, since, &hook()).expect("reads");
        assert_eq!(v["lines"].as_array().map(Vec::len), Some(2), "{v}");
        assert_eq!(v["sum"]["events"].as_u64(), Some(17));
        assert_eq!(
            (v["mode"].as_str(), v["secs"].as_u64()),
            (Some("Grab"), Some(30))
        );
        assert!(v["note"].is_null() && v.get("pass").is_none(), "{v}");
    }

    #[test]
    fn with_no_line_the_note_says_a_mode_still_on_logs_once_turned_off() {
        let since = stamp_us(LATE).expect("a stamp");
        let v = report("Grab", 30, &line(EARLY, 4), since, &hook()).expect("reads");
        assert_eq!(v["note"].as_str(), Some(NO_LINE));
        assert_eq!(v["lines"].as_array().map(Vec::len), Some(0));
    }

    #[test]
    fn each_mode_has_a_label_and_another_name_is_refused() {
        assert!(label(RIGHT).is_ok() && label(GRAB).is_ok());
        assert_ne!(label(RIGHT), label(GRAB));
        assert!(label("x").is_err());
    }
}
