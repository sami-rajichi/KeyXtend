//! The translate page the owner opened: the one browser window that is new, or newly in front, once Translate's
//! question has gone, and only its address bar, the control a row of `[[g19_product.bars]]` names, found by walking
//! its window's controls without ever entering the page's Document. No other control's value is read, and the
//! address is compared, never kept.
#![cfg(windows)]

use std::collections::HashSet;

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Accessibility::{
    IUIAutomationElement, IUIAutomationTreeWalker, IUIAutomationValuePattern, UIA_CONTROLTYPE_ID,
    UIA_DocumentControlTypeId, UIA_ValuePatternId,
};
use windows::Win32::UI::WindowsAndMessaging::{WS_EX_APPWINDOW, WS_EX_TOOLWINDOW};
use windows::core::BSTR;

use crate::g9win::{ex_of, owner_of};
use crate::g19chips::link::text_of;
use crate::g19chips::procs::browser;
use crate::keymap::Reader;
use crate::p10cfg::Bar;
use crate::win;

/// True for a window the taskbar shows, as Windows decides: one marked an app window, or an unowned one that is no
/// tool window. Hover cards, tooltips and other popups are left out.
pub fn on_taskbar(ex: u32, owned: bool) -> bool {
    ex & WS_EX_APPWINDOW.0 != 0 || (ex & WS_EX_TOOLWINDOW.0 == 0 && !owned)
}

/// The browser windows a person sees now, as the taskbar shows them.
pub fn windows(browsers: &[String]) -> Vec<HWND> {
    let shown = |w: &HWND| on_taskbar(ex_of(*w), owner_of(*w).is_some());
    let is_browser = |w: &HWND| win::program_name(*w).is_some_and(|p| browser(&p, browsers));
    let seen = win::seen_windows().into_iter().filter(shown);
    seen.filter(is_browser).collect()
}

/// The window to read among browser windows `now`: none while Translate's question is `asking`; else the new one in
/// front, else the only new one, else one that came in front since `front_before`; none while that is unclear.
pub fn pick(
    before: &HashSet<isize>,
    front_before: isize,
    now: &[isize],
    front: isize,
    asking: bool,
) -> Option<isize> {
    if asking {
        return None;
    }
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

/// The row of `bars` for browser program `exe`.
pub fn row<'a>(bars: &'a [Bar], exe: &str) -> Option<&'a Bar> {
    bars.iter().find(|b| browser(exe, &b.browsers))
}

/// True when a control with `id`, `class` and `name` is `bar`'s address bar; an empty value never matches.
pub fn fits(bar: &Bar, id: &str, class: &str, name: &str) -> bool {
    let has = |list: &[String], v: &str| !v.is_empty() && list.iter().any(|x| x == v);
    has(&bar.ids, id) || has(&bar.classes, class) || has(&bar.names, name)
}

/// The value of control `el`, when it has one.
#[allow(unsafe_code, reason = "Plain pattern reads on a live element.")]
fn value(el: &IUIAutomationElement) -> Option<String> {
    // SAFETY: plain pattern reads on a live element; a control without the pattern gives an error.
    unsafe {
        let p: IUIAutomationValuePattern = el.GetCurrentPatternAs(UIA_ValuePatternId).ok()?;
        p.CurrentValue().ok().map(|b| b.to_string())
    }
}

/// What a control is matched by: its type, then its id, class and name; never its value.
#[allow(unsafe_code, reason = "Plain property reads on a live element.")]
fn facts(el: &IUIAutomationElement) -> (UIA_CONTROLTYPE_ID, [String; 3]) {
    let text = |r: windows::core::Result<BSTR>| r.map(|b| b.to_string()).unwrap_or_default();
    // SAFETY: plain property reads on a live element; a failed read gives an empty value.
    unsafe {
        let kind = el.CurrentControlType().unwrap_or_default();
        let id = text(el.CurrentAutomationId());
        (
            kind,
            [id, text(el.CurrentClassName()), text(el.CurrentName())],
        )
    }
}

/// The first control under `el` that `bar` fits, `depth` levels down at most, never inside a Document.
#[allow(unsafe_code, reason = "Plain tree-walker calls on live elements.")]
fn find(
    walker: &IUIAutomationTreeWalker,
    el: &IUIAutomationElement,
    bar: &Bar,
    depth: u8,
) -> Option<IUIAutomationElement> {
    // SAFETY: plain walker calls on live elements; the end of a level gives an error, read as none.
    let mut child = unsafe { walker.GetFirstChildElement(el) }.ok();
    while let Some(c) = child {
        let (kind, [id, class, name]) = facts(&c);
        if fits(bar, &id, &class, &name) {
            return Some(c);
        }
        let deeper = kind != UIA_DocumentControlTypeId && depth > 1;
        if let Some(found) = deeper.then(|| find(walker, &c, bar, depth - 1)).flatten() {
            return Some(found);
        }
        // SAFETY: as above.
        child = unsafe { walker.GetNextSiblingElement(&c) }.ok();
    }
    None
}

