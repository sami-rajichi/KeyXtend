use super::*;
use kx_module_api::REDACTED_MARKER;
use kx_test_support::tempdir::TempDir;
use std::fs;

/// The kernel's settings with log level `level` and a 1 MiB cap.
fn settings(level: &str) -> KernelSettings {
    KernelSettings {
        disabled: Vec::new(),
        log_level: level.to_owned(),
        log_max_mb: 1,
    }
}

/// The files of a fresh data folder, which lives as long as the returned guard.
fn data() -> (TempDir, Files) {
    let dir = TempDir::new("app-log").unwrap();
    let files = Files::new(dir.path());
    (dir, files)
}

/// What this run's log file holds once `f` has logged through `log`.
fn logged(log: &Log, files: &Files, f: impl FnOnce()) -> String {
    tracing::dispatcher::with_default(&log.dispatch, f);
    fs::read_to_string(&files.log).unwrap()
}

#[test]
fn an_event_lands_in_this_runs_file_at_the_level_set() {
    let (_dir, files) = data();
    let log = open(&settings("warn"), &files).unwrap();
    assert!(log.no_file.is_none());
    let text = logged(&log, &files, || {
        tracing::info!("below the level");
        tracing::warn!(module = "keyboard", "at the level");
    });
    assert!(
        text.contains("at the level") && text.contains("keyboard"),
        "{text}"
    );
    assert!(!text.contains("below the level"), "{text}");
}

#[test]
fn the_last_run_moves_to_the_previous_file() {
    let (_dir, files) = data();
    fs::create_dir_all(&files.logs).unwrap();
    fs::write(&files.log, "the run before\n").unwrap();
    let log = open(&settings("info"), &files).unwrap();
    assert!(log.no_file.is_none());
    let before = fs::read_to_string(&files.log_previous).unwrap();
    assert_eq!(before, "the run before\n");
    assert!(logged(&log, &files, || {}).is_empty(), "a new file");
}

#[test]
fn typed_text_is_redacted_in_the_file() {
    let (_dir, files) = data();
    let log = open(&settings("info"), &files).unwrap();
    let text = logged(&log, &files, || {
        tracing::info!(typed_text = "hunter2", "typed");
    });
    assert!(text.contains(REDACTED_MARKER), "{text}");
    assert!(!text.contains("hunter2"), "{text}");
}

#[test]
fn a_file_that_cannot_open_leaves_the_run_on_standard_error() {
    let (_dir, files) = data();
    fs::write(&files.logs, "a file where the logs folder goes").unwrap();
    let log = open(&settings("info"), &files).unwrap();
    assert!(log.no_file.is_some(), "the reason is kept to say once");
    assert!(!files.log.exists());
}

#[test]
fn a_name_that_is_no_level_opens_nothing() {
    let (_dir, files) = data();
    assert!(open(&settings("loud"), &files).is_none());
    assert!(!files.logs.exists(), "nothing was made");
}

#[test]
fn with_no_loaded_settings_the_kernel_defaults_open_the_log() {
    let (_dir, files) = data();
    let chosen = or_defaults(None).unwrap();
    assert_eq!(chosen, KernelSettings::from_defaults().unwrap());
    assert!(open(&chosen, &files).is_some());
    let mine = settings("warn");
    assert_eq!(or_defaults(Some(mine.clone())), Some(mine));
}
