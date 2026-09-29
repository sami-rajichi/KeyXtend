//! The log file: rotation, the size cap, thread safety and the errors of opening it.

use super::tests::SECRET;
use super::*;
use kx_test_support::tempdir::TempDir;
use std::path::Path;
use tracing::info;

fn open_in(dir: &TempDir, max: u64) -> (Files, LogFile) {
    let files = Files::new(dir.path());
    let log = LogFile::open(&files, max).unwrap();
    (files, log)
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}

#[test]
fn open_makes_the_logs_folder_and_an_empty_log() {
    let dir = TempDir::new("log-new").unwrap();
    let (files, _log) = open_in(&dir, u64::MAX);
    assert_eq!(read(&files.log), "");
    assert!(!files.log_previous.exists());
}

#[test]
fn an_existing_log_becomes_the_previous_log_and_the_new_log_is_empty() {
    let dir = TempDir::new("log-rotate").unwrap();
    let files = Files::new(dir.path());
    fs::create_dir_all(&files.logs).unwrap();
    fs::write(&files.log, "last run").unwrap();
    fs::write(&files.log_previous, "older run").unwrap();
    let _log = LogFile::open(&files, u64::MAX).unwrap();
    assert_eq!(read(&files.log_previous), "last run");
    assert_eq!(read(&files.log), "");
}

#[test]
fn a_file_in_place_of_the_logs_folder_is_an_error() {
    let dir = TempDir::new("log-nofolder").unwrap();
    let files = Files::new(dir.path());
    fs::write(&files.logs, "not a folder").unwrap();
    assert!(LogFile::open(&files, u64::MAX).is_err());
}

#[test]
fn a_last_log_that_cannot_move_is_an_error_and_stays() {
    let dir = TempDir::new("log-nomove").unwrap();
    let files = Files::new(dir.path());
    fs::create_dir_all(&files.log_previous).unwrap();
    fs::write(files.log_previous.join("blocker"), "x").unwrap();
    fs::write(&files.log, "last run").unwrap();
    assert!(LogFile::open(&files, u64::MAX).is_err());
    assert_eq!(read(&files.log), "last run");
}

#[test]
fn a_third_start_drops_the_oldest_run() {
    let dir = TempDir::new("log-third").unwrap();
    for run in ["first", "second", "third"] {
        let (files, log) = open_in(&dir, u64::MAX);
        log.make_writer().write_all(run.as_bytes()).unwrap();
        assert_eq!(read(&files.log), run);
    }
    let files = Files::new(dir.path());
    assert_eq!(read(&files.log_previous), "second");
}

/// A cap of 20 bytes and the ten-byte line the cap tests write.
const CAP: u64 = 20;
const LINE: &[u8] = b"0123456789";

fn size_of(text: &str) -> u64 {
    u64::try_from(text.len()).unwrap()
}

#[test]
fn writing_past_the_cap_leaves_one_final_line_and_drops_the_rest() {
    let dir = TempDir::new("log-cap").unwrap();
    let (files, log) = open_in(&dir, CAP);
    let mut out = log.make_writer();
    for _ in 0..5 {
        assert_eq!(out.write(LINE).unwrap(), LINE.len());
    }
    let text = read(&files.log);
    assert_eq!(text, format!("01234567890123456789{LIMIT_LINE}"));
    assert!(size_of(&text) <= CAP + size_of(LIMIT_LINE));
}

#[test]
fn a_write_that_exactly_fills_the_cap_is_kept() {
    let dir = TempDir::new("log-exact").unwrap();
    let (files, log) = open_in(&dir, CAP);
    let mut out = log.make_writer();
    out.write_all(LINE).unwrap();
    out.write_all(LINE).unwrap();
    assert_eq!(size_of(&read(&files.log)), CAP);
}

#[test]
fn one_write_larger_than_the_cap_leaves_only_the_final_line() {
    let dir = TempDir::new("log-big").unwrap();
    let (files, log) = open_in(&dir, 5);
    log.make_writer().write_all(LINE).unwrap();
    assert_eq!(read(&files.log), LIMIT_LINE);
}

#[test]
fn a_zero_cap_writes_only_the_final_line() {
    let dir = TempDir::new("log-zero").unwrap();
    let (files, log) = open_in(&dir, 0);
    let mut out = log.make_writer();
    out.write_all(LINE).unwrap();
    out.write_all(LINE).unwrap();
    assert_eq!(read(&files.log), LIMIT_LINE);
}

#[test]
fn the_cap_holds_for_events_through_the_subscriber() {
    let dir = TempDir::new("log-events").unwrap();
    let (files, log) = open_in(&dir, CAP * 10);
    tracing::subscriber::with_default(subscriber(LevelFilter::INFO, log), || {
        for _ in 0..100 {
            info!("a line of text");
        }
    });
    let text = read(&files.log);
    assert!(text.ends_with(LIMIT_LINE), "{text}");
    assert_eq!(text.matches(LIMIT_LINE).count(), 1);
    assert!(size_of(&text) <= CAP * 10 + size_of(LIMIT_LINE));
}

#[test]
fn the_file_holds_redacted_lines() {
    let dir = TempDir::new("log-redact").unwrap();
    let (files, log) = open_in(&dir, u64::MAX);
    tracing::subscriber::with_default(subscriber(LevelFilter::INFO, log), || {
        info!(password = SECRET, "x");
    });
    let text = read(&files.log);
    assert!(text.contains(REDACTED_MARKER), "{text}");
    assert!(!text.contains(SECRET), "{text}");
}

#[test]
fn writers_on_two_threads_never_split_a_line() {
    let dir = TempDir::new("log-threads").unwrap();
    let (files, log) = open_in(&dir, u64::MAX);
    std::thread::scope(|scope| {
        for id in 0..4 {
            let log = &log;
            scope.spawn(move || {
                let line = format!("thread {id}\n");
                for _ in 0..25 {
                    log.make_writer().write_all(line.as_bytes()).unwrap();
                }
            });
        }
    });
    let text = read(&files.log);
    assert_eq!(text.lines().count(), 100);
    assert!(text.lines().all(|line| line.starts_with("thread ")));
}

/// A writer that fails every write, to check the sink swallows the errors after the first.
struct Broken;

impl Write for Broken {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("disk full"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::other("disk full"))
    }
}

#[test]
fn after_the_first_error_every_write_reports_success() {
    let mut sink = Sink::new(Broken, u64::MAX);
    assert!(sink.put(LINE).is_err());
    assert_eq!(sink.put(LINE).unwrap(), LINE.len());
    assert_eq!(sink.put(LINE).unwrap(), LINE.len());
    assert!(sink.flush().is_ok());
}
