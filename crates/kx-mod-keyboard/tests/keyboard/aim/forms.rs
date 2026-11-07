//! A form or password line (ADR-0040 decision 2): it aims as it appears, and leaving puts the aim back as it was, unless
//! a later rule took it away.

use super::{ELSEWHERE, LETTER, Quick};
use crate::clip::{self, Clip};
use kx_module_api::Redacted;
use kx_module_api::ports::{ClipLine, FillLine, PanelLine, PointerPressed};

impl Quick {
    /// The vault module says which line shows.
    pub(super) fn line(&self, line: PanelLine) {
        self.b.send(&FillLine(line));
    }
}

/// A search holding `t`.
pub(super) fn search(t: &str) -> PanelLine {
    PanelLine::Search(Redacted::new(t.to_owned()))
}

#[test]
fn a_form_aims_and_leaving_it_puts_an_aim_that_was_off_back_off() {
    let q = Quick::opened(None);
    q.away();
    q.line(PanelLine::Private);
    assert!(q.letter_to_panel(), "the form aims");
    q.line(search(""));
    assert!(!q.letter_to_panel(), "off, as it was");
}

#[test]
fn leaving_a_form_keeps_an_aim_that_was_on() {
    let q = Quick::opened(None);
    q.line(PanelLine::Private);
    q.line(PanelLine::None);
    assert!(q.letter_to_panel());
}

#[test]
fn an_outside_click_during_a_form_stays_off_after_it() {
    let q = Quick::opened(None);
    q.line(PanelLine::Private);
    q.away();
    assert!(!q.letter_to_panel(), "the form lost it");
    q.line(search(""));
    assert!(!q.letter_to_panel(), "and leaving brings nothing back");
}

#[test]
fn a_window_taking_the_front_during_a_form_gets_none_of_its_letters() {
    let q = Quick::opened(None);
    q.line(PanelLine::Private);
    q.window(Some(ELSEWHERE));
    assert!(q.letter_to_panel(), "a secret's letters stay in the form");
    q.away();
    assert!(!q.letter_to_panel(), "only a click moves them");
}

#[test]
fn a_new_front_window_during_a_form_stays_off_after_it() {
    let q = Quick::opened(None);
    q.line(PanelLine::Private);
    q.window(Some(ELSEWHERE));
    q.line(search(""));
    assert!(!q.letter_to_panel());
}

#[test]
fn a_click_on_the_forms_line_aims_it_again() {
    let q = Quick::opened(None);
    q.line(PanelLine::Private);
    q.away();
    q.aim();
    assert!(q.letter_to_panel());
    q.line(search(""));
    assert!(q.letter_to_panel(), "it was on before the form");
}

#[test]
fn a_repeated_private_word_keeps_what_was_saved() {
    let q = Quick::opened(None);
    q.away();
    q.line(PanelLine::Private);
    q.line(PanelLine::Private);
    q.line(PanelLine::None);
    assert!(!q.letter_to_panel(), "the first word saved it off");
}

#[test]
fn a_search_changing_its_text_changes_nothing() {
    let q = Quick::opened(None);
    q.line(search("a"));
    q.line(search("ab"));
    assert!(q.letter_to_panel());
    q.away();
    q.line(search("abc"));
    assert!(!q.letter_to_panel());
}

#[test]
fn a_line_word_from_the_panel_not_open_changes_nothing() {
    let q = Quick::opened(None);
    q.away();
    q.b.send(&ClipLine(PanelLine::Private));
    assert!(!q.letter_to_panel());
    q.aim();
    q.b.send(&ClipLine(PanelLine::Private));
    q.b.send(&ClipLine(PanelLine::None));
    assert!(q.letter_to_panel());
}

#[test]
fn a_line_word_with_no_panel_open_changes_nothing() {
    let q = Quick::quiet(None);
    q.line(PanelLine::Private);
    assert!(!q.aimed());
    assert!(!q.letter_to_panel());
    q.open(true);
    q.heard();
    q.line(PanelLine::None);
    assert!(q.aimed(), "the earlier word saved nothing");
}

#[test]
fn a_close_forgets_the_form() {
    let q = Quick::opened(None);
    q.away();
    q.line(PanelLine::Private);
    q.open(false);
    q.line(PanelLine::None);
    q.open(true);
    q.heard();
    q.line(search(""));
    assert!(q.letter_to_panel(), "opening aimed, and no form restores");
}

#[test]
fn the_clipboards_line_words_are_heard_too() {
    let c = Clip::quiet(None);
    c.open(true);
    c.b.send(&PointerPressed { ours: false });
    c.b.send(&ClipLine(PanelLine::Private));
    c.b.rig.tap(LETTER).unwrap();
    assert_eq!(c.heard(), [clip::text("q")]);
    c.b.send(&ClipLine(PanelLine::None));
    c.b.rig.tap(LETTER).unwrap();
    assert!(c.heard().is_empty());
    assert_eq!(c.strokes().len(), 1);
}
