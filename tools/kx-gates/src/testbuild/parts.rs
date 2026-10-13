//! The test build's pure parts: is a program a gate-input build, which copy to start again, did it warn, how it ended.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::budget::stamp_us;
use crate::hooklog::HookLine;

/// True when `bytes` hold `text`; an empty text is never held.
pub fn holds(bytes: &[u8], text: &str) -> bool {
    let text = text.as_bytes();
    !text.is_empty() && bytes.windows(text.len()).any(|w| w == text)
}

/// The copy to start again once the test build ends: the one running before it, unless that was the test build at
/// `test_build` itself, in any letter case.
pub fn to_restore(running: Option<&str>, test_build: &Path) -> Option<PathBuf> {
    let same = |p: &str| p.to_lowercase() == test_build.to_string_lossy().to_lowercase();
    running.filter(|p| !same(p)).map(PathBuf::from)
}

/// True when `log` holds `warn_line` on a line stamped at or after `since_us`.
pub fn warned(log: &str, since_us: i64, warn_line: &str) -> bool {
    let fresh = |l: &&str| stamp_us(l).is_some_and(|at| at >= since_us);
    log.lines().filter(fresh).any(|l| l.contains(warn_line))
}

/// How a test build's run ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Closed {
    /// It ended within `[timing] quit_wait_ms`.
    pub quit: bool,
    /// The owner's copy it had closed was started again; none when it had closed none.
    pub owner_back: Option<bool>,
    /// Its stats lines since its start, or why they could not be read.
    pub lines: Result<Vec<HookLine>, String>,
}

impl Closed {
    /// Its fields for a result line: flags, numbers and a read error, never a point or a key.
    pub fn json(&self) -> Value {
        let (lines, error) = match &self.lines {
            Ok(lines) => (json!(lines), Value::Null),
            Err(e) => (Value::Null, json!(e)),
        };
        json!({
            "quit": self.quit, "owner_back": self.owner_back, "hook_lines": lines, "hook_error": error,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gate_input() -> crate::p4cfg::GateInput {
        crate::config::load()
            .expect("kx-gates.toml loads")
            .gate_input
    }

    #[test]
    fn a_gate_input_build_holds_the_flag_text_and_another_build_does_not() {
        let flag = gate_input().flag;
        let built = [b"MZ\0head\0".as_slice(), flag.as_bytes(), b"\0tail"].concat();
        assert!(holds(&built, &flag));
        assert!(!holds(b"MZ\0head\0\0tail", &flag), "a shipped build");
        assert!(!holds(&built, ""), "no text names no build");
    }

    /// Where the test build sits, as `open` names it.
    const TEST_BUILD: &str = r"D:\Projects\KeyXtend\target\gate-input\release\keyxtend.exe";
    /// Where the owner's copy runs from.
    const OWNERS: &str = r"C:\Program Files\KeyXtend-dev\keyxtend\keyxtend.exe";

    #[test]
    fn only_the_owners_copy_is_started_again_never_the_test_build() {
        let test_build = Path::new(TEST_BUILD);
        assert_eq!(to_restore(None, test_build), None, "nothing ran");
        assert_eq!(to_restore(Some(OWNERS), test_build), Some(OWNERS.into()));
        let upper = TEST_BUILD.to_uppercase();
        assert_eq!(to_restore(Some(&upper), test_build), None, "any case");
        assert_eq!(to_restore(Some(TEST_BUILD), test_build), None);
    }

    #[test]
    fn only_a_warning_logged_since_the_start_counts() {
        let warn = gate_input().warn_line;
        let at = |stamp: &str| format!("{stamp}  WARN keyxtend::run: {warn} flag=1");
        let (early, late) = ("2026-10-14T08:00:01.000000Z", "2026-10-14T08:00:02.000000Z");
        let since_us = stamp_us(late).expect("a stamp");
        assert!(!warned(&at(early), since_us, &warn), "an older run's");
        assert!(warned(&[at(early), at(late)].join("\n"), since_us, &warn));
        assert!(!warned(
            &format!("{late}  INFO x: started"),
            since_us,
            &warn
        ));
    }

    #[test]
    fn the_result_shows_how_it_ended_and_its_numbers_only() {
        let line = HookLine {
            events: 12,
            ..HookLine::default()
        };
        let ok = Closed {
            quit: true,
            owner_back: Some(false),
            lines: Ok(vec![line]),
        };
        let v = ok.json();
        assert_eq!(
            (v["quit"].as_bool(), v["owner_back"].as_bool()),
            (Some(true), Some(false))
        );
        assert_eq!(v["hook_lines"][0]["events"].as_u64(), Some(12));
        assert!(v["hook_error"].is_null());
        let broke = Closed {
            quit: false,
            owner_back: None,
            lines: Err("no line".into()),
        };
        let v = broke.json();
        assert!(v["owner_back"].is_null() && v["hook_lines"].is_null());
        assert_eq!(v["hook_error"].as_str(), Some("no line"));
    }
}
