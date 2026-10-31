//! The clipboard panel from the keyboard's side, over the bus (spec §7, ADR-0034): the clip key and the history chord
//! ask for the panel, the panel's keys reach it instead of the app, and one plain paste goes out when the module asks.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

mod chord;
mod paste;
mod search;

use crate::assist::{Bench, ear};
use crate::rig::{HELD, ROWS, STATE};
use kx_module_api::ports::{
    ClipKey, ClipOpen, ClipPanel, FaceAction, FaceReply, HistoryAsk, HistoryWanted, PanelKey,
    Pasted, Scan, TextRoute,
};
use kx_module_api::{Bus, Redacted, Subscription};
use kx_platform_fake::Stroke;
use std::sync::{Arc, Mutex, PoisonError};

/// The language list's design, for the tests that open it.
const LIST: (&str, &str) = ("lang_key", "list");

/// What the keyboard said to the clipboard module.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Heard {
    /// It asked whether the history chord is wanted.
    Ask,
    /// The clip key or the history chord asked to open or close the panel.
    Open,
    /// A key for the open panel.
    Key(PanelKey),
    /// Its answer to a paste request.
    Pasted(Pasted),
}

/// The keyboard on a bus the test listens to, with what it said to the clipboard module.
pub struct Clip {
    pub b: Bench,
    heard: Arc<Mutex<Vec<Heard>>>,
}

impl Clip {
    /// The keyboard with `change` over the defaults and `module`'s handlers on the bus before it starts.
    pub fn after(
        change: Option<(&str, toml::Value)>,
        module: impl FnOnce(&Bus) -> Vec<Subscription>,
    ) -> Self {
        let heard = Arc::default();
        let h = Arc::clone(&heard);
        let b = Bench::after(change, move |bus| {
            let mut ears = vec![
                ear(bus, &h, |_: &HistoryAsk| Heard::Ask),
                ear(bus, &h, |_: &ClipOpen| Heard::Open),
                ear(bus, &h, |e: &ClipKey| Heard::Key(e.0.clone())),
                ear(bus, &h, |e: &Pasted| Heard::Pasted(*e)),
            ];
            ears.extend(module(bus));
            ears
        });
        Self { b, heard }
    }

    /// The keyboard started with `change`, with what it said at start already taken.
    pub fn quiet(change: Option<(&str, toml::Value)>) -> Self {
        let c = Self::after(change, |_| Vec::new());
        c.heard();
        c.b.said();
        c
    }

    /// What the keyboard said to the clipboard module since the last look.
    pub fn heard(&self) -> Vec<Heard> {
        let mut heard = self.heard.lock().unwrap_or_else(PoisonError::into_inner);
        std::mem::take(&mut *heard)
    }

    /// The clipboard module says its panel opened or closed.
    pub fn open(&self, open: bool) {
        self.b.send(&ClipPanel { open });
    }

    /// The clipboard module says whether it wants the history chord.
    pub fn want(&self, on: bool) {
        self.b.send(&HistoryWanted { on });
    }

    /// What reached the app since the last look.
    pub fn strokes(&self) -> Vec<Stroke> {
        let injector = &self.b.rig.fakes.injector;
        let strokes = injector.strokes();
        injector.clear();
        strokes
    }

    /// Whether key `id` is lit.
    pub fn on(&self, id: &str) -> bool {
        self.b.key(id)["on"] == true
    }
}

/// Search text `t` for the panel.
pub fn text(t: &str) -> Heard {
    Heard::Key(PanelKey::Text(Redacted::new(t.into())))
}

/// Text typed as Unicode.
fn unicode(t: &str) -> Stroke {
    Stroke::Text(Redacted::new(t.into()), TextRoute::Unicode)
}

#[test]
fn start_and_restart_ask_whether_the_history_chord_is_wanted() {
    let mut c = Clip::after(None, |_| Vec::new());
    assert_eq!(c.heard(), [Heard::Ask]);
    c.b.rig.restart(Some(("lang_key", "cycle".into())));
    assert_eq!(c.heard(), [Heard::Ask]);
}

#[test]
fn the_clip_key_asks_for_the_panel_types_nothing_and_keeps_the_latches() {
    let c = Clip::quiet(None);
    c.b.rig.tap("shift").unwrap();
    assert_eq!(c.b.rig.tap("clip"), Ok(STATE));
    assert_eq!(c.heard(), [Heard::Open]);
    assert!(c.strokes().is_empty());
    assert!(c.on("shift"), "the latches stay");
}

#[test]
fn the_clip_key_lights_while_the_module_last_said_its_panel_is_open() {
    let c = Clip::quiet(None);
    assert!(!c.on("clip"));
    c.open(true);
    let relabel = c.b.rig.act(FaceAction::Relabel);
    assert_eq!(relabel, Ok(STATE), "the next relabel redraws");
    assert!(c.on("clip"));
    c.open(false);
    assert!(!c.on("clip"));
}

