//! G24's trap part (ADR-0059), alone as `g24 trap`: with the keyboard as a bubble, every window of ours lies over the
//! frozen screens at each step, and our keyboard's Esc, another program's Esc, Esc twice and the Save dialog's own
//! Cancel each lead out. It stops at the first step that fails, so the copy is ended at once.
#![cfg(windows)]

use std::path::Path;

use kx_mod_keyboard::Nav;
use kx_module_api::ports::SnipKeyboard;
use serde_json::{Value, json};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::KeyboardAndMouse::{INPUT, VK_ESCAPE};

use super::judge::{self, Kind};
use super::low::{self, close_strip};
use super::order;
use super::scene::Scene;
use super::shots::corners;
use super::snip::Snip;
use super::wins;
use crate::g25::part;
use crate::ownkeys::{self, Real};
use crate::product::Keyboard;
use crate::win::{self, sleep_ms};
use crate::{board, keys};

/// The steps by name, in order: our keyboard's Esc, another program's Esc, Esc twice at the question, and Save.
pub const NAMES: [&str; 4] = ["keys", "esc", "twice", "save"];
/// One step on the copy, given its scene and its workers' folder.
type Step = fn(&Snip, &Scene, &Path) -> Result<Value, String>;
/// The steps `NAMES` names.
const STEPS: [Step; 4] = [keys, esc, twice, save];
/// Another program's mark on its keys: none, as an assistive program such as Voice Access sends them.
const UNMARKED: usize = 0;
/// What shows once the Snip key froze the screen: a frozen screen, the bar and the keyboard's bubble.
const PICKING: [Kind; 3] = [Kind::Frozen, Kind::Capture, Kind::Bubble];
/// Why a step did not run.
const NOT_RUN: &str = "not run: an earlier step failed";

/// True for the keyboard way whose bubble the trap taps to bring the keyboard over a capture; hidden has none.
pub fn taps(way: SnipKeyboard) -> bool {
    way == SnipKeyboard::Bubble
}

/// Each step in turn, by name; once one fails the rest do not run.
pub fn run(s: &Snip, sc: &Scene, workers: &Path) -> Value {
    let mut v = json!({});
    let mut going = true;
    for (name, step) in NAMES.into_iter().zip(STEPS) {
        let got = if going {
            part(step(s, sc, workers))
        } else {
            part(Err(NOT_RUN.to_string()))
        };
        going = got["ok"] == true;
        v[name] = got;
    }
    v
}

/// A step that passed.
fn done() -> Value {
    json!({ "ok": true })
}

/// Waits up to `[g24] wait_ms` until a window of each kind in `need` shows; else the kinds that never did.
fn showing(s: &Snip, need: &[Kind]) -> Result<(), String> {
    let mut lacking = need.to_vec();
    let got = s.until(|| {
        let kinds: Vec<Kind> = wins::ours(s.pid, &s.titles)
            .into_iter()
            .map(|(_, k)| k)
            .collect();
        lacking = judge::lacks(need, &kinds);
        lacking.is_empty().then_some(())
    });
    got.ok_or_else(|| format!("no window of {lacking:?} showed within {} ms", s.g.wait_ms))
}

/// The Snip key, then once the screen froze, every window of ours over the frozen screens.
fn snip_on(s: &Snip) -> Result<(), String> {
    s.press()?;
    showing(s, &PICKING)?;
    order::ours(s)
}

/// The bubble, then once the keyboard is back, every window of ours over the frozen screens.
fn bubble_tap(s: &Snip) -> Result<(), String> {
    s.click(&s.ctx.cfg.g9.bring_back)?;
    showing(s, &[Kind::Keyboard])?;
    order::ours(s)
}

/// Waits for the strip's Copy on top, after naming any window of ours under a frozen screen.
fn strip(s: &Snip) -> Result<(), String> {
    let shown = s.until(|| s.has(&s.g.ids.copy).then_some(()));
    order::ours(s)?;
    shown.ok_or_else(|| format!("the strip did not show on top within {} ms", s.g.wait_ms))
}

