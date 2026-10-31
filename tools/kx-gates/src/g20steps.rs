//! G20's steps on a running copy with the history key on: the gate's copies, the rows they show, the searches typed
//! on the product's keys with their ticks, and Insert's text read back from the gate's own Notepad tab.
#![cfg(windows)]

use kx_mod_clipboard::ClipSettings;
use kx_platform_windows::clip::{self, open};
use serde_json::{Value, json};

use crate::apps::{Ctx, Opened};
use crate::clipnames::{Names, RowName};
use crate::clipown::Owner;
use crate::clippanel::Panel;
use crate::g20plan::{found, pasted, rows_part, saved_as, shown};
use crate::g27type::Typist;
use crate::ownkeys::Real;
use crate::p8cfg::{G20Product, Search};
use crate::win::sleep_ms;
use crate::{clipkeep, g20keys, g20tab, readback};

/// The steps' panel, its names in every language, and the gate's Notepad tab.
pub struct Steps<'a> {
    ctx: &'a Ctx<'a>,
    app: &'a Opened,
    /// The product's clipboard panel.
    pub panel: Panel<'a>,
    names: Names,
}

impl<'a> Steps<'a> {
    /// The steps for `app`, the gate's Notepad tab, on the copy that runs now.
    pub fn new(ctx: &'a Ctx<'a>, app: &'a Opened) -> Result<Self, String> {
        let names = Names::load(&ctx.cfg.g20_product)?;
        Ok(Self {
            ctx,
            app,
            panel: Panel::new(ctx, ctx.cfg.g20_product.panel_wait_ms)?,
            names,
        })
    }

    /// The G20 settings.
    fn g(&self) -> &G20Product {
        &self.ctx.cfg.g20_product
    }

    /// Writes the gate's copies a gap apart: the plain ones oldest first and one per marker, the cleared one and its
    /// clear through the gate's own owner, then the skipped app's copy in Notepad.
    pub fn copy_all(&self) -> Result<(), String> {
        let (g, t) = (self.g(), &self.ctx.cfg.timing);
        let wait = clipkeep::wait(t);
        let write = |text: &str, zero: &[&str]| {
            clip::write_text(Some(text), zero, wait).map_err(|e| e.to_string())
        };
        for text in g.texts() {
            write(text, &[])?;
            sleep_ms(g.copy_gap_ms);
        }
        for marker in clip::markers() {
            write(&g.marked, &[marker.as_str()])?;
            sleep_ms(g.copy_gap_ms);
        }
        let owner = Owner::new()?;
        for text in [Some(g.cleared.as_str()), None] {
            owner.write(text, t)?;
            owner.hold(g.copy_gap_ms, t.poll_ms);
        }
        drop(owner);
        self.copy_skipped()
    }

    /// Copies Notepad's text with Ctrl+A and Ctrl+C in the gate's own tab, then confirms the clipboard holds the
    /// skipped text, so "it never shows" judges a copy that really happened.
    fn copy_skipped(&self) -> Result<(), String> {
        let (g, cfg) = (self.g(), self.ctx.cfg);
        let wait = clipkeep::wait(&cfg.timing);
        let before = open::sequence();
        g20tab::copy_in_tab(&Real, self.app, cfg)?;
        let changed = open::wait_change(before, wait);
        let read = || clip::read_text(wait, cfg.g1.max_read_chars).map_err(|e| e.to_string());
        let got = if changed { read()? } else { None };
        g20tab::skipped_ok(changed, got.as_deref(), &g.skipped)?;
        sleep_ms(g.copy_gap_ms);
        Ok(())
    }

    /// The rows on the page shown, each read back as its title and tick.
    fn read_rows(&self) -> Result<Vec<RowName>, String> {
        Ok(self
            .panel
            .rows()?
            .iter()
            .map(|n| self.names.row(n))
            .collect())
    }

