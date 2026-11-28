//! One copy's snips driven by real clicks: the keyboard's Snip key, Snip's buttons by gate id and the frozen screen,
//! each click only where that window of the copy is the one on top, and the waits for Snip's windows.
#![cfg(windows)]

use std::cell::RefCell;
use std::time::Instant;

use kx_mod_keyboard::Role;
use kx_module_api::ports::SnipKeyboard;
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::UI::WindowsAndMessaging::WS_EX_TRANSPARENT;

use super::clip;
use super::clock::Clock;
use super::judge::Kind;
use super::pic::Pic;
use super::wins::{self, Titles};
use crate::apps::{Ctx, Opened};
use crate::g9win::ex_of;
use crate::g23::live;
use crate::keymap::Reader;
use crate::ownkeys::{self, Real};
use crate::p12cfg::G24;
use crate::product::{self, Keyboard};
use crate::win::{self, sleep_ms};
use crate::{board, mouse, mouseguard, winclip};

/// One picture Copy: its span in clipboard history time, from just before the click to just after the read back, and
/// the picture's bytes when it was read back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Copied {
    pub span: (i64, i64),
    pub bytes: Option<usize>,
}

/// The run's picture Copies, in order.
pub type PictureCopies = RefCell<Vec<Copied>>;

/// A copy of the test build, its keyboard `way`, and target-window in front.
pub struct Snip<'a> {
    pub ctx: &'a Ctx<'a>,
    pub g: &'a G24,
    pub pid: u32,
    pub app: &'a Opened,
    pub way: SnipKeyboard,
    pub titles: Titles,
    /// The record bar's clock in every shipped language.
    pub clock: Clock,
    copied: &'a PictureCopies,
    reader: Reader,
    key: String,
}

impl<'a> Snip<'a> {
    /// The copy `pid` with its keyboard `way`, driven on `app`; each picture Copy's span goes into `copied`.
    pub fn new(
        ctx: &'a Ctx<'a>,
        (pid, way): (u32, SnipKeyboard),
        app: &'a Opened,
        copied: &'a PictureCopies,
    ) -> Result<Self, String> {
        let g = &ctx.cfg.g24;
        Ok(Self {
            ctx,
            g,
            pid,
            app,
            way,
            titles: Titles::new(&g.titles)?,
            clock: Clock::new(&g.clock)?,
            copied,
            reader: Reader::new()?,
            key: board::id_of(Role::Snip)?,
        })
    }

    /// Clicks Copy button `id` and reads its picture back, noting the Copy once a click was tried, so that picture
    /// leaves Windows' history.
    pub fn copy_picture(&self, id: &str) -> Result<Pic, String> {
        let at = self.find(id)?;
        let (from, since) = (winclip::now(), clip::mark());
        let read = self
            .click_at(at)
            .and_then(|()| clip::picture(self.ctx, since));
        let bytes = read.as_ref().ok().map(|(_, b)| *b);
        let span = (from, winclip::now());
        self.copied.borrow_mut().push(Copied { span, bytes });
        read.map(|(pic, _)| pic)
    }

    /// Presses button `id` in window `w`, such as a system dialog's, as a screen reader does, unless the mouse guard
    /// stopped the gate; an element with that id that is no button is passed over.
    pub fn press_in(&self, w: HWND, id: &str) -> Result<(), String> {
        self.reader.press_button(w, id)
    }

    /// The names a screen reader hears in window `w`; none when it could not be read.
    pub fn names(&self, w: HWND) -> Vec<String> {
        live::names(&self.reader, w).unwrap_or_default()
    }

    /// The gate's poll step, in ms.
    pub fn poll(&self) -> u64 {
        self.ctx.cfg.timing.poll_ms
    }

    /// Waits up to `[g24] wait_ms` for `f` to give a value.
    pub fn until<T>(&self, f: impl FnMut() -> Option<T>) -> Option<T> {
        win::poll_until(self.g.wait_ms, self.poll(), f)
    }

    /// Waits for a window titled one of `which` to show, or with `show` false to go; an error naming `what` after
    /// `[g24] wait_ms`.
    pub fn wait_for(&self, which: &[String], show: bool, what: &str) -> Result<(), String> {
        let now = || (wins::shown(self.pid, which).is_some() == show).then_some(());
        let verb = if show { "show" } else { "go" };
        self.until(now)
            .ok_or_else(|| format!("the {what} did not {verb} within {} ms", self.g.wait_ms))
    }

