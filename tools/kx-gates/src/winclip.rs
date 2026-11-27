//! Removes kx-gates' own test texts and its copies' pictures from Windows clipboard history (Win+V), and nothing else.
#![cfg(windows)]

mod pics;

use windows::ApplicationModel::DataTransfer::{
    Clipboard, ClipboardHistoryItem, ClipboardHistoryItemsResultStatus, StandardDataFormats,
};
use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize};
use windows::Win32::System::SystemInformation::GetSystemTimeAsFileTime;
use windows::core::HSTRING;

use crate::clipkeep;
pub use pics::{forget_pictures, padded};

/// What `forget` may delete: entries made since `since`, no longer than `max_chars`, whose text `is_ours`.
pub struct Scope<'a> {
    /// Start of the run, from `now`; older entries are the owner's.
    pub since: i64,
    /// Longest test text, in characters; longer entries are never read.
    pub max_chars: usize,
    /// True for a test text.
    pub is_ours: &'a (dyn Fn(&str) -> bool + Sync),
}

/// The time now, in the 100 ns units of clipboard history time stamps.
#[allow(unsafe_code, reason = "A plain query.")]
pub fn now() -> i64 {
    // SAFETY: a plain query.
    let ft = unsafe { GetSystemTimeAsFileTime() };
    (i64::from(ft.dwHighDateTime) << 32) | i64::from(ft.dwLowDateTime)
}

/// True when an entry stamped `stamp` with `units` UTF-16 units may be a test text of this run.
fn worth_reading(stamp: i64, since: i64, units: usize, max_units: usize) -> bool {
    stamp >= since && units <= max_units
}

/// `text` without the line break an app may add to a copied line.
pub fn line(text: &str) -> &str {
    text.trim_end_matches(['\r', '\n'])
}

/// Deletes the entries in `scope` from Windows' history; returns how many went.
pub fn forget(scope: &Scope) -> Result<usize, String> {
    guarded(scope, |s| on_com(|| forget_here(s)))
}

/// Runs `delete` on `scope` once `clipkeep::live` allows it, so a test never touches Windows' history.
fn guarded(
    scope: &Scope,
    delete: impl FnOnce(&Scope) -> Result<usize, String>,
) -> Result<usize, String> {
    clipkeep::live();
    delete(scope)
}

/// `f` on a single-threaded COM thread, as Windows' history requires.
#[allow(unsafe_code, reason = "COM start-up on this thread, undone after.")]
fn on_com<T: Send>(f: impl FnOnce() -> Result<T, String> + Send) -> Result<T, String> {
    std::thread::scope(|s| {
        s.spawn(|| {
            // SAFETY: COM start-up on this fresh thread, undone below when it succeeded.
            let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
            if hr.is_err() {
                return Err(format!("CoInitializeEx: {hr:?}"));
            }
            let got = f();
            // SAFETY: pairs the successful CoInitializeEx above.
            unsafe { CoUninitialize() };
            got
        })
        .join()
        .map_err(|_| "the clipboard history thread failed".to_string())?
    })
}

/// Any history failure, as text.
#[allow(clippy::needless_pass_by_value, reason = "It fits `map_err` as it is.")]
fn err(e: windows::core::Error) -> String {
    format!("clipboard history: {e}")
}

/// Windows' history entries, newest first, on the current COM thread; none when history is off.
fn history() -> Result<Option<Vec<ClipboardHistoryItem>>, String> {
    if !Clipboard::IsHistoryEnabled().map_err(err)? {
        return Ok(None);
    }
    let found = Clipboard::GetHistoryItemsAsync()
        .and_then(|op| op.join())
        .map_err(err)?;
    let status = found.Status().map_err(err)?;
    if status != ClipboardHistoryItemsResultStatus::Success {
        return Err(format!("clipboard history: status {}", status.0));
    }
    let list = found.Items().map_err(err)?;
    let items = (0..list.Size().map_err(err)?).filter_map(|i| list.GetAt(i).ok());
    Ok(Some(items.collect()))
}

