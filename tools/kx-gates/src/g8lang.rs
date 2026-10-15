//! `g8 arabic`: G8 with an Arabic layout in front from before its cold start, and the owner's layout put back after,
//! even when the run breaks or panics (ADR-0028). Each switch goes through target-window, which G8 starts in front and
//! asks as the language hotkey asks: the app that happens to be in front may not take the request.
#![cfg(windows)]

use kx_module_api::ports::{LayoutId, LayoutSource};
use kx_platform_windows::{WinLayouts, front, layouts};
use serde_json::{Value, json};

use crate::apps::{self, Ctx, Opened};
use crate::g8cfg::G8;
use crate::{capture, g8use, launch, win};

/// The name that runs G8 with an Arabic layout in front.
pub const ARABIC: &str = "arabic";
/// Why nothing was switched.
const NO_FRONT: &str = "no app is in front to read the owner's layout from, so G8 changed nothing";
/// What a result or an error says once the owner's layout is back.
const BACK: &str = "the owner's layout is back";
/// What it says when it is not.
const NOT_BACK: &str =
    "the owner's layout did not come back, so switch it back with the keyboard's language key";

/// The layout of the app in front and the session's switch: Windows in G8, a fake in tests.
pub trait Front {
    /// The layout of the app in front; none with no app in front.
    fn layout(&self) -> Option<LayoutId>;
    /// Switches the session to installed layout `id`, or says why not.
    fn switch(&self, id: LayoutId) -> Result<(), String>;
}

/// A window G8 starts in front and asks for a layout: target-window on Windows, a fake in tests.
pub trait Own {
    /// The started window.
    type Window;
    /// Starts it and brings it to the front.
    fn open(&self) -> Result<Self::Window, String>;
    /// Asks `w` for installed layout `id`, as the language hotkey asks, and waits until it types in it.
    fn ask(&self, w: &Self::Window, id: LayoutId) -> Result<(), String>;
    /// Closes `w`; the layout stays, as Windows 8 and later switch the whole session.
    fn close(&self, w: Self::Window);
}

/// Switches the session to `id` through a window G8 started in front, never the app that was in front; the window
/// closes either way.
pub fn through<O: Own>(own: &O, id: LayoutId) -> Result<(), String> {
    let w = own.open()?;
    let asked = own.ask(&w, id);
    own.close(w);
    asked
}

/// The session's switch on Windows, through target-window, which waits up to `[g8] layout_wait_ms` for each layout.
pub struct WinFront<'a> {
    ctx: &'a Ctx<'a>,
}

impl<'a> WinFront<'a> {
    /// The switch with `ctx`'s target-window and waits.
    pub fn of(ctx: &'a Ctx<'a>) -> Self {
        Self { ctx }
    }
}

impl Own for WinFront<'_> {
    type Window = Opened;

    fn open(&self) -> Result<Opened, String> {
        launch::to_front(self.ctx, launch::start_target(self.ctx)?)
    }

    fn ask(&self, w: &Opened, id: LayoutId) -> Result<(), String> {
        layouts::ask(w.hwnd, id).map_err(|e| format!("target-window: {e}"))?;
        let (wait_ms, poll_ms) = (self.ctx.cfg.g8.layout_wait_ms, self.ctx.cfg.timing.poll_ms);
        let types = || (front::thread_layout(w.hwnd).map(layouts::id) == Some(id)).then_some(());
        win::poll_until(wait_ms, poll_ms, types)
            .ok_or_else(|| format!("target-window did not type in it within {wait_ms} ms"))
    }

    fn close(&self, w: Opened) {
        let _ = apps::clean_up(self.ctx, w);
    }
}

impl Front for WinFront<'_> {
    fn layout(&self) -> Option<LayoutId> {
        WinLayouts.foreground()
    }

    fn switch(&self, id: LayoutId) -> Result<(), String> {
        through(self, id)
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
        self.front.switch(self.owner).is_ok()
    }
}

impl Drop for Back<'_> {
    fn drop(&mut self) {
        if !self.done {
            let _ = self.front.switch(self.owner);
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
    if let Err(e) = front.switch(to) {
        return Err(format!(
            "layout {to} did not show: {e}; {}",
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