    /// The rows' titles on the page shown.
    fn titles(&self) -> Result<Vec<String>, String> {
        Ok(self.read_rows()?.into_iter().map(|r| r.title).collect())
    }

    /// Opens the panel with the clip key and checks its rows: the plain copies newest first and no hidden one.
    pub fn rows(&self) -> Result<Value, String> {
        let g = self.g();
        g20keys::clip_key(self.app, &self.panel, true)?;
        sleep_ms(g.settle_ms);
        let ours: Vec<&str> = g.texts().into_iter().chain(g.hidden()).collect();
        Ok(rows_part(
            &self.titles()?,
            &shown(&g.texts()),
            &g.hidden(),
            &ours,
        ))
    }

    /// Ticks the first row a search found, as tick number `n`: whether that row then reads ticked so.
    fn tick(&self, want: Option<&String>, n: usize) -> Result<bool, String> {
        self.panel.click_row(0)?;
        sleep_ms(self.g().settle_ms);
        let first = self.read_rows()?.into_iter().next();
        let number = u16::try_from(n).ok();
        Ok(first.is_some_and(|r| r.tick == number && Some(&r.title) == want))
    }

    /// Types search `s` on the product's keys and checks the line and the rows found, ticks the first when `s` says,
    /// adding its title to `ticks`, then clears the search with the product's Esc.
    fn search(&self, t: &Typist, s: &Search, ticks: &mut Vec<String>) -> Result<Value, String> {
        let g = self.g();
        t.type_in(t.switch(&s.lang)?, &s.typed)?;
        sleep_ms(g.settle_ms);
        let want = found(&shown(&g.texts()), &s.typed);
        let (line, titles) = (self.panel.search()?, self.titles()?);
        let mut part = json!({ "lang": s.lang, "typed": s.typed, "line_ok": line == s.typed, "found_ok": titles == want });
        if s.tick {
            let ticked = self.tick(want.first(), ticks.len() + 1)?;
            ticks.extend(want.first().cloned());
            part["tick_ok"] = json!(ticked);
        }
        g20keys::esc(self.app)?;
        sleep_ms(g.settle_ms);
        part["cleared"] = json!(self.panel.search()?.is_empty());
        let ok = ["line_ok", "found_ok", "cleared"]
            .iter()
            .all(|k| part[k] == true)
            && part["tick_ok"] != false;
        part["ok"] = json!(ok);
        Ok(part)
    }

    /// Every search in turn through G27's typist: each one's part, and the titles ticked in order.
    pub fn searches(&self, t: &Typist) -> (Vec<Value>, Vec<String>) {
        let mut ticks = Vec::new();
        let parts = self.g().searches.iter().map(|s| {
            let part = self.search(t, s, &mut ticks);
            part.unwrap_or_else(|e| json!({ "lang": s.lang, "error": e, "ok": false }))
        });
        (parts.collect(), ticks)
    }

    /// Selects Notepad's text, then clicks Insert, whose label must count the `ticks`: the panel must close and the
    /// saved tab hold the ticked texts in order, the join between.
    pub fn insert(&self, ticks: &[String]) -> Result<Value, String> {
        let join = ClipSettings::from_defaults()
            .map_err(|e| e.to_string())?
            .join_config();
        let want = pasted(ticks, &join);
        let count = self.names.count(&self.panel.label(&self.g().insert)?);
        let click = || self.panel.click(&self.g().insert);
        g20tab::paste(&Real, self.app, self.ctx.cfg, &click)?;
        let closed = self.panel.wait(false).is_ok();
        let saved = readback::saved_until(self.ctx, self.app, |s| saved_as(s, &want))?;
        let count_ok = count.map(usize::from) == Some(ticks.len());
        Ok(json!({
            "ticks": ticks.len(), "count": count, "count_ok": count_ok, "closed": closed, "saved_ok": saved,
            "ok": count_ok && closed && saved,
        }))
    }
}