/// The value of window `hwnd`'s address bar as `bar` finds it outside the page; none with no such bar.
#[allow(
    unsafe_code,
    reason = "UI Automation calls on objects this client made."
)]
fn bar_value(reader: &Reader, hwnd: HWND, bar: &Bar, depth: u8) -> Option<String> {
    // SAFETY: plain calls on this client; a dead window gives an error.
    let (walker, root) = unsafe {
        (
            reader.auto().ControlViewWalker(),
            reader.auto().ElementFromHandle(hwnd),
        )
    };
    let el = find(&walker.ok()?, &root.ok()?, bar, depth)?;
    Some(value(&el).unwrap_or_default())
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

/// What one look at a browser window's address bar found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Look {
    /// The address holds the text.
    Page(Shown),
    /// The address holds no text of ours yet.
    Elsewhere,
    /// No control matched the browser's row.
    NoBar,
    /// No row names the browser.
    NoRow,
}

/// The decoded text of query key `key` in address `v`, when it starts as `long` does.
fn ours(v: &str, key: &str, long: &str) -> Option<String> {
    let first: String = long.chars().take(1).collect();
    text_of(v, key).filter(|t| !first.is_empty() && t.starts_with(&first))
}

/// What window `hwnd`'s address bar, as its browser's row of `bars` finds it, shows of key `key` against `long`.
pub fn page(
    reader: &Reader,
    hwnd: HWND,
    bars: &[Bar],
    (key, long): (&str, &str),
    depth: u8,
) -> Look {
    let Some(bar) = win::program_name(hwnd).and_then(|p| row(bars, &p)) else {
        return Look::NoRow;
    };
    let Some(v) = bar_value(reader, hwnd, bar, depth) else {
        return Look::NoBar;
    };
    let shown = |text: String| {
        let (chars, whole) = (text.chars().count(), text == long);
        Look::Page(Shown { hwnd, chars, whole })
    };
    ours(&v, key, long).map_or(Look::Elsewhere, shown)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shipped table.
    fn bars() -> Vec<Bar> {
        crate::config::load().expect("loads").g19_product.bars
    }

    #[test]
    fn only_an_address_whose_text_starts_as_the_long_line_counts() {
        let long = "حدد هذا";
        let ours_ = |v: &str| ours(v, "text", long);
        assert_eq!(
            ours_("kx.test/?text=%D8%AD%D8%AF%D8%AF&op=t").as_deref(),
            Some("حدد"),
            "cut short"
        );
        assert_eq!(ours_("kx.test/?q=1"), None);
        assert_eq!(ours("kx.test/?text=%D8%AD", "text", ""), None, "no line");
    }

    #[test]
    fn the_window_read_is_the_new_one_or_the_one_newly_in_front_and_never_a_guess() {
        let before: HashSet<isize> = [1, 2].into_iter().collect();
        let pick =
            |front_before, now: &[isize], front| pick(&before, front_before, now, front, false);
        assert_eq!(pick(9, &[1, 2, 3], 9), Some(3), "a new window");
        assert_eq!(pick(9, &[1, 2, 3, 4], 4), Some(4), "the new one in front");
        assert_eq!(pick(9, &[1, 2, 3, 4], 9), None, "two new, neither in front");
        assert_eq!(pick(9, &[1, 2], 2), Some(2), "a new tab brought it forward");
        assert_eq!(pick(2, &[1, 2], 2), None, "it was in front already");
        assert_eq!(pick(9, &[1, 2], 9), None, "nothing changed");
    }

    #[test]
    fn no_window_is_picked_while_the_question_still_asks() {
        let before: HashSet<isize> = [1].into_iter().collect();
        assert_eq!(
            pick(&before, 9, &[1, 3], 3, true),
            None,
            "a new one in front"
        );
        assert_eq!(pick(&before, 9, &[1], 1, true), None, "one newly in front");
        assert_eq!(
            pick(&before, 9, &[1, 3], 3, false),
            Some(3),
            "once it has gone"
        );
    }

    #[test]
    fn only_the_control_a_row_names_is_the_address_bar() {
        let table = bars();
        let chrome = row(&table, "Chrome.EXE").expect("a row for Chrome");
        let class = chrome.classes.first().expect("a class");
        assert!(fits(chrome, "", class, ""));
        let name = chrome.names.first().expect("a name");
        assert!(fits(chrome, "", "", name));
        assert!(!fits(chrome, "", "Edit", "Find"), "another edit");
        assert!(!fits(chrome, "", "", ""), "an empty value");
        let firefox = row(&table, "firefox.exe").expect("a row for Firefox");
        let id = firefox.ids.first().expect("an id");
        assert!(fits(firefox, id, "", "") && !fits(firefox, "", class, ""));
        assert!(row(&table, "notepad.exe").is_none());
    }

    #[test]
    fn hover_cards_tooltips_and_popups_are_no_browser_window() {
        let (app, tool) = (WS_EX_APPWINDOW.0, WS_EX_TOOLWINDOW.0);
        assert!(on_taskbar(0, false), "a plain window");
        assert!(!on_taskbar(tool, false), "a tool window");
        assert!(!on_taskbar(0, true), "an owned popup");
        assert!(on_taskbar(app, true), "marked an app window");
    }
}
