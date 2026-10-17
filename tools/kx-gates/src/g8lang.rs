//! `g8 arabic`: G8 with an Arabic layout in front from before its cold start, and the owner's layout put back after,
//! even when the run breaks or panics (ADR-0028). Each switch and each read goes through a target-window G8 starts in
//! front: the keyboard follows the app in front, whose layout may lag the session's, so G8 keeps its own window there,
//! or the one a step keeps, such as the scroller's, which a fresh start would close.
#![cfg(windows)]

use kx_module_api::ports::{LayoutId, LayoutSource};
use kx_platform_windows::{WinLayouts, front, layouts};
use serde_json::{Value, json};

use crate::apps::{self, Ctx, Opened};
use crate::g8cfg::G8;
use crate::{capture, g8use, launch, win};

mod hold;

use hold::Held;
pub use hold::{Free, Holds};

/// The name that runs G8 with an Arabic layout in front.
pub const ARABIC: &str = "arabic";
/// Why nothing was switched.
const NO_LAYOUT: &str = "target-window read no layout, so G8 changed nothing";
/// What a result or an error says once the owner's layout is back.
const BACK: &str = "the owner's layout is back";
/// What it says when it is not.
const NOT_BACK: &str =
    "the owner's layout did not come back, so switch it back with the keyboard's language key";

/// How G8 brings its target-window to the front.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tap {
    /// With the Alt tap, which the mouse guard refuses once it stopped the gate.
    Plain,
    /// With the Alt tap a put-back still sends after that stop.
    PutBack,
}

/// The windows `g8 arabic` works through: target-window on Windows, a fake in tests.
pub trait Own {
    /// A started target-window.
    type Window;
    /// Starts a target-window and brings it to the front with `tap`.
    fn open(&self, tap: Tap) -> Result<Self::Window, String>;
    /// Brings `w`, already open, back to the front with the plain tap.
    fn front(&self, w: &Self::Window) -> Result<(), String>;
    /// The layout `w`'s thread types in: the session's, for a window just started.
    fn layout(&self, w: &Self::Window) -> Option<LayoutId>;
    /// Asks `w` for installed layout `id`, as the language hotkey asks, and waits until it types in it.
    fn ask(&self, w: &Self::Window, id: LayoutId) -> Result<(), String>;
    /// Closes `w`; the layout stays, as Windows 8 and later switch the whole session.
    fn close(&self, w: Self::Window);
    /// The layout of the app in front, as the keyboard reads it.
    fn in_front(&self) -> Option<LayoutId>;
}

/// Target-window on Windows; each switch waits up to `[g8] layout_wait_ms` for its layout.
pub struct WinTarget<'a> {
    ctx: &'a Ctx<'a>,
}

impl<'a> WinTarget<'a> {
    /// The target-window of `ctx`, with its waits.
    pub fn of(ctx: &'a Ctx<'a>) -> Self {
        Self { ctx }
    }
}

impl Own for WinTarget<'_> {
    type Window = Opened;

    fn open(&self, tap: Tap) -> Result<Opened, String> {
        let front: win::Front = match tap {
            Tap::Plain => win::front,
            Tap::PutBack => win::front_back,
        };
        launch::to_front_by(self.ctx, launch::start_target(self.ctx)?, front)
    }

    fn front(&self, w: &Opened) -> Result<(), String> {
        if win::front(w.hwnd, &self.ctx.cfg.timing, &self.ctx.cfg.keys) {
            Ok(())
        } else {
            Err(format!(
                "could not bring {} to the front",
                win::describe(w.hwnd)
            ))
        }
    }

    fn layout(&self, w: &Opened) -> Option<LayoutId> {
        front::thread_layout(w.hwnd).map(layouts::id)
    }

    fn ask(&self, w: &Opened, id: LayoutId) -> Result<(), String> {
        layouts::ask(w.hwnd, id).map_err(|e| format!("target-window: {e}"))?;
        let (wait_ms, poll_ms) = (self.ctx.cfg.g8.layout_wait_ms, self.ctx.cfg.timing.poll_ms);
        let types = || (self.layout(w) == Some(id)).then_some(());
        win::poll_until(wait_ms, poll_ms, types)
            .ok_or_else(|| format!("target-window did not type in it within {wait_ms} ms"))
    }

    fn close(&self, w: Opened) {
        let _ = apps::clean_up(self.ctx, w);
    }

    fn in_front(&self) -> Option<LayoutId> {
        WinLayouts.foreground()
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
struct Back<'a, O: Own> {
    own: &'a O,
    owner: LayoutId,
    done: bool,
}

