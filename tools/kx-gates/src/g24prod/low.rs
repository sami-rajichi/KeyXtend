//! Snip's windows under a system dialog: the strip's Save opens the Save dialog over the frozen screens, Snip's capture
//! windows leave the top while it shows and come back over them once the gate cancels it, each kept out of captures.
#![cfg(windows)]

use std::path::Path;

use serde_json::{Value, json};
use windows::Win32::Foundation::HWND;

use super::judge::{Kind, settled};
use super::scene::Scene;
use super::shots::{corners, covered};
use super::snip::Snip;
use super::{order, wins};
use crate::{win, workertok};

/// The shown Save dialog of the copy or of a worker in folder `workers`.
pub fn dialog(s: &Snip, workers: &Path) -> Option<HWND> {
    let found = workertok::processes();
    let mut pids = workertok::beside(&found, workers);
    pids.push(s.pid);
    let ours = |w: &HWND| pids.contains(&win::owner(*w).0) && win::class(*w) == s.g.dialog_class;
    win::seen_windows().into_iter().find(ours)
}

/// The copy's capture windows that show now.
fn capture(s: &Snip) -> Vec<HWND> {
    let all = wins::ours(s.pid, &s.titles);
    all.into_iter()
        .filter(|(_, k)| *k == Kind::Capture)
        .map(|(w, _)| w)
        .collect()
}

/// What `f` says of each capture window shown now.
fn each(s: &Snip, f: fn(HWND) -> bool) -> Vec<bool> {
    capture(s).into_iter().map(f).collect()
}

/// Waits until every capture window is on top, or with `top` false none is: whether it came, and whether each
/// capture window was kept out of captures then.
fn settles(s: &Snip, top: bool) -> (bool, bool) {
    let came = s.until(|| settled(&each(s, wins::topmost), top).then_some(()));
    (came.is_some(), settled(&each(s, wins::shielded), true))
}

/// Closes the strip with its Close, answering Don't save when the question shows, whatever is in front, then waits for
/// the snip to end.
pub fn close_strip(s: &Snip) -> Result<(), String> {
    let ids = &s.g.ids;
    s.click_any(&ids.close)?;
    if s.until(|| s.has(&ids.drop).then_some(())).is_some() {
        s.click_any(&ids.drop)?;
    }
    s.ended()
}

/// The order check's verdict, with its fault when it failed.
fn over(got: Result<(), String>) -> Value {
    json!({ "ok": got.is_ok(), "under": got.err() })
}

/// A region kept by Save: the dialog shows over the frozen screens, then the gate cancels it and ours lie over them
/// again; the strip's Close and Don't save end the snip.
pub fn low(s: &Snip, sc: &Scene, workers: &Path) -> Result<Value, String> {
    let ids = &s.g.ids;
    covered(s, &sc.frame)?;
    s.click(&ids.region)?;
    corners(&sc.region)
        .iter()
        .try_for_each(|&p| s.on_frozen(p))?;
    s.click(&ids.save)?;
    let shown = s
        .until(|| dialog(s, workers))
        .ok_or("the Save dialog did not show")?;
    let ((left_top, shield_low), dialog_over) = (settles(s, false), over(order::dialog(s, shown)));
    win::close(shown)?;
    s.until(|| dialog(s, workers).is_none().then_some(()))
        .ok_or("the Save dialog did not close")?;
    let ((back_top, shield_back), back_over) = (settles(s, true), over(order::ours(s)));
    close_strip(s)?;
    let above = dialog_over["ok"] == true && back_over["ok"] == true;
    let ok = left_top && shield_low && back_top && shield_back && above;
    Ok(json!({
        "left_top": left_top, "shielded_low": shield_low, "back_on_top": back_top, "shielded_back": shield_back,
        "dialog_over": dialog_over, "back_over": back_over, "ok": ok,
    }))
}
