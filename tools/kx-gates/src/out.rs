//! Output files: names, plain paths, file URLs and the results log.
#![cfg(windows)]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::win::{poll_until, sleep_ms};

/// Every run appends its JSON line here, inside the out folder.
const RESULTS: &str = "results.jsonl";
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
    since_epoch().as_nanos() as u64
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
    std::fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_url_uses_slashes_and_escapes() {
        let url = file_url(Path::new(r"D:\a b\c#1%.html"));
        assert_eq!(url, "file:///D:/a%20b/c%231%25.html");
    }

    #[test]
    fn wait_read_fails_when_the_file_stays_empty() {
        let missing = std::env::temp_dir().join("kx-gates-never-written.txt");
        let err = wait_read(&missing, 0, 0).expect_err("nothing to read");
        assert!(err.contains("still empty after 0 ms"), "{err}");
    }
}