impl<O: Own> Back<'_, O> {
    /// Puts it back now: the layout the session typed in until then, and whether the owner's came back.
    fn now(&mut self) -> (Option<LayoutId>, bool) {
        self.done = true;
        put_back(self.own, self.owner)
    }
}

impl<O: Own> Drop for Back<'_, O> {
    fn drop(&mut self) {
        if !self.done {
            let _ = put_back(self.own, self.owner);
        }
    }
}

/// Puts `owner` back through a fresh target-window, brought forward with the tap a put-back still sends: the layout
/// that window typed in at first, the session's, and whether `owner` came back.
fn put_back<O: Own>(own: &O, owner: LayoutId) -> (Option<LayoutId>, bool) {
    let Ok(w) = own.open(Tap::PutBack) else {
        return (None, false);
    };
    let end = own.layout(&w);
    let came = own.ask(&w, owner).is_ok();
    own.close(w);
    (end, came)
}

/// What a result or an error says about the owner's layout.
fn said(back: bool) -> &'static str {
    if back { BACK } else { NOT_BACK }
}

/// `id`, or `none`.
fn named(id: Option<LayoutId>) -> String {
    id.map_or_else(|| "none".to_string(), |l| l.to_string())
}

/// Switches the session to `to` through a fresh target-window, left in front for the cold start, after reading the
/// owner's layout there; the guard that puts it back. Nothing changes when that window reads no layout.
fn switched<O: Own>(own: &O, to: LayoutId) -> Result<Back<'_, O>, String> {
    let w = own
        .open(Tap::Plain)
        .map_err(|e| format!("{e}, so G8 changed nothing"))?;
    let Some(owner) = own.layout(&w) else {
        own.close(w);
        return Err(NO_LAYOUT.to_string());
    };
    let mut back = Back {
        own,
        owner,
        done: false,
    };
    if let Err(e) = own.ask(&w, to) {
        own.close(w);
        return Err(format!(
            "layout {to} did not show: {e}; {}",
            said(back.now().1)
        ));
    }
    // `w` stays in front for the cold start; the next target-window's start closes it.
    Ok(back)
}

/// Runs `run` with layout `to` in front and puts the owner's back after, even when it fails; `run` gets the holds to
/// call after each step that took the front. The result line gains the layout, the session's at the end, the layout
/// each hold found in front and whether the owner's came back; it passes only when the reading after use and the
/// session at the end were both in `to` and the owner's came back.
pub fn within<O: Own>(
    own: &O,
    to: LayoutId,
    run: impl FnOnce(&dyn Holds<O::Window>) -> Result<Value, String>,
) -> Result<Value, String> {
    let mut back = switched(own, to)?;
    let holds = Held::new(own, to);
    let got = run(&holds);
    let (end, came) = back.now();
    let mut v = got.map_err(|e| format!("{e}; {}", said(came)))?;
    let held = end == Some(to) && g8use::layout_in(&v) == Some(to.to_string());
    v["layout"] = json!(to.to_string());
    v["layout_end"] = json!(end.map(|l| l.to_string()));
    v["layout_behind"] = json!(holds.behind());
    v["layout_held"] = json!(held);
    v["layout_back"] = json!(came);
    if !(held && came) {
        v["pass"] = json!(false);
    }
    Ok(v)
}

#[cfg(test)]
mod tests;
