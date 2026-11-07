//! A pick names its target (ADR-0040 decision 8): one for a line reaches the open panel as one key and nothing reaches
//! the app; one whose target no longer matches the aim goes nowhere, so a late pick never erases text in the app.

use super::forms::search;
use super::told::{quiet, searching};
use super::{ELSEWHERE, LETTER, Quick};
use crate::clip::{self, Clip};
use crate::fill::{Heard, login, unicode};
use kx_mod_keyboard::KeyboardSettings;
use kx_module_api::ports::{ClipLine, FaceAction, PanelKey, PanelLine, TypeWord};
use kx_module_api::{Module, Redacted};

/// `erase` Backspaces, then `t`, for a line or the app.
pub(super) fn word(erase: u16, t: &str, line: bool) -> TypeWord {
    TypeWord {
        round: 0,
        erase,
        text: Redacted::new(t.into()),
        line,
    }
}

/// The one panel key of a word of `erase` Backspaces, then `t`.
pub(super) fn chip(erase: u16, t: &str) -> PanelKey {
    PanelKey::Word {
        erase,
        text: Redacted::new(t.into()),
    }
}

impl Quick {
    /// A pick reaches the keyboard.
    pub(super) fn pick(&self, erase: u16, t: &str, line: bool) {
        self.b.send(&word(erase, t, line));
    }

    /// Nothing reached the app or the panel since the last look.
    fn nothing_went(&self) {
        assert!(self.strokes().is_empty(), "nothing reaches the app");
        assert!(self.heard().is_empty(), "nothing reaches the panel");
    }
}

#[test]
fn an_erasing_word_for_the_line_reaches_the_panel_as_one_key_and_nothing_reaches_the_app() {
    let (q, ears) = searching();
    q.pick(2, "llo ", true);
    assert_eq!(q.heard(), [Heard::Key(chip(2, "llo "))]);
    assert!(q.strokes().is_empty());
    assert!(ears.take().is_empty(), "no Typed");
}

#[test]
fn a_word_for_the_line_with_nothing_to_erase_or_type_sends_only_what_it_asks() {
    let (q, _ears) = searching();
    q.pick(0, "lo ", true);
    assert_eq!(q.heard(), [Heard::Key(chip(0, "lo "))]);
    q.pick(1, "", true);
    assert_eq!(q.heard(), [Heard::Key(chip(1, ""))]);
    q.pick(0, "", true);
    q.nothing_went();
}

#[test]
fn a_word_for_the_clipboards_line_reaches_its_panel_as_one_key() {
    let c = Clip::quiet(None);
    c.open(true);
    c.b.send(&ClipLine(search("he")));
    c.b.send(&word(1, "ello ", true));
    assert_eq!(c.heard(), [clip::Heard::Key(chip(1, "ello "))]);
    assert!(c.strokes().is_empty());
}

#[test]
fn an_arabic_word_for_the_line_goes_whole() {
    let (q, _ears) = searching();
    q.pick(2, "مرحبا ", true);
    assert_eq!(q.heard(), [Heard::Key(chip(2, "مرحبا "))]);
}

#[test]
fn a_word_for_the_line_lets_the_latches_go_as_a_letter_does() {
    let (q, _ears) = searching();
    q.b.rig.tap("shift").unwrap();
    assert!(q.on("shift"));
    q.pick(0, "Hello ", true);
    assert!(!q.on("shift"));
}

#[test]
fn a_held_key_ends_before_a_word_for_the_line() {
    let (q, _ears) = quiet();
    q.b.rig.press(LETTER).unwrap();
    q.strokes();
    q.open(true);
    q.line(search(""));
    q.heard();
    q.pick(0, "hi ", true);
    assert_eq!(q.heard(), [Heard::Key(chip(0, "hi "))]);
    assert!(!q.b.rig.tick_in(1_000).holding, "the key no longer repeats");
    assert!(q.strokes().is_empty());
}

