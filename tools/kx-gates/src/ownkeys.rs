//! The one check before any key or pasting click reaches the gate's own app (the owner's Notepad rule): it goes out
//! only while that window is in front with the gate's own tab shown, through a desk that tests replace with a fake.
#![cfg(windows)]

use kx_platform_windows::window::foreground;
use windows::Win32::Foundation::HWND;

use crate::apps::Opened;
use crate::keys;
use crate::win::{self, sleep_ms};

/// Why keys are held back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Held {
    /// Another window is in front.
    Front,
    /// Our window is in front, but another tab shows.
    Tab,
}

/// True when window title `title` shows our tab: it holds `has`, our file or page name, when the app has one.
pub(crate) fn tab_ours(title: &str, has: Option<&str>) -> bool {
    has.is_none_or(|h| title.contains(h))
}

/// Whether keys may go to window `app`: it is the window in `front`, and its `title` shows our tab named `has`.
pub fn may_send(front: HWND, app: HWND, title: &str, has: Option<&str>) -> Result<(), Held> {
    if front != app {
        return Err(Held::Front);
    }
    if !tab_ours(title, has) {
        return Err(Held::Tab);
    }
    Ok(())
}

/// The desktop as the gate's keys meet it: the window in front, a window's title, and the keys, clicks and pauses.
pub trait Desk {
    /// The window in front.
    fn front(&self) -> HWND;
    /// Window `w`'s title.
    fn title(&self, w: HWND) -> String;
    /// Window `w` named for an error, never by its title.
    fn describe(&self, w: HWND) -> String;
    /// Sends virtual keys `vks`, pressed in order.
    fn combo(&self, vks: &[u16]) -> Result<(), String>;
    /// Runs `act`, a click whose result lands in the app.
    fn click(&self, act: &dyn Fn() -> Result<(), String>) -> Result<(), String>;
    /// Waits `ms` for the app to take what was sent.
    fn settle(&self, ms: u64);
}

/// The real desktop.
pub struct Real;

impl Desk for Real {
    fn front(&self) -> HWND {
        foreground()
    }

    fn title(&self, w: HWND) -> String {
        win::title(w)
    }

    fn describe(&self, w: HWND) -> String {
        win::describe(w)
    }

    fn combo(&self, vks: &[u16]) -> Result<(), String> {
        keys::combo(vks)
    }

    fn click(&self, act: &dyn Fn() -> Result<(), String>) -> Result<(), String> {
        act()
    }

    fn settle(&self, ms: u64) {
        sleep_ms(ms);
    }
}

/// Fails unless `may_send` lets keys go to `app` on `desk`, naming what holds them back.
pub fn ours(desk: &dyn Desk, app: &Opened) -> Result<(), String> {
    let front = desk.front();
    let title = desk.title(app.hwnd);
    may_send(front, app.hwnd, &title, app.title_has.as_deref()).map_err(|held| match held {
        Held::Front => format!("keys held back: {} is in front", desk.describe(front)),
        Held::Tab => format!("our tab is not shown: {}", desk.describe(app.hwnd)),
    })
}

/// Sends `vks` to `app` on `desk` once `ours` allows it; else no key goes out.
pub fn send(desk: &dyn Desk, app: &Opened, vks: &[u16]) -> Result<(), String> {
    ours(desk, app)?;
    desk.combo(vks)
}

/// Sends each of `combos` to `app` on `desk` in turn, checking `ours` before each, so a tab that changes stops the rest.
pub fn each(desk: &dyn Desk, app: &Opened, combos: &[&[u16]]) -> Result<(), String> {
    combos.iter().try_for_each(|vks| send(desk, app, vks))
}

/// Runs `act`, a click that pastes into `app`, on `desk` once `ours` allows it; else nothing is clicked.
pub fn click(
    desk: &dyn Desk,
    app: &Opened,
    act: &dyn Fn() -> Result<(), String>,
) -> Result<(), String> {
    ours(desk, app)?;
    desk.click(act)
}

#[cfg(test)]
pub(crate) mod fake;

#[cfg(test)]
mod tests;
