//! `g8 arabic`: G8 with an Arabic layout in front from before its cold start, asked for as the language hotkey asks,
//! and the owner's layout put back after, even when the run breaks or panics (ADR-0028).
#![cfg(windows)]

use kx_module_api::ports::{LayoutId, LayoutSource};
use kx_platform_windows::window::foreground;
use kx_platform_windows::{WinLayouts, layouts};
use serde_json::{Value, json};

use crate::apps::Ctx;
use crate::g8cfg::G8;
use crate::{capture, g8use, win};

/// The name that runs G8 with an Arabic layout in front.
pub const ARABIC: &str = "arabic";
/// Why nothing was switched.
const NO_FRONT: &str = "no app is in front to ask for a layout, so G8 changed nothing";
/// What a result or an error says once the owner's layout is back.
const BACK: &str = "the owner's layout is back";
/// What it says when it is not.
const NOT_BACK: &str =
    "the owner's layout did not come back, so switch it back with the keyboard's language key";

/// The layout of the app in front and its switch: Windows in G8, a fake in tests.
pub trait Front {
    /// The layout of the app in front; none with no app in front.
    fn layout(&self) -> Option<LayoutId>;
    /// Asks the app in front for installed layout `id`, as the language hotkey asks; true once it shows there.
    fn switch(&self, id: LayoutId) -> bool;
}

/// The app in front on Windows; a switch waits up to `[g8] layout_wait_ms` for its layout to show.
pub struct WinFront {
    wait_ms: u64,
    poll_ms: u64,
}

impl WinFront {
    /// The front with `ctx`'s waits.
    pub fn of(ctx: &Ctx) -> Self {
        Self {
            wait_ms: ctx.cfg.g8.layout_wait_ms,
            poll_ms: ctx.cfg.timing.poll_ms,
        }
    }
}

impl Front for WinFront {
    fn layout(&self) -> Option<LayoutId> {
        WinLayouts.foreground()
    }

    fn switch(&self, id: LayoutId) -> bool {
        let shows = || (self.layout() == Some(id)).then_some(());
        layouts::ask(foreground(), id).is_ok()
            && win::poll_until(self.wait_ms, self.poll_ms, shows).is_some()
    }
}

/// Refuses `g8 arabic` settings that name no language or never wait for a switch.
pub fn check(g: &G8) -> Result<(), String> {
    if g.arabic.is_empty() || g.layout_wait_ms == 0 {
        return Err("g8.arabic must name a language and g8.layout_wait_ms be above 0".to_string());
    }
    Ok(())
}

/// The layout run `name` puts in front: for `arabic`, the first of `source`'s installed layouts in `[g8] arabic`'s
/// language; none for another name. An error before anything changed when none is installed.
pub fn wanted(source: &dyn LayoutSource, g: &G8, name: &str) -> Result<Option<LayoutId>, String> {
    if name != ARABIC {
        return Ok(None);
    }
    let found = capture::find(source, &g.arabic);
    found.map(Some).ok_or_else(|| {
        format!(
            "no {} layout is installed, so g8 {ARABIC} stopped and changed nothing",
            g.arabic
        )
    })
}

/// Puts the owner's layout back once, at the latest when dropped, so a run that breaks or panics still puts it back.
struct Back<'a> {
    front: &'a dyn Front,
    owner: LayoutId,
    done: bool,
}

impl Back<'_> {
    /// Puts it back now; true once it shows.
    fn now(&mut self) -> bool {
        self.done = true;
        self.front.switch(self.owner)
    }
}

impl Drop for Back<'_> {
    fn drop(&mut self) {
        if !self.done {
            self.front.switch(self.owner);
        }
    }
}

/// What a result or an error says about the owner's layout.
fn said(back: bool) -> &'static str {
    if back { BACK } else { NOT_BACK }
}

/// Runs `run` with layout `to` in front and puts the owner's back after, even when it fails; with no app in front
/// nothing changes. The result line gains the layout, whether it was still in front at the reading after use and
/// whether the owner's came back, and passes only when both held.
pub fn within(
    front: &dyn Front,
    to: LayoutId,
    run: impl FnOnce() -> Result<Value, String>,
) -> Result<Value, String> {
    let owner = front.layout().ok_or(NO_FRONT)?;
    let mut back = Back {
        front,
        owner,
        done: false,
    };
    if !front.switch(to) {
        return Err(format!(
            "layout {to} did not show in front; {}",
            said(back.now())
        ));
    }
    let got = run();
    let came = back.now();
    let mut v = got.map_err(|e| format!("{e}; {}", said(came)))?;
    let held = g8use::layout_in(&v) == Some(to.to_string());
    v["layout"] = json!(to.to_string());
    v["layout_held"] = json!(held);
    v["layout_back"] = json!(came);
    if !(held && came) {
        v["pass"] = json!(false);
    }
    Ok(v)
}

#[cfg(test)]
mod tests;
