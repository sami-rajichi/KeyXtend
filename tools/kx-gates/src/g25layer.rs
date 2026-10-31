//! G25's shortcuts layer on the product: its keys clicked in Notepad, each checked on the clipboard, the saved file or
//! the task switcher; then the layer goes back to the page it showed first, shown or hidden as it was.
#![cfg(windows)]

use std::cell::Cell;

use kx_mod_keyboard::Role;
use kx_mod_keyboard::spec::{NEXT_ID, shortcut_key_id};
use kx_module_api::ports::ShortcutId;
use kx_platform_windows::clip::{self, open};
use serde_json::{Value, json};

use crate::apps::{Ctx, Opened};
use crate::g25::{self, part};
use crate::g25watch::Watch;
use crate::g25wins::{self, EscTo};
use crate::modekey::ModeKey;
use crate::ownkeys::{self, Real};
use crate::product::{self, Keyboard};
use crate::win::{self, sleep_ms};
use crate::{board, clipkeep, readback};

/// Selects all of Notepad's text.
const SELECT_ALL: &str = "select-all";
/// Copies the selection.
const COPY: &str = "copy";
/// Pastes at the caret.
const PASTE: &str = "paste";
/// Undoes the paste.
const UNDO: &str = "undo";
/// Selects all and copies it, in two steps.
const COPY_ALL: &str = "copy-all";
/// Keeps the task switcher open.
const SWITCH_APP: &str = "switch-app";

/// The layer key of shortcut `name`.
fn key(name: &str) -> Result<String, String> {
    let id = ShortcutId::new(name).ok_or_else(|| format!("{name} is no shortcut id"))?;
    Ok(shortcut_key_id(&id))
}

/// The page of `pages` whose first key `shows`; none while no page's does, as before the layer is drawn.
fn page_of(pages: &[Vec<String>], shows: impl Fn(&str) -> bool) -> Option<usize> {
    pages
        .iter()
        .position(|p| p.first().is_some_and(|id| shows(id)))
}

/// The page of `pages` that holds key `id`.
fn page_with(pages: &[Vec<String>], id: &str) -> Option<usize> {
    pages.iter().position(|p| p.iter().any(|k| k == id))
}

/// How many Next clicks take a layer of `count` pages from page `from` to page `to`, as Next wraps.
fn turns(from: usize, to: usize, count: usize) -> usize {
    if count == 0 {
        return 0;
    }
    (to % count + count - from % count) % count
}

/// The layer as G25 drives it: Notepad, each page's key ids as the product draws them, and the page shown now.
struct Layer<'a> {
    ctx: &'a Ctx<'a>,
    app: &'a Opened,
    pages: Vec<Vec<String>>,
    at: Cell<usize>,
}

impl Layer<'_> {
    /// Reads the keyboard until `read` takes it, up to G25's switch wait; else the error `why`.
    fn read<T>(&self, why: &str, read: impl Fn(Keyboard) -> Option<T>) -> Result<T, String> {
        let (wait, poll) = (self.ctx.cfg.g25.switch_wait_ms, self.ctx.cfg.timing.poll_ms);
        let found = || Keyboard::find().ok().and_then(&read);
        win::poll_until(wait, poll, found).ok_or_else(|| why.to_string())
    }

    /// The page the layer shows once its keys are drawn, which it goes back to at the end.
    fn first(&self) -> Result<usize, String> {
        let shown = |kb: Keyboard| page_of(&self.pages, |id| kb.at(id).is_ok());
        let page = self.read("the layer's keys did not show", shown)?;
        self.at.set(page);
        Ok(page)
    }

    /// The keyboard once page `page`'s first key shows.
    fn showing(&self, page: usize) -> Result<Keyboard, String> {
        let first = self.pages.get(page).and_then(|p| p.first());
        let first = first.ok_or("the layer has no such page")?;
        let why = format!("key {first} did not show");
        self.read(&why, |kb| kb.at(first).is_ok().then_some(kb))
    }

    /// Turns to page `page` with Next, waiting for each page in turn to show; the keyboard then.
    fn turn_to(&self, page: usize) -> Result<Keyboard, String> {
        let count = self.pages.len();
        let mut kb = self.showing(self.at.get())?;
        for _ in 0..turns(self.at.get(), page, count) {
            kb.click(NEXT_ID)?;
            self.at.set((self.at.get() + 1) % count);
            kb = self.showing(self.at.get())?;
        }
        Ok(kb)
    }

    /// Clicks shortcut `name` on its page, only while Notepad is in front with our tab, so its keys reach it alone.
    fn press(&self, name: &str) -> Result<(), String> {
        let id = key(name)?;
        let page = page_with(&self.pages, &id).ok_or_else(|| format!("{id} is on no page"))?;
        let kb = self.turn_to(page)?;
        ownkeys::click(&Real, self.app, &|| kb.click(&id))
    }

    /// Clicks shortcut `name`, then reads the clipboard once it changed: true when it holds G25's text.
    fn copies(&self, name: &str) -> Result<bool, String> {
        let t = &self.ctx.cfg.timing;
        let before = open::sequence();
        self.press(name)?;
        let changed = open::wait_change(before, clipkeep::wait(t));
        let text = clip::read_text(clipkeep::wait(t), self.ctx.cfg.g1.max_read_chars)
            .map_err(|e| e.to_string())?;
        Ok(changed && text.as_deref() == Some(self.ctx.cfg.g25.text.as_str()))
    }

    /// Clicks shortcut `name`, then saves Notepad until its text is `want`: true once it is.
    fn edits(&self, name: &str, want: &str) -> Result<bool, String> {
        self.press(name)?;
        sleep_ms(self.ctx.cfg.probes.settle_ms);
        readback::saved_until(self.ctx, self.app, |s| s.trim_end() == want)
    }

    /// Select all and Copy, Ctrl+End from kx-gates and Paste, Undo, then Copy all; each checked on the clipboard or
    /// the saved file.
    fn shortcuts(&self) -> Result<Value, String> {
        let text = &self.ctx.cfg.g25.text;
        self.press(SELECT_ALL)?;
        sleep_ms(self.ctx.cfg.timing.key_settle_ms);
        let copy = self.copies(COPY)?;
        ownkeys::send(&Real, self.app, &self.ctx.cfg.edit_keys.doc_end)?;
        let paste = self.edits(PASTE, &text.repeat(2))?;
        let undo = self.edits(UNDO, text)?;
        let copy_all = self.copies(COPY_ALL)?;
        let ok = copy && paste && undo && copy_all;
        Ok(json!({ "copy": copy, "paste": paste, "undo": undo, "copy_all": copy_all, "ok": ok }))
    }

    /// Switch app shows the task switcher, which Esc closes; G25 then brings Notepad back to the front.
    fn switch_app(&self) -> Result<Value, String> {
        let watch = Watch::start(&self.ctx.cfg.g25.switcher_classes, None);
        self.press(SWITCH_APP)?;
        g25wins::closes(self.ctx, self.app, &watch, EscTo::WatchedFirst)
    }

    /// Both parts from the page the layer shows first, then back to that page: their results, the first saying
    /// whether it came back.
    fn drive(&self) -> Result<(Value, Value), String> {
        let start = self.first()?;
        let mut shortcuts = part(self.shortcuts());
        let switch = part(self.switch_app());
        let back = self.turn_to(start).is_ok();
        shortcuts["page_back"] = json!(back);
        shortcuts["ok"] = json!(shortcuts["ok"] == true && back);
        Ok((shortcuts, switch))
    }
}

