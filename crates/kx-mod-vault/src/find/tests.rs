//! Search: the query folds as the clipboard's does, matches a name, a label or a shown value on its own, never a
//! hidden value, finds a table label only by the text the ui shows, drops hidden marks, and copes with 2,000 entries.

use super::*;
use crate::book::Notebook;
use crate::entry::Stamp;
use crate::fixture::{LIMITS, draft, field, own};
use kx_module_api::ports::EntryKind;
use kx_platform_fake::FakeRandom;

/// The time the tests save at.
const NOW: Stamp = Stamp::from_unix_ms(1_762_000_000_000);

/// The entry a draft of `kind` named `name` with `fields` saves as.
fn saved(name: &str, kind: EntryKind, fields: Vec<crate::entry::Field>) -> Entry {
    let mut book = Notebook::default();
    let id = book
        .add(draft(name, kind, fields), NOW, &FakeRandom::new(1), &LIMITS)
        .unwrap();
    book.get(id).unwrap().clone()
}

/// True when `find` keeps `entry`.
fn keeps(find: &Find, entry: &Entry) -> bool {
    !find.list([entry]).is_empty()
}

/// True when `query`, with no label texts from the ui, finds an Other entry named `name` holding a shown `value`.
fn finds(query: &str, name: &str, value: &str) -> bool {
    let entry = saved(name, EntryKind::Other, vec![field("text", value, false)]);
    keeps(&Find::new(query, &LabelNames::new()), &entry)
}

#[test]
fn case_and_accents_fold_both_ways() {
    assert!(finds("ECOLE", "École Jean Moulin", "x"));
    assert!(finds("école", "x", "ECOLE"));
    assert!(!finds("ecoles", "école", "x"));
}

#[test]
fn arabic_folds_its_alef_forms_harakat_tatweel_and_alef_maksura() {
    assert!(finds("احمد", "أحمد", "x"));
    assert!(finds("أحمد", "x", "احمد"));
    assert!(finds("احمد", "أَحْمَد", "x"));
    assert!(finds("احمد", "أحــمد", "x"));
    assert!(finds("مصطفي", "مصطفى", "x"));
    assert!(!finds("محمود", "أحمد", "x"));
}

#[test]
fn a_joiner_in_the_query_matches_the_same_joiner() {
    assert!(finds("عبد\u{200c}الله", "عبد\u{200c}الله", "x"));
    assert!(
        finds("\u{200f}عبد", "عبد\u{200c}الله", "x"),
        "a stray RLM in the query is dropped"
    );
}

#[test]
fn a_hidden_value_is_never_found() {
    let entry = saved(
        "Bank",
        EntryKind::Login,
        vec![field("password", "zebra-quartz", true)],
    );
    assert!(!keeps(&Find::new("zebra", &LabelNames::new()), &entry));
    let shown = saved(
        "Bank",
        EntryKind::Login,
        vec![field("user", "zebra-quartz", false)],
    );
    assert!(keeps(&Find::new("zebra", &LabelNames::new()), &shown));
}

#[test]
fn a_table_label_is_found_by_its_shown_text_and_never_by_its_id() {
    let entry = saved(
        "Alex",
        EntryKind::Person,
        vec![field("email", "a@b.c", true)],
    );
    let none = LabelNames::new();
    assert!(
        !keeps(&Find::new("EMAIL", &none), &entry),
        "with no shown texts a table label matches nothing"
    );
    let arabic = LabelNames::from([("email".to_owned(), "البريد الإلكتروني".to_owned())]);
    assert!(keeps(&Find::new("البريد", &arabic), &entry));
    assert!(
        !keeps(&Find::new("email", &arabic), &entry),
        "the shown text stands in for the id"
    );
}

#[test]
fn an_own_label_is_found_folded() {
    let entry = saved(
        "Bank",
        EntryKind::Other,
        vec![own("Clé de sécurité", "1234", true)],
    );
    assert!(keeps(
        &Find::new("cle de securite", &LabelNames::new()),
        &entry
    ));
}

#[test]
fn a_match_never_spans_two_texts() {
    assert!(!finds("bank alex", "Bank", "alex"));
    assert!(finds("bank", "Bank", "alex"));
}

#[test]
fn a_blank_query_finds_everything_and_line_breaks_read_as_spaces() {
    assert!(finds("", "Bank", "x"));
    assert!(finds("  \n ", "Bank", "x"));
    assert!(finds("credit\r\nagricole", "x", "Crédit Agricole"));
    assert!(
        finds("bank\u{2028}login", "x", "bank login"),
        "a line separator"
    );
    assert!(
        finds("bank\u{2029}login", "x", "bank login"),
        "a paragraph separator"
    );
    assert!(!finds("bank\u{2028}login", "x", "banklogin"));
}

#[test]
fn two_thousand_entries_are_searched_whole() {
    let (mut book, random) = (Notebook::default(), FakeRandom::new(7));
    for n in 0..LIMITS.max_entries {
        let value = format!("user{n}@example.com");
        let d = draft(
            &format!("Site {n}"),
            EntryKind::Login,
            vec![field("user", &value, false)],
        );
        book.add(d, NOW, &random, &LIMITS).unwrap();
    }
    let find = Find::new("user1999@", &LabelNames::new());
    assert_eq!(find.list(book.entries()).len(), 1);
    assert_eq!(
        Find::new("site", &LabelNames::new())
            .list(book.entries())
            .len(),
        LIMITS.max_entries
    );
}

#[test]
fn a_hidden_mark_in_the_query_is_dropped_as_no_text_holds_one() {
    assert!(finds("pass\u{ad}word", "x", "password"), "a soft hyphen");
    assert!(finds("credit\u{200b} agricole", "x", "Crédit Agricole"));
    assert!(finds("\u{202e}bank", "Bank", "x"));
}
