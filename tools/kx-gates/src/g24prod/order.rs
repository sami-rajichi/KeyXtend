//! The order check (ADR-0059): the copy shows a frozen screen on each screen, and every shown window of ours, or the
//! Save dialog, lies over all of them. A failure names which window lay under which, ours by title, others by class.
#![cfg(windows)]

use windows::Win32::Foundation::HWND;

use super::snip::Snip;
use super::wins;
use crate::win;

/// Waits up to `[g24] order_ms` for a frozen screen on each screen, with each window `above` names over every one;
/// else the faults last read. A read without frozen screens never passes.
fn held(s: &Snip, above: impl Fn(HWND) -> bool) -> Result<(), String> {
    let mut last = Vec::new();
    let got = win::poll_until(s.g.order_ms, s.poll(), || {
        last = faults(s, &above);
        last.is_empty().then_some(())
    });
    got.ok_or_else(|| format!("after {} ms, {}", s.g.order_ms, last.join("; ")))
}

/// One read of the order check's faults now.
fn faults(s: &Snip, above: impl Fn(HWND) -> bool) -> Vec<String> {
    let order = wins::layers(s.pid, &s.titles, above);
    wins::faults(s.pid, &order, wins::screens())
}

/// A frozen screen on each screen now, read once, so an Esc the trap sends next has a snip to end.
pub fn frozen(s: &Snip) -> Result<(), String> {
    let got = faults(s, |_| false);
    got.is_empty()
        .then_some(())
        .ok_or_else(|| format!("before the Esc, {}", got.join("; ")))
}

/// Every shown window of the copy over every frozen screen: bar, hint, strip, question, keyboard and bubble alike.
pub fn ours(s: &Snip) -> Result<(), String> {
    held(s, |w| win::owner(w).0 == s.pid)
}

/// The system dialog `dialog` over every frozen screen.
pub fn dialog(s: &Snip, dialog: HWND) -> Result<(), String> {
    held(s, |w| w == dialog)
}