#[test]
fn with_the_panel_open_space_ticks_backspace_erases_and_the_moves_drive_it() {
    let c = Clip::quiet(None);
    c.open(true);
    for id in ["space", "bksp", "up", "down", "left", "right", "enter"] {
        assert_eq!(c.b.rig.press(id), Ok(STATE), "{id} never holds");
    }
    c.b.rig.tap("esc").unwrap();
    let keys = [
        PanelKey::Tick,
        PanelKey::Erase,
        PanelKey::Up,
        PanelKey::Down,
        PanelKey::Left,
        PanelKey::Right,
        PanelKey::Enter,
        PanelKey::Esc,
    ];
    assert_eq!(c.heard(), keys.map(Heard::Key));
    assert!(c.strokes().is_empty(), "nothing reaches the app");
    let sent = c.b.rig.fakes.shortcuts.sent();
    assert!(sent.is_empty(), "the keyboard pastes only on PasteAsk");
}

#[test]
fn with_the_panel_open_a_letter_goes_to_the_search_and_not_to_the_app() {
    let c = Clip::quiet(None);
    c.open(true);
    assert_eq!(c.b.rig.press("c-0x0010"), Ok(STATE));
    c.b.rig.tap("c-0x0011").unwrap();
    assert_eq!(c.heard(), [text("q"), text("w")]);
    assert!(c.strokes().is_empty());
    assert!(c.b.rig.fakes.shortcuts.sent().is_empty());
}

#[test]
fn with_the_panel_shut_space_backspace_and_letters_type_as_usual() {
    let c = Clip::quiet(None);
    c.open(true);
    c.open(false);
    for id in ["space", "bksp", "c-0x0010"] {
        c.b.rig.tap(id).unwrap();
    }
    assert!(c.heard().is_empty());
    let tap = |code| Stroke::Tap(Scan::new(code));
    assert_eq!(c.strokes(), [tap(0x39), tap(0x0E), unicode("q")]);
}

#[test]
fn ctrl_alt_or_win_latched_sends_a_letter_to_the_app_as_usual() {
    let c = Clip::quiet(None);
    c.open(true);
    for (latch, code) in [("ctrl", 0x1D), ("alt", 0x38), ("win", 0xE05B)] {
        c.b.rig.tap(latch).unwrap();
        c.b.rig.press("c-0x002E").unwrap();
        let held = vec![Scan::new(code)];
        assert_eq!(
            c.strokes(),
            [Stroke::Chord(held, Scan::new(0x2E))],
            "{latch}"
        );
    }
    assert!(c.heard().is_empty());
}

#[test]
fn opening_the_panel_closes_the_language_list() {
    let c = Clip::quiet(Some((LIST.0, LIST.1.into())));
    c.b.rig.tap("lang").unwrap();
    assert!(!c.b.rig.state()["list"].is_null());
    c.open(true);
    assert!(c.b.rig.state()["list"].is_null(), "one panel at a time");
}

#[test]
fn a_language_list_opened_over_the_panel_takes_its_moves_and_leaves_it_the_rest() {
    let c = Clip::quiet(Some((LIST.0, LIST.1.into())));
    c.open(true);
    c.b.rig.tap("lang").unwrap();
    assert_eq!(c.b.rig.panel("down", ROWS), Ok(STATE), "the list moves");
    assert_eq!(c.b.rig.tap("up"), Ok(FaceReply::default()));
    c.b.rig.press("space").unwrap();
    assert_eq!(c.heard(), [Heard::Key(PanelKey::Tick)]);
    assert_eq!(
        c.b.rig.press("esc"),
        Ok(FaceReply::default()),
        "Esc waits for the list"
    );
    c.b.rig.panel("esc", ROWS).unwrap();
    assert!(
        c.b.rig.state()["list"].is_null(),
        "Esc closed the list first"
    );
    assert!(c.heard().is_empty() && c.on("clip"), "the panel stays open");
    c.b.rig.tap("esc").unwrap();
    assert_eq!(
        c.heard(),
        [Heard::Key(PanelKey::Esc)],
        "then Esc is the panel's"
    );
    assert!(c.strokes().is_empty());
}

#[test]
fn a_restart_forgets_the_open_panel_until_told_again() {
    let mut c = Clip::quiet(None);
    c.open(true);
    c.b.rig.restart(Some(("lang_key", "cycle".into())));
    assert_eq!(c.heard(), [Heard::Ask]);
    assert!(!c.on("clip"));
    c.b.rig.tap("space").unwrap();
    assert_eq!(c.strokes(), [Stroke::Tap(Scan::new(0x39))]);
}

#[test]
fn a_held_letter_with_the_panel_shut_still_repeats() {
    let c = Clip::quiet(None);
    assert_eq!(c.b.rig.press("c-0x0010"), Ok(HELD));
    assert!(c.heard().is_empty());
}
