//! One G19 run on the test build: the strip's reader, the gate's acts, and each selection case judged as the P10 design
//! says: seen with the clipboard untouched, the window in front kept, placed where `geom::anchor` puts it, and the
//! keycaps of the box's kind. The selected text is never kept: only its boxes are read.
#![cfg(windows)]

use std::time::Instant;

use kx_mod_keyboard::KeyboardSettings;
use kx_mod_selection::SelSettings;
use kx_module_api::ports::{Rect, SelChip};
use kx_platform_windows::clip::{self, open};
use kx_platform_windows::screen;
use kx_platform_windows::uia::{self, Uia};
use kx_platform_windows::window::foreground;
use kx_theme::{Shape, Themes};
use serde_json::{Value, json};
use windows::Win32::Foundation::{HWND, RECT};

use crate::apps::Ctx;
use crate::g19chips::act::Acts;
use crate::g19chips::place;
use crate::g19chips::strip::{Shown, Strip, ids};
use crate::keymap::{Reader, centre};
use crate::p10cfg::{G19Product, selection_defaults};
use crate::product::Keyboard;
use crate::{clipkeep, win};

/// What held just before an act: the window in front, the clipboard's sequence number and the time.
pub struct Before {
    front: HWND,
    seq: u32,
    at: Instant,
}

/// The run's readers and acts.
pub struct Gate<'a> {
    /// The run's settings.
    pub ctx: &'a Ctx<'a>,
    /// The strip of the test build's process.
    pub strip: Strip<'a>,
    /// The gate's acts.
    pub acts: Acts<'a>,
    /// A reader of the product's buttons.
    pub reader: Reader,
    /// The selection module's defaults, as the fresh data folder holds them.
    pub sel: SelSettings,
    uia: Uia,
    /// The shared sizes, whose `[chips]` gap sets where the strip should sit.
    shape: Shape,
    /// The keyboard's size, as the fresh data folder holds it.
    size: f64,
}

impl<'a> Gate<'a> {
    /// The readers of the copy with process id `pid`.
    pub fn new(ctx: &'a Ctx<'a>, pid: u32) -> Result<Self, String> {
        let themes = Themes::shipped().map_err(|e| format!("theme {e}"))?;
        let keyboard = KeyboardSettings::from_defaults()
            .map_err(|e| format!("the keyboard's defaults: {e}"))?;
        Ok(Self {
            ctx,
            strip: Strip::new(ctx, pid)?,
            acts: Acts::new(ctx),
            reader: Reader::new()?,
            sel: selection_defaults()?,
            uia: Uia::new().map_err(|e| e.to_string())?,
            shape: themes.shape,
            size: keyboard.size,
        })
    }

    /// The settings.
    pub fn g(&self) -> &G19Product {
        &self.ctx.cfg.g19_product
    }

    /// What holds now, before an act.
    pub fn before() -> Before {
        Before {
            front: foreground(),
            seq: open::sequence(),
            at: Instant::now(),
        }
    }

    /// The focused box's class and control type, for the notes; never its name or text.
    pub fn kind(&self) -> String {
        self.uia
            .focused()
            .map_or_else(|e| e.to_string(), |el| uia::kind(&el))
    }

    /// The boxes of the focused box's selection.
    pub fn boxes(&self) -> Result<Vec<RECT>, String> {
        let el = self.uia.focused().map_err(|e| e.to_string())?;
        let tp = self
            .uia
            .text_of(&el)
            .ok_or("the focused box has no text pattern")?;
        let sel = uia::selection(&tp, 1).map_err(|e| e.to_string())?;
        Ok(sel.ok_or("the focused box holds no selection")?.boxes)
    }

    /// The UI Automation reader of text boxes.
    pub fn uia(&self) -> &Uia {
        &self.uia
    }

    /// The verdict on where `s` sits for the focused box's selection, clear of the keyboard.
    fn place(&self, s: &Shown) -> Result<Value, String> {
        let boxes = self.boxes()?;
        let last = boxes.last().ok_or("the selection has no box")?;
        let area = place::rect(&screen::work_area(last).map_err(|e| e.to_string())?);
        let gap = place::gap(&self.shape, self.size, screen::dpi_of(last));
        let kb = Keyboard::find().ok().and_then(|k| win::rect(k.hwnd).ok());
        let kb = kb.as_ref().map(place::rect);
        let (rects, got) = (
            boxes.iter().map(place::rect).collect::<Vec<Rect>>(),
            place::rect(&s.rect),
        );
        let clear: Vec<Rect> = kb.into_iter().collect();
        let want = place::want(&rects, (got.w, got.h), gap, area, &clear).ok_or("no line box")?;
        let (first, end) = (
            place::line(&rects, 0),
            place::line(&rects, rects.len().saturating_sub(1)),
        );
        let mut v = place::judge(
            want,
            got,
            self.g().place_slack_px,
            kb,
            first.unwrap_or_default(),
        );
        v["boxes"] = json!(rects.len());
        v["one_line"] = json!(first == end);
        Ok(v)
    }

    /// Case `name` after an act from `before`: the strip shows within `strip_wait_ms` with the keycaps `want` names.
    pub fn case(
        &self,
        name: &str,
        before: &Before,
        want: &[String],
    ) -> Result<(Value, Option<Shown>), String> {
        let shown = self.strip.shows()?;
        let ms = u64::try_from(before.at.elapsed().as_millis()).unwrap_or(u64::MAX);
        let untouched = open::sequence() == before.seq;
        let mut v =
            json!({ "case": name, "focus": foreground() == before.front, "kind": self.kind() });
        let Some(s) = shown else {
            v["seen"] = json!(false);
            v["chips"] = json!(false);
            return Ok((v, None));
        };
        let chips = ids(&s.chips(), self.g());
        let placed = self
            .place(&s)
            .unwrap_or_else(|e| json!({ "ok": false, "error": e }));
        v["seen"] = json!(untouched);
        v["chips"] = json!(chips == want);
        v["shown"] = json!(chips);
        v["ms"] = json!(ms);
        v["place"] = placed["ok"].clone();
        v["placed"] = placed;
        Ok((v, Some(s)))
    }

    /// Case `name` after an act from `before`: no strip shows for `quiet_ms`.
    pub fn quiet(&self, name: &str, before: &Before) -> Result<Value, String> {
        let quiet = self.strip.quiet()?;
        Ok(
            json!({ "case": name, "chips": quiet, "focus": foreground() == before.front, "kind": self.kind() }),
        )
    }

    /// Clicks keycap `chip` on `s`: whether the window in front stayed, and the clipboard's number before the click.
    pub fn tap(&self, s: &Shown, chip: SelChip) -> Result<(bool, u32), String> {
        let (_, r) = (s.caps.iter())
            .find(|(c, _)| *c == chip)
            .ok_or_else(|| format!("the strip shows no {chip:?}"))?;
        let (front, seq) = (foreground(), open::sequence());
        self.acts.press_on(s.hwnd, centre(r))?;
        Ok((foreground() == front, seq))
    }

    /// True once the clipboard changed from `seq` and holds `want`, line ends aside; its text is never kept.
    pub fn clip_holds(&self, seq: u32, want: &str) -> bool {
        let t = &self.ctx.cfg.timing;
        let max = self.ctx.cfg.g1.max_read_chars;
        let read = || clip::read_text(clipkeep::wait(t), max).ok().flatten();
        open::wait_change(seq, clipkeep::wait(t)) && read().is_some_and(|c| c.trim_end() == want)
    }
}