/// Region on the bar and its two corners on the frozen screen, then the strip over them.
fn region(s: &Snip, sc: &Scene) -> Result<(), String> {
    s.click(&s.g.ids.region)?;
    corners(&sc.region)
        .iter()
        .try_for_each(|&p| s.on_frozen(p))?;
    strip(s)
}

/// Esc's press and release, unmarked.
fn esc_keys() -> [INPUT; 2] {
    [false, true].map(|up| keys::tagged(VK_ESCAPE.0, up, UNMARKED))
}

/// Sends Esc's press and release with `send`; after an error it sends the release alone, which goes out even once the
/// mouse guard stopped the gate, as Windows may have taken the press but not the release.
fn press_esc(mut send: impl FnMut(&[INPUT]) -> Result<(), String>) -> Result<(), String> {
    let [down, up] = esc_keys();
    send(&[down, up]).map_err(|e| match send(&[up]) {
        Ok(()) => e,
        Err(again) => format!("{e}; letting Esc go failed too: {again}"),
    })
}

/// One Esc as another program sends it, after `[g24] settle_ms`, only while target-window is in front and the screens
/// are still frozen.
fn other_esc(s: &Snip) -> Result<(), String> {
    sleep_ms(s.g.settle_ms);
    ownkeys::ours(&Real, s.app)?;
    order::frozen(s)?;
    press_esc(keys::send)
}

/// Our keyboard: the Snip key, the bubble, then the keyboard's Esc, clicked while the screens are still frozen, ends
/// the snip.
fn keys(s: &Snip, _: &Scene, _: &Path) -> Result<Value, String> {
    snip_on(s)?;
    bubble_tap(s)?;
    let esc = board::nav_key(Nav::Esc)?;
    let click = || {
        order::frozen(s)?;
        Keyboard::find()?.click(&esc)
    };
    ownkeys::click(&Real, s.app, &click)?;
    sleep_ms(s.g.settle_ms);
    s.ended()?;
    Ok(done())
}

/// Another program's Esc ends a pick.
fn esc(s: &Snip, _: &Scene, _: &Path) -> Result<Value, String> {
    snip_on(s)?;
    other_esc(s)?;
    s.ended()?;
    Ok(done())
}

/// A region's strip: another program's Esc asks first, over the frozen screens, and a second ends the snip unkept.
fn twice(s: &Snip, sc: &Scene, _: &Path) -> Result<Value, String> {
    snip_on(s)?;
    region(s, sc)?;
    other_esc(s)?;
    s.wait_for(&s.titles.ask, true, "close question")?;
    order::ours(s)?;
    other_esc(s)?;
    s.ended()?;
    Ok(json!({ "asked": true, "ok": true }))
}

/// The dialog `shown` over the frozen screens, then its own Cancel by `[g24] cancel_id`; on a failure it is closed, so
/// its worker ends.
fn cancel(s: &Snip, shown: HWND, workers: &Path) -> Result<(), String> {
    let gone = || low::dialog(s, workers).is_none().then_some(());
    let got = order::dialog(s, shown)
        .and_then(|()| s.press_in(shown, &s.g.cancel_id))
        .and_then(|()| {
            s.until(gone)
                .ok_or_else(|| "Cancel left the Save dialog".to_string())
        });
    if got.is_err() {
        let _ = win::close(shown);
    }
    got
}

/// Save with the keyboard back: the dialog lies over the frozen screens, its Cancel closes it, ours lie over them again,
/// and the strip's Close ends the snip as the full run does.
fn save(s: &Snip, sc: &Scene, workers: &Path) -> Result<Value, String> {
    snip_on(s)?;
    bubble_tap(s)?;
    region(s, sc)?;
    s.click(&s.g.ids.save)?;
    let shown = s
        .until(|| low::dialog(s, workers))
        .ok_or("the Save dialog did not show")?;
    cancel(s, shown, workers)?;
    showing(s, &[Kind::Keyboard])?;
    strip(s)?;
    close_strip(s)?;
    Ok(done())
}

#[cfg(test)]
mod tests;
