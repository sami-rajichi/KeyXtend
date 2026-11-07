//! The panel's search line: what its text takes and keeps, and a picked word's space (ADR-0040).

use super::*;

#[test]
fn the_search_types_erases_and_brings_the_highlight_to_the_top() {
    let h = history(&["a b", "a bc", "z"]);
    let mut p = opened();
    p.key(&PanelKey::Down, &h);
    assert_eq!(p.key(&chars("a\u{7}\r\nb"), &h), Do::Show);
    assert_eq!(p.query().expose(), "a b", "one line");
    assert_eq!(p.key(&PanelKey::Enter, &h), Do::Insert(ids(&h, &["a bc"])));
    assert_eq!(p.key(&PanelKey::Erase, &h), Do::Show);
    assert_eq!(p.query().expose(), "a ");
    p.key(&PanelKey::Erase, &h);
    p.key(&PanelKey::Erase, &h);
    assert_eq!(p.key(&PanelKey::Erase, &h), Do::Stay);
    assert_eq!(p.key(&chars(""), &h), Do::Stay);
}

#[test]
fn the_search_keeps_at_most_query_chars_characters() {
    let h = history(&["a"]);
    let mut p = opened_with(PanelConfig {
        query_chars: 3,
        ..*PANEL
    });
    assert_eq!(p.key(&chars("éàüö"), &h), Do::Show);
    assert_eq!(p.query().expose(), "éàü");
    assert_eq!(p.key(&chars("e"), &h), Do::Stay);
}

#[test]
fn what_is_typed_joins_the_search_on_one_line() {
    let h = history(&["a b"]);
    let mut p = opened_with(PanelConfig {
        query_chars: 3,
        ..*PANEL
    });
    p.key(&chars("a "), &h);
    assert_eq!(p.key(&chars(" b"), &h), Do::Show);
    assert_eq!(p.query().expose(), "a b", "one space, three characters");
    assert_eq!(p.key(&chars("\n"), &h), Do::Stay, "full");
}

/// The key of a chip that erases `erase` characters, then types `text`.
fn word(erase: u16, text: &str) -> PanelKey {
    PanelKey::Word {
        erase,
        text: Redacted::new(text.to_owned()),
    }
}

#[test]
fn a_chip_changes_the_search_once_in_english_and_arabic() {
    let h = history(&["a", "b"]);
    for (typed, erase, chip, want) in [
        ("hel", 3, "Hello ", "Hello "),
        ("hel", 0, "lo ", "hello "),
        ("مرحب", 2, "حبا ", "مرحبا "),
    ] {
        let mut p = opened();
        p.key(&chars(typed), &h);
        assert_eq!(p.key(&word(erase, chip), &h), Do::Show, "{want}");
        assert_eq!(p.query().expose(), want);
        assert_eq!(p.line(), PanelLine::Search(Redacted::new(want.to_owned())));
    }
}

#[test]
fn a_chip_erasing_more_than_the_search_holds_stops_at_empty() {
    let h = history(&["a"]);
    let mut p = opened();
    p.key(&chars("ab"), &h);
    assert_eq!(p.key(&word(9, ""), &h), Do::Show);
    assert_eq!(p.query().expose(), "");
    assert_eq!(p.key(&word(9, ""), &h), Do::Stay, "nothing left");
    assert_eq!(p.key(&word(9, "c"), &h), Do::Show);
    assert_eq!(p.query().expose(), "c");
}

#[test]
fn a_chip_keeps_the_rules_of_the_query_as_text_does() {
    let h = history(&["a"]);
    let mut p = opened_with(PanelConfig {
        query_chars: 3,
        ..*PANEL
    });
    p.key(&chars("ab"), &h);
    assert_eq!(p.key(&word(1, "é\u{7}\r\nàü"), &h), Do::Show);
    assert_eq!(p.query().expose(), "aé ", "flat, and cut at the cap");
    assert_eq!(p.key(&word(1, " "), &h), Do::Stay, "the same query");
    assert_eq!(p.key(&word(0, "z"), &h), Do::Stay, "full");
}

#[test]
fn a_chip_goes_nowhere_on_a_preview_or_under_a_question() {
    let h = history(&["a"]);
    let mut p = opened();
    p.key(&chars("a"), &h);
    p.act(ClipAct::DeleteAll, &h);
    assert_eq!(p.key(&word(1, "z"), &h), Do::Stay, "a question");
    p.act(ClipAct::Confirm(false), &h);
    p.show(id(&h, "a"), content());
    assert!(p.preview().is_some());
    assert_eq!(p.key(&word(1, "z"), &h), Do::Stay, "a preview");
    assert_eq!(p.query().expose(), "a");
}

#[test]
fn a_picked_word_brings_its_space_in_and_the_search_still_finds_its_row() {
    let h = history(&["alpha", "zeta"]);
    let mut p = opened();
    p.key(&chars("alp"), &h);
    assert_eq!(p.key(&chars("ha "), &h), Do::Show);
    assert_eq!(
        p.query().expose(),
        "alpha ",
        "only a text brings a space in"
    );
    assert_eq!(
        p.line(),
        PanelLine::Search(Redacted::new("alpha ".to_owned()))
    );
    assert_eq!(p.key(&PanelKey::Enter, &h), Do::Insert(ids(&h, &["alpha"])));
    let mut p = opened();
    p.key(&chars("alpha z"), &h);
    assert_eq!(p.key(&PanelKey::Enter, &h), Do::Stay, "two words, no row");
}