/// Shows the layer, runs its parts, then puts it back on the page it showed first and shown or hidden as it was,
/// whatever happened: the `shortcuts` and `switch_app` results.
pub fn run(ctx: &Ctx, app: &Opened) -> Result<(Value, Value), String> {
    let pages = board::layer_pages(&product::keyboard()?.shortcut_pages)?;
    let (mode, wait) = (ModeKey::of(Role::Layer)?, g25::wait(ctx));
    let was = mode.on()?;
    let layer = Layer {
        ctx,
        app,
        pages,
        at: Cell::new(0),
    };
    let driven = mode.set(true, wait).and_then(|()| layer.drive());
    let back = mode.put_back(was, wait);
    let (mut shortcuts, switch) = driven.unwrap_or_else(|e| (part(Err(e.clone())), part(Err(e))));
    shortcuts["layer_as_found"] = json!(back.is_ok());
    shortcuts["ok"] = json!(shortcuts["ok"] == true && back.is_ok());
    if let Err(e) = back {
        shortcuts["layer_error"] = json!(e);
    }
    Ok((shortcuts, switch))
}

#[cfg(test)]
mod tests {
    use kx_mod_keyboard::KeyboardSettings;

    use super::*;

    fn pages() -> Vec<Vec<String>> {
        let page = |ids: &[&str]| ids.iter().map(|&id| id.to_owned()).collect();
        vec![
            page(&["sc-a", "sc-b", "layer-next"]),
            page(&["sc-c", "layer-next"]),
            page(&["sc-d", "layer-next"]),
        ]
    }

    #[test]
    fn next_turns_forward_and_wraps() {
        assert_eq!(turns(0, 1, 3), 1);
        assert_eq!(turns(1, 0, 3), 2, "back to page 1 through page 3");
        assert_eq!(turns(2, 2, 3), 0);
        assert_eq!(turns(0, 0, 1), 0);
        assert_eq!(turns(0, 0, 0), 0, "no page, no turn");
    }

    #[test]
    fn a_page_is_known_by_its_first_key_and_a_key_by_its_page() {
        let p = pages();
        assert_eq!(
            page_of(&p, |id| id == "sc-c" || id == "layer-next"),
            Some(1)
        );
        assert_eq!(
            page_of(&p, |id| id == "layer-next"),
            None,
            "Next is on every page"
        );
        assert_eq!(page_of(&p, |_| false), None, "not drawn yet");
        assert_eq!(page_with(&p, "sc-d"), Some(2));
        assert_eq!(page_with(&p, "sc-x"), None);
    }

    #[test]
    fn the_shortcuts_g25_clicks_are_on_the_products_default_pages() {
        let defaults = KeyboardSettings::from_defaults().expect("defaults");
        let pages = board::layer_pages(&defaults.shortcut_pages).expect("the board reads");
        for name in [SELECT_ALL, COPY, PASTE, UNDO, COPY_ALL, SWITCH_APP] {
            let id = key(name).expect("a shortcut id");
            assert!(page_with(&pages, &id).is_some(), "{id} is on no page");
        }
        let switch = key(SWITCH_APP).expect("a shortcut id");
        assert_eq!(page_with(&pages, &switch), Some(1), "page 2");
        assert!(key("Copy").is_err(), "not an id's form");
    }
}
