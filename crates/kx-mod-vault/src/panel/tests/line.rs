//! The line each page shows, which the keyboard aims by (ADR-0040): the search where it is drawn, a form or password line
//! as private, and no line anywhere else.

use kx_module_api::Redacted;
use kx_module_api::ports::PanelLine;

use super::*;
use crate::fixture::{draft, field, own};

/// The search line, holding `text`.
fn search(text: &str) -> PanelLine {
    PanelLine::Search(Redacted::new(text.to_owned()))
}

/// The line `r` shows with no backup password page up.
fn line(r: &Rig) -> PanelLine {
    r.panel.line(false)
}

#[test]
fn a_closed_panel_shows_no_line() {
    let mut r = Rig::sample();
    r.panel.close();
    assert_eq!(line(&r), PanelLine::None);
}

#[test]
fn the_list_and_by_label_show_the_search_with_its_text() {
    let mut r = Rig::sample();
    assert_eq!(line(&r), search(""));
    r.text("ban");
    assert_eq!(line(&r), search("ban"));
    r.click(FillClick::Filter(FillFilter::Labels));
    assert!(matches!(r.page.sheet, Sheet::Labels(_)));
    assert_eq!(line(&r), search("ban"));
}

#[test]
fn an_entry_page_and_a_labels_values_show_no_line() {
    let mut r = Rig::sample();
    r.click(open(1));
    assert!(matches!(r.page.sheet, Sheet::Entry(_)));
    assert_eq!(line(&r), PanelLine::None);
    r.key(&PanelKey::Esc);
    r.click(FillClick::Filter(FillFilter::Labels));
    r.click(open(0));
    assert!(matches!(r.page.sheet, Sheet::Values { .. }));
    assert_eq!(line(&r), PanelLine::None);
}

#[test]
fn letters_on_a_labels_values_do_nothing_as_no_line_shows() {
    let mut r = Rig::sample();
    r.click(FillClick::Filter(FillFilter::Labels));
    r.click(open(0));
    let keys = [
        PanelKey::Text(Redacted::new("z".to_owned())),
        PanelKey::Tick,
        PanelKey::Erase,
    ];
    for key in &keys {
        assert_eq!(r.key(key), Do::Stay);
    }
    assert_eq!(r.page.search.expose(), "");
    assert!(matches!(r.page.sheet, Sheet::Values { .. }));
}

#[test]
fn new_shows_no_line_on_its_kinds_then_a_private_one() {
    let mut r = Rig::sample();
    r.click(button(FillButton::New));
    assert_eq!(line(&r), PanelLine::None, "the kind chips");
    r.click(FillClick::Kind(EntryKind::Note));
    assert_eq!(line(&r), PanelLine::Private);
}

#[test]
fn every_line_of_the_form_is_private_and_its_text_never_shows() {
    let mut r = Rig::sample();
    r.click(open(1));
    r.click(button(FillButton::Edit));
    assert_eq!(line(&r), PanelLine::Private, "the name");
    r.text("kept");
    r.click(FillClick::Field {
        field: 0,
        button: FieldButton::Value,
    });
    assert_eq!(line(&r), PanelLine::Private, "a value");
    r.text("kept");
    assert_eq!(line(&r), PanelLine::Private, "no text travels");
}

#[test]
fn every_caret_of_a_form_is_private_an_own_label_and_a_hidden_value_too() {
    let fields = vec![own("Code", "42", false), field("password", "hunter2", true)];
    let mut r = Rig::new(vec![draft("Locker", EntryKind::Other, fields)]);
    r.click(open(0));
    r.click(button(FillButton::Edit));
    assert_eq!(line(&r), PanelLine::Private, "the name");
    let carets = [
        (0, FieldButton::Label),
        (0, FieldButton::Value),
        (1, FieldButton::Value),
    ];
    for (field, button) in carets {
        assert_eq!(r.click(FillClick::Field { field, button }), Do::Show);
        r.text("kept");
        assert_eq!(line(&r), PanelLine::Private, "{field} {button:?}");
    }
}

#[test]
fn a_search_a_picked_word_left_ending_in_a_space_still_finds_its_row() {
    let mut r = Rig::sample();
    r.text("ban");
    assert_eq!(r.names(), ["Bank"]);
    r.text("k ");
    assert_eq!(r.page.search.expose(), "bank ");
    assert_eq!(r.names(), ["Bank"], "the blank at the end does not count");
    assert_eq!(
        line(&r),
        search("bank "),
        "the line keeps it, so the next word opens"
    );
    for _ in 0..5 {
        r.key(&PanelKey::Erase);
    }
    r.text("Bank ");
    assert_eq!(r.names(), ["Bank"], "a pick that erased first");
}

