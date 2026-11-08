//! The translate page the owner opened: the one browser window that is new, or newly in front, once Translate's
//! question has gone, and only its address bar, the Edit a row of `[[g19_product.bars]]` names. The walk never enters
//! the page's Document or the tabs, and the address is compared, never kept.
#![cfg(windows)]

use std::collections::HashSet;

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Accessibility::{
    IUIAutomationElement, IUIAutomationTreeWalker, IUIAutomationValuePattern, UIA_CONTROLTYPE_ID,
    UIA_DocumentControlTypeId, UIA_EditControlTypeId, UIA_TabControlTypeId,
    UIA_TabItemControlTypeId, UIA_ValuePatternId,
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

/// What the walk does at a control of one type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// A Document, Tab or `TabItem`: never entered, nothing in it read.
    Skip,
    /// An Edit: matched against the row, never entered.
    Edit,
    /// Any other control: entered.
    Enter,
}

/// The walk's step at a control of type `kind`, so the page and the owner's tab titles are never read.
pub fn step(kind: UIA_CONTROLTYPE_ID) -> Step {
    match kind {
        k if [
            UIA_DocumentControlTypeId,
            UIA_TabControlTypeId,
            UIA_TabItemControlTypeId,
        ]
        .contains(&k) =>
        {
            Step::Skip
        }
        k if k == UIA_EditControlTypeId => Step::Edit,
        _ => Step::Enter,
    }
}

/// True when an Edit with `id` and `class` is `bar`'s address bar, else by its name, which `name` reads only when the
/// row lists names; an empty value never matches.
pub fn fits(bar: &Bar, id: &str, class: &str, name: impl FnOnce() -> String) -> bool {
    let has = |list: &[String], v: &str| !v.is_empty() && list.iter().any(|x| x == v);
    has(&bar.ids, id)
        || has(&bar.classes, class)
        || (!bar.names.is_empty() && has(&bar.names, &name()))
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

/// A text property of `el` from `read`; empty when it does not read.
fn text(read: windows::core::Result<BSTR>) -> String {
    read.map(|b| b.to_string()).unwrap_or_default()
}

/// True when Edit `el` is `bar`'s address bar: its id and class read first, its name only when the row needs it.
#[allow(unsafe_code, reason = "Plain property reads on a live element.")]
fn is_bar(el: &IUIAutomationElement, bar: &Bar) -> bool {
    // SAFETY: plain property reads on a live element; a failed read gives an empty value.
    let (id, class) = unsafe { (text(el.CurrentAutomationId()), text(el.CurrentClassName())) };
    // SAFETY: as above.
    fits(bar, &id, &class, || text(unsafe { el.CurrentName() }))
}

/// The first Edit under `el` that `bar` fits, `depth` levels down at most, never inside a Document, Tab or `TabItem`.
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
        // SAFETY: a plain property read on a live element.
        let kind = unsafe { c.CurrentControlType() }.unwrap_or_default();
        match step(kind) {
            Step::Edit if is_bar(&c, bar) => return Some(c),
            Step::Enter if depth > 1 => {
                if let Some(found) = find(walker, &c, bar, depth - 1) {
                    return Some(found);
                }
            }
            _ => {}
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
mod tests;
