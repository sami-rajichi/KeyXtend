//! Output files: names, plain paths, file URLs and the results log.
#![cfg(windows)]

use std::io::Write;
use std::path::{Component, Path, PathBuf, Prefix};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde_json::{Value, json};

use crate::win::{poll_until, sleep_ms};

/// Every run appends its JSON line here, inside the out folder.
pub const RESULTS: &str = "results.jsonl";
/// Prefix Windows puts on canonical paths.
const VERBATIM: &str = r"\\?\";
/// Characters escaped in a file URL path.
const URL_ESCAPES: [(char, &str); 3] = [('%', "%25"), (' ', "%20"), ('#', "%23")];

fn since_epoch() -> Duration {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
}

/// Seconds since 1970, to name this run's files.
pub fn stamp() -> u64 {
    since_epoch().as_secs()
}

/// A seed from the clock, for runs without `--seed`.
pub fn clock_seed() -> u64 {
    u64::try_from(since_epoch().as_nanos()).unwrap_or(u64::MAX)
}

/// Prints one progress or result line on stdout.
#[allow(
    clippy::print_stdout,
    reason = "Results and progress on stdout are this tool's output."
)]
pub fn line(args: std::fmt::Arguments<'_>) {
    println!("{args}");
}

/// Like `println!`, through the one place that may print.
macro_rules! say {
    ($($arg:tt)*) => {
        $crate::out::line(format_args!($($arg)*))
    };
}
pub(crate) use say;

/// `r` for a result line: its value, or its error.
pub fn shown<T: Serialize>(r: &Result<T, String>) -> Value {
    r.as_ref()
        .map_or_else(|e| json!({ "error": e }), |v| json!(v))
}

/// Creates the out folder and returns it as a plain absolute path.
pub fn out_dir(dir: &Path) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    plain(dir)
}

/// `path` made absolute, without the `\\?\` prefix.
pub fn plain(path: &Path) -> Result<PathBuf, String> {
    let full = std::fs::canonicalize(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let text = full.to_string_lossy();
    Ok(PathBuf::from(text.strip_prefix(VERBATIM).unwrap_or(&text)))
}

/// `path` in full form as Windows names a running program, from its text alone: `.` and `..` resolved, `/` as `\`
/// and no `\\?\` prefix. Unlike `plain`, the file need not exist.
pub fn full(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            Component::Prefix(p) => match p.kind() {
                Prefix::VerbatimDisk(d) => out.push(format!("{}:", char::from(d))),
                _ => out.push(p.as_os_str()),
            },
            Component::CurDir => {}
            Component::ParentDir => match out.components().next_back() {
                Some(Component::Normal(_)) => drop(out.pop()),
                Some(Component::RootDir | Component::Prefix(_)) => {}
                _ => out.push(".."),
            },
            other => out.push(other),
        }
    }
    out
}

/// A `file:///` URL for an absolute Windows path.
pub fn file_url(path: &Path) -> String {
    let mut url = path.to_string_lossy().replace('\\', "/");
    for (c, code) in URL_ESCAPES {
        url = url.replace(c, code);
    }
    format!("file:///{url}")
}

/// Writes `text` to `path`.
pub fn write(path: &Path, text: &str) -> Result<(), String> {
    write_bytes(path, text.as_bytes())
}

/// Writes `bytes` to `path`.
pub fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), String> {
    std::fs::write(path, bytes).map_err(|e| format!("{}: {e}", path.display()))
}

/// Waits up to `timeout_ms` for `path` to be non-empty, lets the writer finish, then reads it.
pub fn wait_read(path: &Path, timeout_ms: u64, poll_ms: u64) -> Result<String, String> {
    let filled = || {
        std::fs::metadata(path)
            .is_ok_and(|m| m.len() > 0)
            .then_some(())
    };
    poll_until(timeout_ms, poll_ms, filled)
        .ok_or_else(|| format!("{} still empty after {timeout_ms} ms", path.display()))?;
    sleep_ms(poll_ms);
    std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

/// Appends the result line to `results.jsonl` in `out`.
pub fn save_result(out: &Path, line: &str) -> Result<(), String> {
    let file = out.join(RESULTS);
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file)
        .map_err(|e| format!("{}: {e}", file.display()))?;
    writeln!(f, "{line}").map_err(|e| format!("{}: {e}", file.display()))
}

/// A verdict short of a pass: what ran held, but part of the gate did not run or could not be judged.
pub const PARTIAL: &str = "partial";

/// A `pass` value: false when what ran failed, `PARTIAL` when it held but the gate was not `whole`, else true.
pub fn pass(ok: bool, whole: bool) -> Value {
    match (ok, whole) {
        (false, _) => json!(false),
        (true, false) => json!(PARTIAL),
        (true, true) => json!(true),
    }
}

/// True when result line `v` broke off or has a verdict other than a pass, so kx-gates exits with a failure.
pub fn failed(v: &Value) -> bool {
    let pass = v.get("pass");
    v.get("error").is_some() || pass.is_some_and(|p| p != &Value::Bool(true))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_url_uses_slashes_and_escapes() {
        let url = file_url(Path::new(r"D:\a b\c#1%.html"));
        assert_eq!(url, "file:///D:/a%20b/c%231%25.html");
    }

    #[test]
    fn a_result_fails_on_an_error_or_a_verdict_that_did_not_pass() {
        assert!(failed(&json!({ "gate": "G26", "error": "no keyboard" })));
        assert!(failed(&json!({ "gate": "G26", "pass": false })));
        assert!(!failed(&json!({ "gate": "G26", "pass": true })));
        assert!(
            failed(&json!({ "gate": "G8", "pass": "partial" })),
            "a partial result is not a pass"
        );
        assert!(
            !failed(&json!({ "gate": "G2", "clicks": 5 })),
            "a gate without a verdict only reports"
        );
    }

    #[test]
    fn a_result_part_shows_its_value_or_its_error() {
        assert_eq!(shown(&Ok::<u8, String>(3)), json!(3));
        let cut = Err::<u8, String>("cut".to_string());
        assert_eq!(shown(&cut), json!({ "error": "cut" }));
    }

    #[test]
    fn a_verdict_is_partial_only_when_what_ran_held_and_part_did_not_run() {
        assert_eq!(pass(true, true), json!(true));
        assert_eq!(pass(true, false), json!(PARTIAL));
        assert_eq!(pass(false, true), json!(false));
        assert_eq!(pass(false, false), json!(false));
    }

    #[test]
    fn a_path_is_put_in_full_by_its_text_as_windows_names_a_running_program() {
        let p = |s: &str| full(Path::new(s));
        assert_eq!(p(r"D:\a\b\../../c/./d.exe"), PathBuf::from(r"D:\c\d.exe"));
        assert_eq!(
            p(r"D:\..\a"),
            PathBuf::from(r"D:\a"),
            "no higher than the root"
        );
        assert_eq!(
            p(r"\\?\D:\a\b"),
            PathBuf::from(r"D:\a\b"),
            "a canonical one"
        );
        assert_eq!(
            p(r"..\a"),
            PathBuf::from(r"..\a"),
            "a relative one keeps what it cannot drop"
        );
        assert_eq!(p(r"a\..\..\b"), PathBuf::from(r"..\b"));
    }

    #[test]
    fn wait_read_fails_when_the_file_stays_empty() {
        let missing = std::env::temp_dir().join("kx-gates-never-written.txt");
        let err = wait_read(&missing, 0, 0).expect_err("nothing to read");
        assert!(err.contains("still empty after 0 ms"), "{err}");
    }
}