/// The key of a chip that erases `erase` characters, then types `text`.
fn word(erase: u16, text: &str) -> PanelKey {
    PanelKey::Word {
        erase,
        text: Redacted::new(text.to_owned()),
    }
}

#[test]
fn a_chip_changes_the_search_in_one_key_in_english_and_arabic() {
    for (typed, erase, chip, want) in [
        ("ban", 3, "Bank ", "Bank "),
        ("ban", 0, "k ", "bank "),
        ("مرحب", 2, "حبا ", "مرحبا "),
    ] {
        let mut r = Rig::sample();
        r.text(typed);
        r.key(&PanelKey::Down);
        assert_eq!(r.key(&word(erase, chip)), Do::Show, "{want}");
        assert_eq!(r.page.search.expose(), want);
        assert_eq!(line(&r), search(want));
        let top = r.names().first().map(|_| 0);
        assert_eq!(r.page.highlight, top, "a new search starts at the top");
    }
}

#[test]
fn a_chip_erasing_more_than_the_search_holds_stops_at_empty() {
    let mut r = Rig::sample();
    r.text("ba");
    assert_eq!(r.key(&word(9, "")), Do::Show);
    assert_eq!(r.page.search.expose(), "");
    assert_eq!(r.key(&word(9, "")), Do::Stay, "nothing left to erase");
    assert_eq!(r.key(&word(9, "k")), Do::Show);
    assert_eq!(r.page.search.expose(), "k");
    assert_eq!(r.key(&word(0, "\n")), Do::Stay, "as `Text` refuses it");
}

#[test]
fn a_chip_goes_nowhere_on_a_page_with_no_search() {
    let mut r = Rig::sample();
    r.text("e");
    r.click(FillClick::Filter(FillFilter::Labels));
    r.click(open(0));
    assert!(matches!(r.page.sheet, Sheet::Values { .. }));
    assert_eq!(r.key(&word(1, "z")), Do::Stay, "a label's values");
    assert_eq!(r.page.search.expose(), "e");
    let mut r = Rig::sample();
    r.text("a");
    r.click(open(1));
    assert_eq!(r.key(&word(1, "z")), Do::Stay, "an entry");
    r.click(button(FillButton::Edit));
    assert_eq!(r.key(&word(1, "z")), Do::Stay, "the form's line");
    assert_eq!(line(&r), PanelLine::Private);
    r.key(&PanelKey::Esc);
    r.key(&PanelKey::Esc);
    assert_eq!(r.page.search.expose(), "a");
}

#[test]
fn a_question_or_the_labels_table_over_the_form_shows_no_line() {
    let mut r = Rig::sample();
    r.click(open(1));
    r.click(button(FillButton::Edit));
    r.click(button(FillButton::Delete));
    assert!(matches!(&r.page.sheet, Sheet::Edit(e) if e.confirm));
    assert_eq!(line(&r), PanelLine::None, "the question");
    r.key(&PanelKey::Esc);
    assert_eq!(line(&r), PanelLine::Private, "No is back on the form");
    r.click(button(FillButton::AddField));
    assert!(matches!(r.page.sheet, Sheet::AddField(_)));
    assert_eq!(line(&r), PanelLine::None, "the labels table");
    r.key(&PanelKey::Esc);
    assert_eq!(line(&r), PanelLine::Private);
}

#[test]
fn the_backup_pages_show_a_private_line_only_on_a_password() {
    let mut r = Rig::sample();
    r.click(button(FillButton::Backup));
    assert_eq!(r.panel.line(false), PanelLine::None);
    assert_eq!(r.panel.line(true), PanelLine::Private);
}

#[test]
fn a_password_up_changes_no_other_page() {
    let r = Rig::sample();
    assert_eq!(r.panel.line(true), search(""));
}

#[test]
fn the_debug_of_a_search_shows_no_text() {
    let mut r = Rig::sample();
    r.text("ban");
    let shown = format!("{:?}", line(&r));
    assert!(!shown.contains("ban"), "{shown}");
    assert!(shown.contains(REDACTED_MARKER), "{shown}");
}