/// The text of `item` when it may be ours; `None` for anything else, or when it cannot be read.
fn text_of(item: &ClipboardHistoryItem, scope: &Scope, text: &HSTRING) -> Option<String> {
    let stamp = item.Timestamp().ok()?.UniversalTime;
    let content = item.Content().ok()?;
    if !content.Contains(text).ok()? {
        return None;
    }
    let t = content.GetTextAsync().and_then(|op| op.join()).ok()?;
    let max_units = scope.max_chars.saturating_mul(2);
    worth_reading(stamp, scope.since, t.len(), max_units).then(|| t.to_string())
}

/// `forget` on the current thread; 0 when history is off. An entry that cannot be read is skipped.
fn forget_here(scope: &Scope) -> Result<usize, String> {
    let Some(items) = history()? else {
        return Ok(0);
    };
    let text = StandardDataFormats::Text().map_err(err)?;
    let ours =
        |it: &&ClipboardHistoryItem| text_of(it, scope, &text).is_some_and(|t| (scope.is_ours)(&t));
    let gone = |it: &&ClipboardHistoryItem| Clipboard::DeleteItemFromHistory(*it).unwrap_or(false);
    Ok(items.iter().filter(ours).filter(gone).count())
}

/// True when `text`, without its line break, is a leading part of `probe` at least `min_chars` long.
pub fn leads(text: &str, probe: &str, min_chars: usize) -> bool {
    let t = line(text);
    t.chars().count() >= min_chars && probe.starts_with(t)
}

/// True when `text` starts with one of the test prefixes.
pub fn has_prefix(text: &str, prefixes: &[&str]) -> bool {
    prefixes
        .iter()
        .any(|p| !p.is_empty() && text.starts_with(p))
}

#[cfg(test)]
mod tests {
    use std::panic::AssertUnwindSafe;

    use super::*;
    use crate::clipkeep::refused;

    #[test]
    fn a_test_that_reaches_forget_panics_before_windows_history_is_touched() {
        let none = |_: &str| false;
        let scope = Scope {
            since: i64::MAX,
            max_chars: 0,
            is_ours: &none,
        };
        let reached =
            |_: &Scope| -> Result<usize, String> { panic!("Windows' history was reached") };
        // The fake comes first: while the guard is missing it fails here, before `forget` could reach the history.
        assert!(
            refused(AssertUnwindSafe(|| _ = guarded(&scope, reached))),
            "the guard comes first"
        );
        assert!(refused(AssertUnwindSafe(|| _ = forget(&scope))), "forget");
        assert!(
            refused(AssertUnwindSafe(|| _ = forget_pictures(&[(0, i64::MAX)]))),
            "forget_pictures"
        );
    }

    #[test]
    fn only_texts_with_a_test_prefix_are_ours() {
        let p = ["kx-clip-", "kx-hidden-"];
        assert!(has_prefix("kx-clip-07", &p));
        assert!(!has_prefix("my kx-clip-07", &p));
        assert!(!has_prefix("anything", &["", "kx-"]));
    }

    #[test]
    fn only_entries_from_this_run_and_of_test_length_are_read() {
        assert!(worth_reading(100, 100, 10, 10));
        assert!(!worth_reading(99, 100, 10, 10), "older than the run");
        assert!(
            !worth_reading(100, 100, 11, 10),
            "longer than any test text"
        );
    }

    #[test]
    fn line_drops_only_the_trailing_line_break() {
        assert_eq!(line("kx-clip-16\r\n"), "kx-clip-16");
        assert_eq!(line("a\nb"), "a\nb");
    }

    #[test]
    fn a_leading_part_of_the_probe_text_is_ours_when_long_enough() {
        let probe = "Hold still to grab, move";
        assert!(leads("Hold s", probe, 4));
        assert!(leads("Hold still to grab, move", probe, 4));
        assert!(leads("Hold s\r\n", probe, 4));
        assert!(!leads("Hol", probe, 4));
        assert!(!leads("still to", probe, 4));
        assert!(!leads("Hold still to grab, move on", probe, 4));
    }
}
