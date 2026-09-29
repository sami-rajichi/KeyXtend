//! Removes kx-gates' own test texts from Windows clipboard history (Win+V), and nothing else.
#![cfg(windows)]

use windows::ApplicationModel::DataTransfer::{
    Clipboard, ClipboardHistoryItem, ClipboardHistoryItemsResultStatus, StandardDataFormats,
};
use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize};
use windows::Win32::System::SystemInformation::GetSystemTimeAsFileTime;
use windows::core::HSTRING;

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

/// Deletes the entries in `scope`, on a single-threaded COM thread as Windows requires; returns how many went.
#[allow(unsafe_code, reason = "COM start-up on this thread, undone after.")]
pub fn forget(scope: &Scope) -> Result<usize, String> {
    std::thread::scope(|s| {
        s.spawn(|| {
            // SAFETY: COM start-up on this fresh thread, undone below when it succeeded.
            let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
            if hr.is_err() {
                return Err(format!("CoInitializeEx: {hr:?}"));
            }
            let gone = forget_here(scope);
            // SAFETY: pairs the successful CoInitializeEx above.
            unsafe { CoUninitialize() };
            gone
        })
        .join()
        .map_err(|_| "the clipboard history thread failed".to_string())?
    })
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
    let err = |e: windows::core::Error| format!("clipboard history: {e}");
    if !Clipboard::IsHistoryEnabled().map_err(err)? {
        return Ok(0);
    }
    let found = Clipboard::GetHistoryItemsAsync()
        .and_then(|op| op.join())
        .map_err(err)?;
    let status = found.Status().map_err(err)?;
    if status != ClipboardHistoryItemsResultStatus::Success {
        return Err(format!("clipboard history: status {}", status.0));
    }
    let (items, text) = (
        found.Items().map_err(err)?,
        StandardDataFormats::Text().map_err(err)?,
    );
    let mut gone = 0;
    for i in 0..items.Size().map_err(err)? {
        let Ok(item) = items.GetAt(i) else { continue };
        let ours = text_of(&item, scope, &text).is_some_and(|t| (scope.is_ours)(&t));
        if ours && Clipboard::DeleteItemFromHistory(&item).unwrap_or(false) {
            gone += 1;
        }
    }
    Ok(gone)
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
    use super::*;

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
