//! The translate page the owner opened: the one browser window that is new, or newly in front, after the owner's
//! Yes, and only its address bar, found by walking its window's controls without ever entering the page's Document.
//! No other window is read, and the address is compared, never kept.
#![cfg(windows)]

use std::collections::HashSet;

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Accessibility::{
    IUIAutomationElement, IUIAutomationTreeWalker, IUIAutomationValuePattern,
    UIA_DocumentControlTypeId, UIA_EditControlTypeId, UIA_ValuePatternId,
};

use crate::g19chips::link::text_of;
use crate::g19chips::procs::browser;
use crate::keymap::Reader;
use crate::win;

/// The browser windows on screen now.
pub fn windows(browsers: &[String]) -> Vec<HWND> {
    let is_browser = |w: &HWND| win::program_name(*w).is_some_and(|p| browser(&p, browsers));
    win::top_windows().into_iter().filter(is_browser).collect()
}

/// The window to read among browser windows `now`: the new one in front, else the only new one, else one that came
/// in front since `front_before`; none while that is unclear.
pub fn pick(
    before: &HashSet<isize>,
    front_before: isize,
    now: &[isize],
    front: isize,
) -> Option<isize> {
    let new: Vec<isize> = now
        .iter()
        .copied()
        .filter(|w| !before.contains(w))
        .collect();
    if new.contains(&front) {
        return Some(front);
    }
    if let [only] = new[..] {
        return Some(only);
    }
    (now.contains(&front) && front != front_before).then_some(front)
}

/// The value of edit `el`, when it has one.
#[allow(unsafe_code, reason = "Plain pattern reads on a live element.")]
fn value(el: &IUIAutomationElement) -> Option<String> {
    // SAFETY: plain pattern reads on a live element; a box without the pattern gives an error.
    unsafe {
        let p: IUIAutomationValuePattern = el.GetCurrentPatternAs(UIA_ValuePatternId).ok()?;
        p.CurrentValue().ok().map(|b| b.to_string())
    }
}

/// The values of the edits under `el`, `depth` levels down at most, never inside a Document.
#[allow(unsafe_code, reason = "Plain tree-walker calls on live elements.")]
fn edits(
    walker: &IUIAutomationTreeWalker,
    el: &IUIAutomationElement,
    depth: u8,
    out: &mut Vec<String>,
) {
    // SAFETY: plain walker calls on live elements; the end of a level gives an error, read as none.
    let mut child = unsafe { walker.GetFirstChildElement(el) }.ok();
    while let Some(c) = child {
        // SAFETY: a plain property read on a live element.
        let kind = unsafe { c.CurrentControlType() }.unwrap_or_default();
        if kind == UIA_EditControlTypeId {
            out.extend(value(&c));
        } else if kind != UIA_DocumentControlTypeId && depth > 1 {
            edits(walker, &c, depth - 1, out);
        }
        // SAFETY: as above.
        child = unsafe { walker.GetNextSiblingElement(&c) }.ok();
    }
}

/// The values of window `hwnd`'s edits outside its page, among them its address bar's.
#[allow(
    unsafe_code,
    reason = "UI Automation calls on objects this client made."
)]
fn bar(reader: &Reader, hwnd: HWND, depth: u8) -> Result<Vec<String>, String> {
    let e = |e: windows::core::Error| format!("UI Automation: {e}");
    // SAFETY: plain calls on this client; a dead window gives an error.
    let (walker, root) = unsafe {
        (
            reader.auto().ControlViewWalker(),
            reader.auto().ElementFromHandle(hwnd),
        )
    };
    let (walker, root) = (walker.map_err(e)?, root.map_err(e)?);
    let mut out = Vec::new();
    edits(&walker, &root, depth, &mut out);
    Ok(out)
}

/// What the page shows of the text: its characters, and whether it is `long` whole.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shown {
    /// The browser window.
    pub hwnd: HWND,
    /// The decoded text's characters.
    pub chars: usize,
    /// It is the long line, whole.
    pub whole: bool,
}

/// The decoded text of query key `key` in one of `values`, when it starts as `long` does.
fn ours(values: &[String], key: &str, long: &str) -> Option<String> {
    let first: String = long.chars().take(1).collect();
    values
        .iter()
        .filter_map(|v| text_of(v, key))
        .find(|t| !first.is_empty() && t.starts_with(&first))
}

/// Window `hwnd`'s address, when it holds key `key` with a text that starts as `long` does.
pub fn page(reader: &Reader, hwnd: HWND, key: &str, long: &str, depth: u8) -> Option<Shown> {
    let text = ours(&bar(reader, hwnd, depth).ok()?, key, long)?;
    Some(Shown {
        hwnd,
        chars: text.chars().count(),
        whole: text == long,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_an_address_whose_text_starts_as_the_long_line_counts() {
        let long = "حدد هذا";
        let values = [
            "kx.test/?q=1".to_string(),
            "kx.test/?text=%D8%AD%D8%AF%D8%AF&op=t".to_string(),
        ];
        assert_eq!(
            ours(&values, "text", long).as_deref(),
            Some("حدد"),
            "cut short"
        );
        assert_eq!(ours(&values[..1], "text", long), None);
        assert_eq!(ours(&values, "text", ""), None, "no line");
    }

    #[test]
    fn the_window_read_is_the_new_one_or_the_one_newly_in_front_and_never_a_guess() {
        let before: HashSet<isize> = [1, 2].into_iter().collect();
        assert_eq!(pick(&before, 9, &[1, 2, 3], 9), Some(3), "a new window");
        assert_eq!(
            pick(&before, 9, &[1, 2, 3, 4], 4),
            Some(4),
            "the new one in front"
        );
        assert_eq!(
            pick(&before, 9, &[1, 2, 3, 4], 9),
            None,
            "two new, neither in front"
        );
        assert_eq!(
            pick(&before, 9, &[1, 2], 2),
            Some(2),
            "a new tab brought it forward"
        );
        assert_eq!(
            pick(&before, 2, &[1, 2], 2),
            None,
            "it was in front already"
        );
        assert_eq!(pick(&before, 9, &[1, 2], 9), None, "nothing changed");
    }
}
