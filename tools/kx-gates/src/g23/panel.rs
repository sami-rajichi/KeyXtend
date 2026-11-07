//! The product's Quick-fill panel as a screen reader meets it, by the gate ids `[g23]` names: its window, its rows'
//! names, and a real click on a part once it shows on top; and the keyboard's `fill` key that opens and shuts it. A
//! failed read is an error, never a part that is not there.
#![cfg(windows)]

use kx_mod_keyboard::Role;
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::UI::Accessibility::UIA_ListItemControlTypeId;

use crate::apps::{Ctx, Opened};
use crate::clippanel::{has_area, placed};
use crate::g23::live::{self, kept, poll_read};
use crate::keymap::{Reader, centre};
use crate::ownkeys::{self, Real};
use crate::p9cfg::G23;
use crate::product::{self, Keyboard};
use crate::win::{self, sleep_ms};
use crate::{board, mouse};

/// A reader of the product's Quick-fill panel.
pub struct Panel<'a> {
    ctx: &'a Ctx<'a>,
    reader: Reader,
}

impl<'a> Panel<'a> {
    /// A reader for the panel of the product that runs now.
    pub fn new(ctx: &'a Ctx<'a>) -> Result<Self, String> {
        Ok(Self {
            ctx,
            reader: Reader::new()?,
        })
    }

    /// The G23 settings.
    fn g(&self) -> &G23 {
        &self.ctx.cfg.g23
    }

    /// The UI Automation client it reads with.
    pub fn reader(&self) -> &Reader {
        &self.reader
    }

    /// The first box of part `id` with an area in window `w`; none when it has none or `w` went meanwhile.
    fn area(&self, w: HWND, id: &str) -> Result<Option<RECT>, String> {
        let boxes = kept(live::boxes(&self.reader, w, id), || live::shown(w))?;
        Ok(boxes.and_then(|b| b.into_iter().find(has_area)))
    }

    /// The shown product window whose header close has an area on screen; none while the panel is closed.
    pub fn window(&self) -> Result<Option<HWND>, String> {
        let close = self.g().id(&self.g().close);
        for w in product::shown() {
            if self.area(w, &close)?.is_some() {
                return Ok(Some(w));
            }
        }
        Ok(None)
    }

    /// Waits up to `[g23] panel_wait_ms` for the panel to be `open`, or closed: false when the wait ran out.
    pub fn wait(&self, open: bool) -> Result<bool, String> {
        let now = || Ok((self.window()?.is_some() == open).then_some(()));
        Ok(poll_read(self.g().panel_wait_ms, self.ctx.cfg.timing.poll_ms, now)?.is_some())
    }

    /// Like `wait`, with an error when the wait ran out.
    pub fn shows(&self, open: bool) -> Result<(), String> {
        let what = if open { "open" } else { "close" };
        let wait = self.g().panel_wait_ms;
        self.wait(open)?
            .then_some(())
            .ok_or_else(|| format!("the Quick-fill panel did not {what} within {wait} ms"))
    }

    /// The centre of part `id` while it shows on the open panel with nothing over it.
    fn on_top(&self, id: &str) -> Result<Option<POINT>, String> {
        let Some(hwnd) = self.window()? else {
            return Ok(None);
        };
        let at = self.area(hwnd, id)?.map(|r| centre(&r));
        Ok(at.filter(|&p| win::root_at(p) == hwnd))
    }

    /// True while part `id` has an area on the open panel, covered or not.
    fn has(&self, id: &str) -> Result<bool, String> {
        match self.window()? {
            Some(w) => Ok(self.area(w, id)?.is_some()),
            None => Ok(false),
        }
    }

    /// True once part `id` has gone from the panel within `[g23] panel_wait_ms`, as a page it was on closed.
    pub fn goes(&self, id: &str) -> Result<bool, String> {
        let gone = || Ok((!self.has(id)?).then_some(()));
        Ok(poll_read(self.g().panel_wait_ms, self.ctx.cfg.timing.poll_ms, gone)?.is_some())
    }

    /// Clicks part `id` with the real pointer once it shows on top, then pauses `[g23] settle_ms`.
    pub fn click(&self, id: &str) -> Result<(), String> {
        let (wait, poll) = (self.g().panel_wait_ms, self.ctx.cfg.timing.poll_ms);
        let at = poll_read(wait, poll, || self.on_top(id))?.ok_or_else(|| {
            format!("{id} did not show on top of the Quick-fill panel within {wait} ms")
        })?;
        mouse::click_at(at)?;
        sleep_ms(self.g().settle_ms);
        Ok(())
    }

    /// Shuts the panel with its header's close, a real click: true once it went within `[g23] panel_wait_ms`.
    pub fn close(&self) -> Result<bool, String> {
        self.click(&self.g().id(&self.g().close))?;
        self.wait(false)
    }

    /// The rows' accessible names on the page shown, in place order; none while the panel is closed.
    fn rows(&self) -> Result<Vec<String>, String> {
        let Some(hwnd) = self.window()? else {
            return Ok(Vec::new());
        };
        let items = live::items(&self.reader, hwnd, UIA_ListItemControlTypeId);
        let items = kept(items, || live::shown(hwnd))?.unwrap_or_default();
        Ok(placed(&items, &self.g().id(&self.g().row)))
    }

    /// The first row's accessible name once a row shows within `[g23] panel_wait_ms`; none when none did.
    pub fn first_row(&self) -> Result<Option<String>, String> {
        let first = || Ok(self.rows()?.into_iter().next());
        poll_read(self.g().panel_wait_ms, self.ctx.cfg.timing.poll_ms, first)
    }
}

/// Opens the panel with the keyboard's `fill` key, or shuts it, unless it is so already, and waits for it to follow;
/// the click goes out only while `app` is in front with the gate's page.
pub fn fill_key(app: &Opened, panel: &Panel, open: bool) -> Result<(), String> {
    if panel.window()?.is_some() == open {
        return Ok(());
    }
    let id = board::id_of(Role::Fill)?;
    ownkeys::click(&Real, app, &|| Keyboard::find()?.click(&id))?;
    panel.shows(open)
}
