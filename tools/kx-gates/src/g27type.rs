//! Typing through the product's keys and chips into another app's box: the layout in front, the keys for a prefix, the
//! first chip that offers it, and its click. G27, G21's product half and G8's typing churns share it.
#![cfg(windows)]

use std::time::Instant;

use kx_mod_predict::script;
use kx_module_api::Redacted;
use kx_module_api::ports::LayoutId;
use kx_platform_windows::uia::{self, Uia};
use kx_platform_windows::{WinLayouts, layouts};
use spike_core::geom;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Accessibility::IUIAutomationElement;

use crate::apps::{Ctx, Opened};
use crate::g27lane::{self, Lane};
use crate::keymap::Reader;
use crate::ownkeys::{self, Real};
use crate::p7cfg::{Case, G27};
use crate::product::{self, Keyboard};
use crate::simuser;
use crate::walk::{self, CharKey};
use crate::win::{self, sleep_ms};
use crate::{board, capture, ftl, langs};

/// The keys among `keys` that type each character of `prefix` at once, in order; an error names the first character
/// no key types.
pub fn keys_for(keys: &[CharKey], prefix: &str) -> Result<Vec<CharKey>, String> {
    let key = |c: char| {
        let want = c.to_string();
        let found = keys
            .iter()
            .find(|k| k.types.as_deref() == Some(want.as_str()));
        let none = || format!("no key types U+{:04X} in this layout", u32::from(c));
        found.cloned().ok_or_else(none)
    };
    prefix.chars().map(key).collect()
}

/// The layout `app`'s focus types in; none when the window is gone.
fn layout_of(app: &Opened) -> Option<LayoutId> {
    langs::layout_of(app.hwnd).map(langs::id)
}

/// Types through the product into `app`'s box, and puts back the layout the app had first.
pub struct Typist<'a> {
    ctx: &'a Ctx<'a>,
    app: &'a Opened,
    kb: HWND,
    reader: Reader,
    uia: Uia,
    forms: Vec<(String, String)>,
    first: Option<LayoutId>,
}

impl<'a> Typist<'a> {
    /// A typist for `app`, in front, while the product's keyboard shows.
    pub fn new(ctx: &'a Ctx<'a>, app: &'a Opened) -> Result<Self, String> {
        let g = &ctx.cfg.g27;
        Ok(Self {
            ctx,
            app,
            kb: Keyboard::find()?.hwnd,
            reader: Reader::new()?,
            uia: Uia::new().map_err(|e| e.to_string())?,
            forms: ftl::around(&g.chip_name, &g.chip_word)?,
            first: layout_of(app),
        })
    }

    /// The app it types for.
    pub fn app(&self) -> &Opened {
        self.app
    }

    /// The G27 settings.
    fn g(&self) -> &G27 {
        &self.ctx.cfg.g27
    }

    /// Calls `f` every `[timing] poll_ms` until it gives something, for up to `wait_ms`.
    fn wait_for<T>(&self, wait_ms: u64, f: impl FnMut() -> Option<T>) -> Option<T> {
        win::poll_until(wait_ms, self.ctx.cfg.timing.poll_ms, f)
    }

    /// Asks the app's focus for layout `to`, as the language hotkey does, and waits until it types in it.
    fn ask(&self, to: LayoutId) -> Result<(), String> {
        layouts::ask(self.app.hwnd, to).map_err(|e| format!("layout request: {e}"))?;
        let wait = self.g().switch_wait_ms;
        let took = || (layout_of(self.app) == Some(to)).then_some(());
        self.wait_for(wait, took)
            .ok_or_else(|| format!("the app did not take layout {to} within {wait} ms"))
    }

    /// Puts the first installed layout of `lang` in front of the app: the layout.
    pub fn switch(&self, lang: &str) -> Result<LayoutId, String> {
        let to = capture::find(&WinLayouts, lang)
            .ok_or_else(|| format!("no installed {lang} layout"))?;
        self.ask(to)?;
        Ok(to)
    }

    /// Waits one follow period, then until the keyboard names `keys` after the layout in front.
    fn follow(&self, keys: &[CharKey]) -> Result<(), String> {
        sleep_ms(product::follow_ms(self.ctx.cfg.timing.poll_ms)?);
        let named = || {
            let read = self.reader.walk(self.kb).ok()?;
            walk::misnamed(&read, keys).is_empty().then_some(())
        };
        self.wait_for(self.g().switch_wait_ms, named)
            .ok_or_else(|| "the keyboard's keys did not follow the layout".to_string())
    }

    /// Clicks the product's keys that type `prefix` in `layout`, with the app in front, pausing after each: when the
    /// last click went out.
    pub fn type_in(&self, layout: LayoutId, prefix: &str) -> Result<Instant, String> {
        let keys = keys_for(&walk::char_keys(layout)?, prefix)?;
        self.follow(&keys)?;
        let kb = Keyboard::find()?;
        let mut last = Instant::now();
        for k in &keys {
            ownkeys::click(&Real, self.app, &|| kb.click(&k.id))?;
            last = Instant::now();
            sleep_ms(self.g().key_gap_ms);
        }
        Ok(last)
    }

