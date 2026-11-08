//! The translate page the owner opened: the browser window whose address bar holds the site's link with the gate's own
//! long Arabic line whole, found by the values of its edit boxes. Other windows' addresses are compared, never kept.
#![cfg(windows)]

use windows::Win32::Foundation::HWND;
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::Accessibility::{
    IUIAutomationElement, IUIAutomationValuePattern, TreeScope_Descendants,
    UIA_ControlTypePropertyId, UIA_EditControlTypeId, UIA_ValuePatternId,
};

use crate::g19chips::link::text_of;
use crate::g19chips::procs::browser;
use crate::g23::live::each;
use crate::keymap::Reader;
use crate::win;

/// The values of the edit boxes in window `hwnd`, among them its address bar's.
#[allow(
    unsafe_code,
    reason = "UI Automation calls on objects this client made."
)]
fn values(reader: &Reader, hwnd: HWND) -> Result<Vec<String>, String> {
    let want = VARIANT::from(UIA_EditControlTypeId.0);
    // SAFETY: a plain call on this client, with a live value.
    let cond = unsafe {
        reader
            .auto()
            .CreatePropertyCondition(UIA_ControlTypePropertyId, &want)
    }
    .map_err(|e| format!("UI Automation: {e}"))?;
    let value = |el: &IUIAutomationElement| {
        // SAFETY: plain pattern reads on a live element; a box without the pattern gives an empty error.
        unsafe {
            let p: IUIAutomationValuePattern = el.GetCurrentPatternAs(UIA_ValuePatternId)?;
            p.CurrentValue().map(|b| b.to_string())
        }
    };
    each(reader, hwnd, TreeScope_Descendants, &cond, value)
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

/// The browser window among `browsers` whose address holds key `key` with a text that starts as `long` does.
pub fn page(reader: &Reader, browsers: &[String], key: &str, long: &str) -> Option<Shown> {
    let is_browser = |w: &HWND| win::program_name(*w).is_some_and(|p| browser(&p, browsers));
    win::top_windows()
        .into_iter()
        .filter(is_browser)
        .find_map(|hwnd| {
            let text = ours(&values(reader, hwnd).ok()?, key, long)?;
            Some(Shown {
                hwnd,
                chars: text.chars().count(),
                whole: text == long,
            })
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
}
