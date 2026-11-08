//! The strip as a screen reader meets it: the test build's window titled with kx-ui's `sel.name` text in a shipped
//! language, its keycaps by UI Automation id, left to right, and a real click on one once it shows on top.
#![cfg(windows)]

use kx_module_api::ports::SelChip;
use windows::Win32::Foundation::{HWND, RECT};

use crate::apps::Ctx;
use crate::clippanel::has_area;
use crate::g23::live::{kept, poll_read};
use crate::keymap::{KeyRead, Reader};
use crate::p10cfg::G19Product;
use crate::win::{self, sleep_ms};
use crate::{ftl, product};

/// The keycaps among `reads` with an area, as chips with their boxes, left to right.
pub fn keycaps(reads: &[KeyRead], g: &G19Product) -> Vec<(SelChip, RECT)> {
    let shown = reads.iter().filter(|k| has_area(&k.rect));
    let mut caps: Vec<(SelChip, RECT)> = shown
        .filter_map(|k| Some((g.chip_of(&k.id)?, k.rect)))
        .collect();
    caps.sort_by_key(|(_, r)| r.left);
    caps
}

/// `chips` by their ids in `g`, for the result line.
pub fn ids(chips: &[SelChip], g: &G19Product) -> Vec<String> {
    chips.iter().filter_map(|&c| g.chip_id(c)).collect()
}

/// A strip on screen: its window, its box and its keycaps.
#[derive(Debug, Clone)]
pub struct Shown {
    /// Its window.
    pub hwnd: HWND,
    /// Its window's box, in physical px.
    pub rect: RECT,
    /// Its keycaps, left to right.
    pub caps: Vec<(SelChip, RECT)>,
}

impl Shown {
    /// Its chips, left to right.
    pub fn chips(&self) -> Vec<SelChip> {
        self.caps.iter().map(|(c, _)| *c).collect()
    }
}

/// A reader of the strip of the test build's process.
pub struct Strip<'a> {
    ctx: &'a Ctx<'a>,
    pid: u32,
    titles: Vec<String>,
    reader: Reader,
}

impl<'a> Strip<'a> {
    /// A reader of process `pid`'s strip.
    pub fn new(ctx: &'a Ctx<'a>, pid: u32) -> Result<Self, String> {
        let titles = ftl::every(&ctx.cfg.g19_product.strip_title)?;
        Ok(Self {
            ctx,
            pid,
            titles,
            reader: Reader::new()?,
        })
    }

    /// The settings.
    fn g(&self) -> &G19Product {
        &self.ctx.cfg.g19_product
    }

    /// The strip's window while it shows.
    fn window(&self) -> Option<HWND> {
        let titled = |w: &HWND| self.titles.contains(&win::title(*w));
        product::shown_by(self.pid).into_iter().find(titled)
    }

    /// The strip now, with at least one keycap; none while it is hidden or went as it was read.
    pub fn now(&self) -> Result<Option<Shown>, String> {
        let Some(hwnd) = self.window() else {
            return Ok(None);
        };
        let read = kept(self.reader.walk(hwnd), || self.window() == Some(hwnd))?;
        let caps = keycaps(&read.unwrap_or_default(), self.g());
        let rect = win::rect(hwnd)?;
        Ok((!caps.is_empty()).then_some(Shown { hwnd, rect, caps }))
    }

    /// The strip once it shows within `wait_ms`, read again after `settle_ms` so its pop-in has ended.
    pub fn within(&self, wait_ms: u64) -> Result<Option<Shown>, String> {
        let poll = self.ctx.cfg.timing.poll_ms;
        if poll_read(wait_ms, poll, || self.now())?.is_none() {
            return Ok(None);
        }
        sleep_ms(self.g().settle_ms);
        self.now()
    }

    /// The strip once it shows within `strip_wait_ms`.
    pub fn shows(&self) -> Result<Option<Shown>, String> {
        self.within(self.g().strip_wait_ms)
    }

    /// True once no strip `like` shows within `gone_wait_ms`.
    pub fn goes(&self, like: impl Fn(&Shown) -> bool) -> Result<bool, String> {
        let gone = || Ok(self.now()?.filter(&like).is_none().then_some(()));
        let poll = self.ctx.cfg.timing.poll_ms;
        Ok(poll_read(self.g().gone_wait_ms, poll, gone)?.is_some())
    }

    /// True when no strip showed for `quiet_ms`.
    pub fn quiet(&self) -> Result<bool, String> {
        let poll = self.ctx.cfg.timing.poll_ms;
        Ok(poll_read(self.g().quiet_ms, poll, || self.now())?.is_none())
    }

    /// The chips of each strip seen within `wait_ms`, in order, each change once.
    pub fn seen(&self, wait_ms: u64) -> Result<Vec<Vec<SelChip>>, String> {
        let mut seen: Vec<Vec<SelChip>> = Vec::new();
        let poll = self.ctx.cfg.timing.poll_ms;
        let mut note = || -> Result<Option<()>, String> {
            let now = self.now()?.map(|s| s.chips());
            if let Some(c) = now.filter(|c| seen.last() != Some(c)) {
                seen.push(c);
            }
            Ok(None)
        };
        poll_read(wait_ms, poll, &mut note)?;
        Ok(seen)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::load;

    fn read(id: &str, left: i32, wide: i32) -> KeyRead {
        let rect = RECT {
            left,
            top: 10,
            right: left + wide,
            bottom: 40,
        };
        KeyRead {
            id: id.to_string(),
            name: String::new(),
            rect,
            invokable: true,
        }
    }

    #[test]
    fn keycaps_read_left_to_right_and_other_buttons_or_hidden_ones_are_left_out() {
        let g = load().expect("loads").g19_product;
        let reads = [
            read("sel.translate", 300, 80),
            read("sel.copy", 100, 80),
            read("bar.close", 0, 20),
            read("sel.cut", 200, 0),
            read("sel.paste", 200, 80),
        ];
        let chips: Vec<SelChip> = keycaps(&reads, &g).into_iter().map(|(c, _)| c).collect();
        assert_eq!(chips, [SelChip::Copy, SelChip::Paste, SelChip::Translate]);
        assert_eq!(ids(&chips, &g), ["sel.copy", "sel.paste", "sel.translate"]);
    }
}