    /// The lane now.
    pub fn lane(&self) -> Result<Lane, String> {
        g27lane::read(&self.reader, self.kb, self.g())
    }

    /// Waits up to `[g27] chip_wait_ms` for a lane `pick` takes.
    pub fn lane_until(&self, pick: impl Fn(&Lane) -> bool) -> Option<Lane> {
        self.wait_for(self.g().chip_wait_ms, || self.lane().ok().filter(&pick))
    }

    /// The best chip's word now, when it shows.
    pub fn first_word(&self) -> Option<Redacted<String>> {
        let lane = self.lane().ok()?;
        g27lane::word_of(lane.first.as_ref()?.expose(), &self.forms)
    }

    /// Waits for a best chip that offers `case`'s prefix: its word, and the ms since `since`, the last key click.
    pub fn offered(&self, case: &Case, since: Instant) -> Result<(Redacted<String>, u64), String> {
        let wait = self.g().chip_wait_ms;
        let offers = || {
            let word = self.first_word()?;
            g27lane::offers(word.expose(), &case.prefix, &case.lang).then_some(word)
        };
        let word = self.wait_for(wait, offers);
        let word = word
            .ok_or_else(|| format!("no chip offered the {} prefix within {wait} ms", case.lang))?;
        Ok((word, win::ms_since(since)))
    }

    /// Clicks the product's key `id` only while the app is in front with our tab.
    fn click(&self, id: &str) -> Result<(), String> {
        ownkeys::click(&Real, self.app, &|| Keyboard::find()?.click(id))
    }

    /// Clicks the best chip with the app in front.
    pub fn click_first(&self) -> Result<(), String> {
        self.click(&self.g().chip_id(0))
    }

    /// Ends a word begun with `prefix`: the product's key that types its script's gap, then the key pause.
    pub fn end_word(&self, prefix: &str) -> Result<(), String> {
        let gap = script::of(prefix).ok_or("the prefix has no script")?.gap();
        self.click(&board::typing(gap)?)?;
        sleep_ms(self.g().key_gap_ms);
        Ok(())
    }

    /// After `[g27] settle_ms`, reads the best chip's word and clicks it: the word.
    pub fn pick(&self) -> Result<Redacted<String>, String> {
        sleep_ms(self.g().settle_ms);
        let word = self
            .first_word()
            .ok_or("the first chip went before its click")?;
        self.click_first()?;
        Ok(word)
    }

    /// target-window's box, by its accessible name.
    fn edit(&self) -> Result<IUIAutomationElement, String> {
        let name = &self.ctx.target.edit_name;
        self.uia
            .named(self.app.hwnd, name)
            .ok_or_else(|| "no target-window box through UI Automation".to_string())
    }

    /// The text of target-window's box, at most `[g27] max_chars`; never logged.
    pub fn box_text(&self) -> Result<Redacted<String>, String> {
        let el = self.edit()?;
        let text = uia::value(&el, self.g().max_chars).map(Redacted::new);
        text.ok_or_else(|| "the box's text was not read".to_string())
    }

    /// Clicks just inside the box's left edge, halfway down: in a fresh one-line box that puts the caret before a
    /// left-to-right text. The installed gate got no text pattern from this box, so its lines are not asked.
    pub fn click_start(&self) -> Result<(), String> {
        let found = uia::rect(&self.edit()?).map_err(|e| e.to_string())?;
        let (start, _) = geom::line_points(&found, self.ctx.cfg.probes.inset_px);
        simuser::click(self.ctx, self.app, start)
    }

    /// Puts back the layout the app had first: true once it types in it again, or when it had none.
    pub fn put_back(&self) -> bool {
        self.first.is_none_or(|to| self.ask(to).is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys() -> Vec<CharKey> {
        vec![
            CharKey::new("c-0x0023", "h", Some("h")),
            CharKey::new("c-0x0012", "e", Some("e")),
            CharKey::new("c-0x0026", "l", Some("l")),
            CharKey::new("c-0x001A", "^", None),
        ]
    }

    #[test]
    fn each_character_of_the_prefix_finds_the_key_that_types_it() {
        let got = keys_for(&keys(), "hel").expect("all typed");
        let ids: Vec<&str> = got.iter().map(|k| k.id.as_str()).collect();
        assert_eq!(ids, ["c-0x0023", "c-0x0012", "c-0x0026"]);
        assert_eq!(keys_for(&keys(), "hell").map(|k| k.len()), Ok(4));
    }

    #[test]
    fn a_character_no_key_types_at_once_names_its_code_point() {
        let e = keys_for(&keys(), "hex").expect_err("no x");
        assert!(e.contains("U+0078"), "{e}");
        let e = keys_for(&keys(), "^").expect_err("a dead key waits");
        assert!(e.contains("U+005E"), "{e}");
        let ar = [CharKey::new("c-0x0023", "ا", Some("ا"))];
        assert!(keys_for(&ar, "ان").expect_err("no noon").contains("U+0646"));
    }
}
