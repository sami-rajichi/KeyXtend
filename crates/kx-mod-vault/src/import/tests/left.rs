//! What an import leaves out: duplicates against the notebook and within the file, values the marks and breaks rules
//! refuse, the entry and size caps, and the worker's own drops, each counted in the note.

use super::*;
use crate::fixture::{draft, field};
use kx_module_api::ports::Dropped;

/// A notebook holding `drafts`.
fn saved(drafts: Vec<Draft>) -> Notebook {
    let (mut book, random) = (Notebook::default(), FakeRandom::new(3));
    for d in drafts {
        book.add(d, NOW, &random, &LIMITS).unwrap();
    }
    book
}

#[test]
fn an_entry_saved_already_is_skipped_as_search_folds_it() {
    let mut book = saved(vec![draft(
        "Crédit Bank",
        EntryKind::Login,
        vec![field("user", "Sam", false)],
    )]);
    let done = into_book(
        &mut book,
        vec![
            entry("credit  BANK ", "SAM", "new"),
            entry("Credit Bank", "alex", "new"),
            entry("Other Bank", "sam", "new"),
            entry("Credit Bank", "sam", ""),
        ],
        &LIMITS,
    );
    assert_eq!(done, counts(3, 1));
    assert_eq!(book.len(), 4, "another user, name or kind comes in");
}

#[test]
fn a_saved_entry_of_another_kind_is_no_duplicate() {
    let mut book = saved(vec![draft(
        "Gate",
        EntryKind::Note,
        vec![field("text", "4321", false)],
    )]);
    let done = into_book(&mut book, vec![entry("Gate", "", "1234")], &LIMITS);
    assert_eq!(done, counts(1, 0));
}

#[test]
fn an_entry_earlier_in_the_file_is_a_duplicate_too() {
    let (book, done) = fresh(vec![
        entry("بنك", "أليكس", "one"),
        entry("Other", "", "two"),
        entry("بنك", "أليكس", "three"),
        entry("OTHER", "", "four"),
    ]);
    assert_eq!(done, counts(2, 2));
    assert_eq!(
        value(&book.entries()[0], "password"),
        Some("one"),
        "first wins"
    );
}

#[test]
fn nameless_entries_are_never_duplicates() {
    let mut book = saved(vec![draft(
        "",
        EntryKind::Login,
        vec![field("user", "sam", false), field("password", "old", true)],
    )]);
    let entries = vec![
        entry("", "sam", "one"),
        entry("", "sam", "two"),
        entry("  ", "sam", "three"),
        entry("Ba\u{202e}nk", "sam", "four"),
        entry("Ba\u{202e}nk", "sam", "five"),
    ];
    let done = into_book(&mut book, entries, &LIMITS);
    assert_eq!(done, counts(5, 2), "only the two refused titles count");
    assert_eq!(book.len(), 6);
}

#[test]
fn an_arabic_custom_value_comes_in_hidden_without_its_direction_marks() {
    let (book, done) = fresh(vec![KeepassEntry {
        custom: vec![custom(
            "\u{200f}كود",
            "\u{200f}می\u{200c}خواهم\u{200e}٤٢\u{61c}",
        )],
        ..entry("Bank", "sam", VALUE)
    }]);
    assert_eq!(done, counts(1, 0), "no value is lost");
    let bank = &book.entries()[0];
    let want = [("كود".to_owned(), "می\u{200c}خواهم٤٢".to_owned())];
    assert_eq!(owns(bank), want, "ZWNJ stays, as words need it");
    assert!(bank.fields().last().unwrap().hidden);
}

#[test]
fn a_custom_value_with_another_hidden_mark_is_refused_and_counted() {
    let (book, done) = fresh(vec![KeepassEntry {
        custom: vec![custom("Code", "٤٢\u{202e}"), custom("Pin", "\u{200f}٤٢")],
        ..entry("Bank", "sam", VALUE)
    }]);
    assert_eq!(done, counts(1, 1));
    assert_eq!(
        owns(&book.entries()[0]),
        [("Pin".to_owned(), "٤٢".to_owned())]
    );
}

#[test]
fn a_password_with_a_direction_mark_is_refused_and_counted_as_it_must_never_change() {
    let (book, done) = fresh(vec![entry("Bank", "sam", "\u{200f}pass\u{200e}word")]);
    assert_eq!(done, counts(1, 1));
    let bank = &book.entries()[0];
    assert_eq!(value(bank, "password"), None);
    assert_eq!(bank.kind(), EntryKind::Other);
}

#[test]
fn a_line_break_outside_the_notes_is_refused_as_the_rules_say() {
    let (book, done) = fresh(vec![KeepassEntry {
        custom: vec![custom("Codes", "one\ntwo")],
        ..entry("Bank", "sam", "pass\nword")
    }]);
    assert_eq!(done, counts(1, 2));
    assert_eq!(book.entries()[0].fields().len(), 1);
}

#[test]
fn the_entry_cap_holds_and_the_rest_are_counted() {
    let limits = Limits {
        max_entries: 2,
        ..LIMITS
    };
    let mut book = saved(vec![draft(
        "Gate",
        EntryKind::Note,
        vec![field("text", "4321", false)],
    )]);
    let entries = vec![
        entry("A", "", "1"),
        entry("B", "", "2"),
        entry("C", "", "3"),
    ];
    assert_eq!(into_book(&mut book, entries, &limits), counts(1, 2));
    assert_eq!(book.len(), 2);
}

#[test]
fn past_the_saved_size_an_entry_stays_out_and_a_smaller_one_still_fits() {
    let probe = saved(vec![draft(
        "A",
        EntryKind::Login,
        vec![field("password", "1", true)],
    )]);
    let limits = Limits {
        max_bytes: probe.bytes() + 1,
        ..LIMITS
    };
    let mut book = Notebook::default();
    let long = "x".repeat(99);
    let entries = vec![entry(&long, "", &long), entry("A", "", "1")];
    assert_eq!(into_book(&mut book, entries, &limits), counts(1, 1));
    assert!(book.bytes() <= limits.max_bytes);
}

#[test]
fn the_workers_own_drop_counts_add_to_the_note() {
    let import = KeepassImport {
        entries: vec![entry("Bank", "sam", VALUE), entry("Bank", "sam", "again")],
        dropped: Dropped {
            entries: 4,
            fields: 3,
        },
    };
    let mut book = Notebook::default();
    let done = bring_in(&mut book, import, NOW, &FakeRandom::new(5), &LIMITS);
    assert_eq!(done, counts(1, 8));
}

#[test]
fn the_counts_never_wrap() {
    let import = KeepassImport {
        entries: vec![entry("", "", "")],
        dropped: Dropped {
            entries: u32::MAX,
            fields: u32::MAX,
        },
    };
    let mut book = Notebook::default();
    let done = bring_in(&mut book, import, NOW, &FakeRandom::new(5), &LIMITS);
    assert_eq!(done, counts(0, u32::MAX));
}