    /// The centre of button `id` in a shown window of the copy that takes clicks, while that window is on top there.
    fn spot(&self, id: &str) -> Result<Option<POINT>, String> {
        for w in product::shown_by(self.pid) {
            if ex_of(w) & WS_EX_TRANSPARENT.0 != 0 {
                continue;
            }
            let found = live::kept(live::boxes(&self.reader, w, id), || live::shown(w))?;
            let at = found.unwrap_or_default().first().map(wins::centre);
            if let Some(at) = at.filter(|&p| win::root_at(p) == w) {
                return Ok(Some(at));
            }
        }
        Ok(None)
    }

    /// Clicks button `id` once it shows on top, while target-window is in front, then pauses `[g24] settle_ms`.
    pub fn click(&self, id: &str) -> Result<(), String> {
        let at = self.find(id)?;
        ownkeys::click(&Real, self.app, &|| mouse::click_at(at))?;
        sleep_ms(self.g.settle_ms);
        Ok(())
    }

    /// Like `click`, whatever is in front, for a step after a system dialog took the front.
    pub fn click_any(&self, id: &str) -> Result<(), String> {
        let at = self.find(id)?;
        mouseguard::allowed()?;
        mouse::click_at(at)?;
        sleep_ms(self.g.settle_ms);
        Ok(())
    }

    /// Where button `id` shows on top, once it does within `[g24] wait_ms`.
    pub fn find(&self, id: &str) -> Result<POINT, String> {
        let wait = self.g.wait_ms;
        live::poll_read(wait, self.poll(), || self.spot(id))?
            .ok_or_else(|| format!("{id} did not show on top within {wait} ms"))
    }

    /// Clicks spot `p`, kept from `find`, while a window of the copy is still on top there and target-window in front.
    pub fn click_at(&self, p: POINT) -> Result<(), String> {
        if !product::shown_by(self.pid).contains(&win::root_at(p)) {
            return Err(format!(
                "{} covers the kept spot",
                win::describe(win::root_at(p))
            ));
        }
        ownkeys::click(&Real, self.app, &|| mouse::click_at(p))?;
        sleep_ms(self.g.settle_ms);
        Ok(())
    }

    /// True while button `id` shows on top.
    pub fn has(&self, id: &str) -> bool {
        self.spot(id).is_ok_and(|p| p.is_some())
    }

    /// Clicks the keyboard's Snip key once the keyboard shows, while target-window is in front: when it was clicked.
    pub fn press(&self) -> Result<Instant, String> {
        self.wait_for(&self.titles.keyboard, true, "keyboard")?;
        ownkeys::click(&Real, self.app, &|| Keyboard::find()?.click(&self.key))?;
        Ok(Instant::now())
    }

    /// The frozen screen, while it shows.
    pub fn frozen(&self) -> Option<HWND> {
        wins::shown(self.pid, &self.titles.frozen)
    }

    /// Clicks the frozen screen at `p`, refused unless it is the window on top there.
    pub fn on_frozen(&self, p: POINT) -> Result<(), String> {
        let frozen = self.frozen().ok_or("the frozen screen is not showing")?;
        if win::root_at(p) != frozen {
            return Err(format!(
                "{} covers the frozen screen where the gate clicks",
                win::describe(win::root_at(p))
            ));
        }
        ownkeys::click(&Real, self.app, &|| mouse::click_at(p))?;
        sleep_ms(self.g.settle_ms);
        Ok(())
    }

    /// True while no window of a snip shows: no frozen screen, capture window, editor or question.
    fn quiet(&self) -> bool {
        let busy = |(w, kind): &(HWND, Kind)| {
            let t = win::title(*w);
            matches!(kind, Kind::Frozen | Kind::Capture)
                || self.titles.editor.contains(&t)
                || self.titles.ask.contains(&t)
        };
        !wins::ours(self.pid, &self.titles).iter().any(busy)
    }

    /// Waits for the snip to end and the keyboard to show again.
    pub fn ended(&self) -> Result<(), String> {
        self.until(|| self.quiet().then_some(()))
            .ok_or_else(|| format!("the snip did not end within {} ms", self.g.wait_ms))?;
        self.wait_for(&self.titles.keyboard, true, "keyboard")
    }

    /// Closes what a failed part left open in up to `[g24] reset_passes` passes, best effort: Stop, the question's Don't
    /// save, the editor's, strip's and bar's Close, each only while it shows.
    pub fn reset(&self) {
        let ids = &self.g.ids;
        let order = [
            &ids.stop,
            &ids.drop,
            &ids.edit_close,
            &ids.close,
            &ids.bar_close,
        ];
        for _ in 0..self.g.reset_passes {
            if self.quiet() {
                return;
            }
            if let Some(id) = order.into_iter().find(|id| self.has(id)) {
                let _ = self.click_any(id);
            }
            win::poll_until(self.g.wait_ms, self.poll(), || self.quiet().then_some(()));
        }
    }
}
