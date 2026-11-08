//! `g19 look`: the product's own selection watch, with the module's defaults, looked at about every `poll_ms` for
//! `look_s` seconds. Each change prints the program in front, the kind, whether a selection holds text, its lines and
//! the watch's tells, with the look's ms, or its error. It starts and drives nothing, and prints no text or name.
#![cfg(windows)]

use std::time::{Duration, Instant};

use kx_module_api::ports::{SelError, SelSpot};
use kx_platform_windows::window::foreground;
use serde_json::{Value, json};

use crate::apps::Ctx;
use crate::g19chips::GATE;
use crate::g19chips::watch::Watch;
use crate::out::say;
use crate::p10cfg::selection_defaults;
use crate::win::{self, ms_since, sleep_ms};

/// The run's name.
pub const LOOK: &str = "look";
/// The program named when no window is in front.
const NONE: &str = "none";
/// The line count of a spot with no line box.
const NO_LINE: u8 = 0;
/// The line count of a spot whose first and last line are one.
const ONE_LINE: u8 = 1;
/// The line count of a spot whose first and last line differ: two lines or more, as a spot keeps only those two.
const LINES: u8 = 2;

/// How many lines a look's selection spans as its spot tells.
fn lines(s: &SelSpot) -> u8 {
    match (s.first, s.last) {
        (None, None) => NO_LINE,
        (Some(a), Some(b)) if a != b => LINES,
        _ => ONE_LINE,
    }
}

/// What a look in program `app` found after the watch told `told` changes; no time, so only a change differs.
fn seen(app: &str, got: &Result<SelSpot, SelError>, told: usize) -> Value {
    match got {
        Ok(s) => {
            let kind = format!("{:?}", s.kind);
            json!({ "app": app, "kind": kind, "selected": s.selected, "lines": lines(s), "told": told })
        }
        Err(e) => json!({ "app": app, "error": format!("{e:?}"), "told": told }),
    }
}

/// Runs `g19 look`: each change of the look is printed and kept, with the ms it took and the ms since the start.
pub fn run(ctx: &Ctx) -> Result<Value, String> {
    let mut watch = Watch::start(&selection_defaults()?)?;
    let (start, span) = (
        Instant::now(),
        Duration::from_secs(ctx.cfg.g19_product.look_s),
    );
    let (mut last, mut changes) = (Value::Null, Vec::new());
    while start.elapsed() < span {
        let (got, ms) = watch.spot();
        let app = win::program_name(foreground()).unwrap_or_else(|| NONE.into());
        let now = seen(&app, &got, watch.told());
        if now != last {
            let mut line = now.clone();
            (line["ms"], line["at_ms"]) = (json!(ms), json!(ms_since(start)));
            say!("{GATE} {LOOK}: {line}");
            changes.push(line);
            last = now;
        }
        sleep_ms(ctx.cfg.timing.poll_ms);
    }
    Ok(json!({ "gate": GATE, "name": LOOK, "changes": changes }))
}

#[cfg(test)]
mod tests {
    use kx_module_api::ports::{Rect, Screen, SelKind};

    use super::*;

    /// A line box at height `y`.
    fn at(y: i32) -> Rect {
        Rect {
            x: 4321,
            y,
            w: 99,
            h: 21,
        }
    }

    /// A read-only box's spot with `first` and `last` lines.
    fn spot(first: Option<Rect>, last: Option<Rect>) -> SelSpot {
        SelSpot {
            box_id: 7,
            kind: SelKind::ReadOnly,
            selected: first.is_some(),
            first,
            last,
            caret: None,
            screen: Screen::default(),
        }
    }

    #[test]
    fn a_look_is_told_by_its_program_kind_flags_lines_and_tells_never_its_boxes() {
        let one = seen("claude.exe", &Ok(spot(Some(at(8765)), Some(at(8765)))), 3);
        let want = json!({ "app": "claude.exe", "kind": "ReadOnly", "selected": true, "lines": 1, "told": 3 });
        assert_eq!(one, want);
        assert!(!one.to_string().contains("4321"), "no box: {one}");
        let two = seen("zen.exe", &Ok(spot(Some(at(10)), Some(at(40)))), 0);
        assert_eq!(
            (two["lines"].as_u64(), two["selected"].as_bool()),
            (Some(2), Some(true))
        );
        let none = seen("zen.exe", &Ok(spot(None, None)), 0);
        assert_eq!(
            (none["lines"].as_u64(), none["selected"].as_bool()),
            (Some(0), Some(false))
        );
    }

    #[test]
    fn a_failed_look_is_told_by_its_error_and_a_same_look_is_no_change() {
        let late = seen("claude.exe", &Err(SelError::TimedOut), 2);
        assert_eq!(
            late,
            json!({ "app": "claude.exe", "error": "TimedOut", "told": 2 })
        );
        assert_eq!(late, seen("claude.exe", &Err(SelError::TimedOut), 2));
        assert_ne!(
            late,
            seen("claude.exe", &Err(SelError::TimedOut), 3),
            "a tell is a change"
        );
    }
}