#[test]
fn a_word_with_a_hidden_mark_sends_nothing_and_keeps_the_latches() {
    let (q, _ears) = searching();
    q.b.rig.tap("shift").unwrap();
    q.pick(1, "he\u{200B}llo", true);
    q.nothing_went();
    assert!(q.on("shift"), "refused whole, as for the app");
}

#[test]
fn a_word_past_the_text_cap_sends_nothing() {
    let (q, _ears) = searching();
    let cap = KeyboardSettings::from_defaults().unwrap().text_cap;
    q.pick(0, &"a".repeat(usize::from(cap) + 1), true);
    q.pick(cap + 1, "", true);
    q.nothing_went();
}

#[test]
fn a_shortcut_latch_refuses_a_word_for_the_line() {
    let (q, _ears) = searching();
    q.b.rig.tap("ctrl").unwrap();
    q.pick(0, "hi ", true);
    q.nothing_went();
    assert!(q.on("ctrl"));
}

#[test]
fn a_stopped_module_sends_nothing() {
    let (mut q, _ears) = searching();
    // The kernel drops the handlers before the stop, but one may still be running on the sender's thread.
    q.b.rig.module.stop();
    q.pick(1, "hi ", true);
    q.nothing_went();
}

#[test]
fn a_waiting_fill_holds_a_word_for_the_line_back() {
    let (q, _ears) = searching();
    q.fill(1, login());
    q.strokes();
    q.aim();
    q.pick(1, "hi ", true);
    assert!(q.heard().is_empty());
    assert!(q.strokes().is_empty());
}

#[test]
fn a_late_word_for_the_line_after_the_aim_left_is_dropped_and_nothing_is_erased_in_the_app() {
    let (q, _ears) = searching();
    q.away();
    q.pick(3, "hello ", true);
    q.nothing_went();
}

#[test]
fn a_word_for_the_line_with_another_window_in_front_is_dropped() {
    let (q, _ears) = searching();
    q.window(Some(ELSEWHERE));
    q.pick(3, "hello ", true);
    q.nothing_went();
}

#[test]
fn a_word_for_the_line_with_no_panel_or_no_search_aimed_is_dropped() {
    let (q, _ears) = quiet();
    q.pick(1, "hi ", true);
    q.nothing_went();
    q.open(true);
    q.heard();
    q.pick(1, "hi ", true);
    q.nothing_went();
    for line in [PanelLine::Private, PanelLine::None] {
        q.line(line);
        q.pick(1, "hi ", true);
        q.nothing_went();
    }
}

#[test]
fn a_late_word_for_the_app_while_a_line_is_aimed_is_dropped() {
    let (q, _ears) = searching();
    q.pick(3, "hello ", false);
    q.nothing_went();
    q.line(PanelLine::Private);
    q.pick(3, "hello ", false);
    q.nothing_went();
}

#[test]
fn a_word_for_the_app_goes_while_the_aimed_panel_shows_no_line() {
    let (q, _ears) = searching();
    q.line(PanelLine::None);
    q.pick(0, "hello ", false);
    assert_eq!(q.strokes(), [unicode("hello ")]);
    assert!(q.heard().is_empty());
}

#[test]
fn a_word_for_the_app_goes_as_ever_once_the_aim_left_or_with_no_panel() {
    let (q, _ears) = searching();
    q.away();
    q.pick(0, "hello ", false);
    assert_eq!(q.strokes(), [unicode("hello ")]);
    assert!(q.heard().is_empty());
    q.open(false);
    q.pick(0, "bye ", false);
    assert_eq!(q.strokes(), [unicode("bye ")]);
}

#[test]
fn the_next_relabel_redraws_the_latches_a_word_for_the_line_used() {
    let (q, _ears) = searching();
    q.b.rig.tap("shift").unwrap();
    q.b.rig.state();
    q.pick(0, "Hi ", true);
    let reply = q.b.rig.act(FaceAction::Relabel).unwrap();
    assert!(reply.state);
}
