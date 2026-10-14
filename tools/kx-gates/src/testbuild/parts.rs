//! The test build's pure parts: may it start, is a program a gate-input build, which copy to start again and what to
//! say when none is, did it warn, how it ended.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::budget::stamp_us;
use crate::hooklog::HookLine;
use crate::product::NAME;

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

/// What a copy that did not quit in time is said to have done.
pub const SLOW: &str = "did not quit in time";
/// What the owner is told to do when their copy is not started again.
pub const START_MENU: &str = "start it from the Start menu";

/// Refuses to start the test build unless this kx-gates surely lacks uiAccess, which the test build would inherit.
pub fn plain(ui_access: Result<bool, String>) -> Result<(), String> {
    match ui_access {
        Ok(false) => Ok(()),
        Ok(true) => Err("this kx-gates has uiAccess, which the test build would inherit; run the driving gates from \
                         the plain kx-gates"
            .to_string()),
        Err(e) => Err(format!(
            "the test build starts only from a kx-gates known to lack uiAccess: {e}"
        )),
    }
}

/// The one line said when the owner's copy is not started again, by whether the test build quit in time.
pub fn not_back(quit: bool) -> String {
    if quit {
        format!("{NAME} did not start again; {START_MENU}")
    } else {
        format!(
            "the test build {SLOW}, so {NAME} is not started again; {START_MENU} once the test build has ended"
        )
    }
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

    /// Error `e` of a run that broke off, with what it left open and how the test build ended.
    pub fn failed(&self, e: &str, left_open: &[String]) -> String {
        format!("{e}; left open: {left_open:?}; test build: {}", self.json())
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
    fn only_a_kx_gates_known_to_lack_ui_access_starts_the_test_build() {
        assert_eq!(plain(Ok(false)), Ok(()));
        assert!(plain(Ok(true)).is_err(), "it would inherit uiAccess");
        let why = "the token said nothing";
        let e = plain(Err(why.into())).expect_err("fails closed");
        assert!(e.contains(why) && e.contains("uiAccess"), "{e}");
    }

    #[test]
    fn a_copy_not_started_again_is_left_to_the_start_menu() {
        for quit in [true, false] {
            let line = not_back(quit);
            assert!(line.contains(START_MENU) && !line.contains('\n'), "{line}");
            assert_eq!(line.contains(SLOW), !quit, "{line}");
        }
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

    #[test]
    fn a_run_that_broke_off_names_what_was_left_open_and_how_the_test_build_ended() {
        let closed = Closed {
            quit: false,
            owner_back: None,
            lines: Ok(Vec::new()),
        };
        let e = closed.failed("no keyboard", &["left open: x".to_string()]);
        assert!(e.starts_with("no keyboard; left open: "), "{e}");
        assert!(
            e.ends_with(&format!("test build: {}", closed.json())),
            "{e}"
        );
    }
}
