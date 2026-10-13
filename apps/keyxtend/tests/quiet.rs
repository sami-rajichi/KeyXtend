//! The quiet panic hook logs where a panic happened and never what it said, says the place on standard error too until
//! the run's log is set up, and then lets the mouse hook go. It is the process's one hook, so this binary holds a single
//! test.

#![allow(clippy::unwrap_used, reason = "a failed setup should fail the test")]

use keyxtend::log::open;
use keyxtend::quiet;
use kx_kernel::KernelSettings;
use kx_settings::Files;
use kx_test_support::tempdir::TempDir;
use std::panic::{AssertUnwindSafe, Location, catch_unwind};
use std::sync::atomic::{AtomicUsize, Ordering};

/// What the panic says; it must never reach the log.
const SECRET: &str = "the user typed hunter2";

/// How many times the hook let the mouse hook go.
static RELEASED: AtomicUsize = AtomicUsize::new(0);

/// The test double of the mouse hook's release: it only counts.
fn release() {
    RELEASED.fetch_add(1, Ordering::SeqCst);
}

/// How many times the hook let go so far.
fn released() -> usize {
    RELEASED.load(Ordering::SeqCst)
}

/// Notes where it is called, then panics there with `SECRET`, so the two places are the same.
#[track_caller]
fn fail(at: &mut Option<Location<'static>>) {
    *at = Some(*Location::caller());
    panic!("{SECRET}");
}

#[test]
fn a_panic_logs_its_place_and_never_its_payload_then_lets_the_mouse_go() {
    let dir = TempDir::new("app-quiet").unwrap();
    let files = Files::new(dir.path());
    let settings = KernelSettings {
        disabled: Vec::new(),
        log_level: "error".to_owned(),
        log_max_mb: 1,
    };
    let log = open(&settings, &files).unwrap();
    quiet::install(release);
    let mut at = None;
    tracing::dispatcher::with_default(&log.dispatch, || {
        assert!(catch_unwind(AssertUnwindSafe(|| fail(&mut at))).is_err());
        quiet::report(None, &mut Vec::new());
    });
    assert_eq!(released(), 1, "one panic, one release");
    let at = at.unwrap();
    let text = std::fs::read_to_string(&files.log).unwrap();
    let file = format!("{:?}", at.file());
    assert!(text.contains(&file), "the file: {text}");
    assert!(
        text.contains(&format!("line={}", at.line())),
        "the line: {text}"
    );
    assert!(
        !text.contains(SECRET) && !text.contains("hunter2"),
        "{text}"
    );
    assert_eq!(text.lines().count(), 2, "one line each: {text}");
    let mut early = Vec::new();
    quiet::report(Some(&at), &mut early);
    let said = String::from_utf8(early).unwrap();
    let place = format!("{}:{}:{}", at.file(), at.line(), at.column());
    assert!(said.contains(&place) && !said.contains(SECRET), "{said}");
    assert!(keyxtend::log::install(log), "the run's log is set up");
    let mut later = Vec::new();
    quiet::report(Some(&at), &mut later);
    assert!(later.is_empty(), "the log has it now");
    assert_eq!(released(), 1, "a report alone lets nothing go");
}
