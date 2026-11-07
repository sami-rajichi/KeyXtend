//! The import's mapping: Login and Other with the right fields, notes on one line, custom strings under their own
//! names, and the rules per value, each refusal counted; `left` tests the duplicates and the caps.

use super::*;
use crate::fixture::{LIMITS, secret};
use kx_module_api::REDACTED_MARKER;
use kx_module_api::ports::KeepassField;
use kx_platform_fake::FakeRandom;

mod left;

/// When the import runs.
const NOW: Stamp = Stamp::from_unix_ms(1_700_000_000_000);
/// A value no `Debug` may show.
const VALUE: &str = "zebra-quartz";

/// A `KeePass` entry with only a title, a user and a password.
fn entry(title: &str, user: &str, password: &str) -> KeepassEntry {
    KeepassEntry {
        title: secret(title),
        user: secret(user),
        password: secret(password),
        address: secret(""),
        notes: secret(""),
        custom: Vec::new(),
    }
}

/// A custom string.
fn custom(label: &str, value: &str) -> KeepassField {
    KeepassField {
        label: Redacted::new(label.to_owned()),
        value: secret(value),
    }
}

/// `limits` of the default settings under another `field_chars`.
fn chars(field_chars: usize) -> Limits {
    Limits {
        field_chars,
        ..LIMITS
    }
}

/// Brings `entries` into `book` under `limits`.
fn into_book(book: &mut Notebook, entries: Vec<KeepassEntry>, limits: &Limits) -> NoteCounts {
    bring_in(book, entries.into(), NOW, &FakeRandom::new(5), limits)
}

/// Brings `entries` into an empty notebook; gives it and the counts.
fn fresh(entries: Vec<KeepassEntry>) -> (Notebook, NoteCounts) {
    let mut book = Notebook::default();
    let counts = into_book(&mut book, entries, &LIMITS);
    (book, counts)
}

/// `added` and `skipped` as counts.
fn counts(added: u32, skipped: u32) -> NoteCounts {
    NoteCounts { added, skipped }
}

/// The value of `entry`'s field under table label `id`.
fn value<'a>(entry: &'a Entry, id: &str) -> Option<&'a str> {
    let under = |f: &&Field| matches!(&f.label, Label::Table(t) if t == id);
    let field = entry.fields().iter().find(under)?;
    Some(field.value.expose().as_str())
}

/// `entry`'s own labels and values, in order.
fn owns(entry: &Entry) -> Vec<(String, String)> {
    let own = |f: &Field| match &f.label {
        Label::Own(label) => Some((label.expose().clone(), f.value.expose().to_string())),
        Label::Table(_) => None,
    };
    entry.fields().iter().filter_map(own).collect()
}

#[test]
fn an_entry_with_a_password_becomes_a_login_with_the_right_fields() {
    let (book, done) = fresh(vec![KeepassEntry {
        address: secret("https://example.com"),
        notes: secret("the blue one"),
        ..entry("Bank", "sam", VALUE)
    }]);
    assert_eq!(done, counts(1, 0));
    let bank = &book.entries()[0];
    assert_eq!(
        (bank.name().expose().as_str(), bank.kind()),
        ("Bank", EntryKind::Login)
    );
    assert_eq!(value(bank, "user"), Some("sam"));
    assert_eq!(value(bank, "password"), Some(VALUE));
    assert_eq!(value(bank, "website"), Some("https://example.com"));
    assert_eq!(value(bank, "text"), Some("the blue one"));
    let hidden: Vec<_> = bank.fields().iter().map(|f| f.hidden).collect();
    assert_eq!(hidden, [false, true, false, false], "as its labels start");
    assert_eq!((bank.changed(), bank.used()), (NOW, None));
}

#[test]
fn an_entry_without_a_password_becomes_other() {
    let (book, done) = fresh(vec![entry("Gate", "sam", "")]);
    assert_eq!(done, counts(1, 0));
    assert_eq!(book.entries()[0].kind(), EntryKind::Other);
    assert_eq!(value(&book.entries()[0], "user"), Some("sam"));
}

#[test]
fn notes_lose_their_breaks_as_paste_turns_them() {
    let notes = KeepassEntry {
        notes: secret("\nfirst\r\nsecond\nthird\u{2028}fourth\r"),
        ..entry("Gate", "", "")
    };
    let only_breaks = KeepassEntry {
        notes: secret("\r\n\n"),
        ..entry("Shed", "sam", "")
    };
    let (book, done) = fresh(vec![notes, only_breaks]);
    assert_eq!(done, counts(2, 0));
    assert_eq!(
        value(&book.entries()[0], "text"),
        Some("first second third fourth")
    );
    assert_eq!(
        value(&book.entries()[1], "text"),
        None,
        "nothing but breaks is empty"
    );
}

#[test]
fn custom_strings_become_fields_under_their_own_names() {
    let (book, done) = fresh(vec![KeepassEntry {
        custom: vec![custom("Email", "a@b.c"), custom("كود", "٤٢")],
        ..entry("Bank", "sam", VALUE)
    }]);
    assert_eq!(done, counts(1, 0));
    let bank = &book.entries()[0];
    let want = [
        ("Email".to_owned(), "a@b.c".to_owned()),
        ("كود".to_owned(), "٤٢".to_owned()),
    ];
    assert_eq!(owns(bank), want, "own labels, after the standard fields");
    assert!(bank.fields()[2..].iter().all(|f| f.hidden));
    assert!(book.emails().is_empty(), "no table label from a name");
}

#[test]
fn every_value_goes_through_the_marks_rule() {
    let (book, done) = fresh(vec![
        KeepassEntry {
            custom: vec![custom("كود", "\u{200f}٤٢")],
            ..entry("\u{200f}بنك\u{200e}", "\u{200f}أليكس", VALUE)
        },
        entry("Shed", "sam", "pass\u{200f}word"),
        entry("Barn", "sam\u{202e}", "ok"),
    ]);
    assert_eq!(done, counts(3, 2));
    let [bank, shed, barn] = book.entries() else {
        panic!("three entries");
    };
    assert_eq!(
        bank.name().expose().as_str(),
        "بنك",
        "a shown text loses it"
    );
    assert_eq!(value(bank, "user"), Some("أليكس"));
    assert_eq!(
        owns(bank),
        [("كود".to_owned(), "٤٢".to_owned())],
        "and a custom one"
    );
    assert_eq!(value(shed, "password"), None, "a password is refused");
    assert_eq!(shed.kind(), EntryKind::Other, "and it has no password now");
    assert_eq!(value(barn, "user"), None, "other marks are refused");
    assert_eq!(barn.kind(), EntryKind::Login);
}

#[test]
fn a_value_over_field_chars_is_dropped_and_counted() {
    let long = "x".repeat(9);
    let mut book = Notebook::default();
    let done = into_book(
        &mut book,
        vec![KeepassEntry {
            notes: secret(&long),
            custom: vec![custom("fits", &long[..8]), custom(&long, "short")],
            ..entry("Bank", "sam", &long)
        }],
        &chars(8),
    );
    assert_eq!(done, counts(1, 3), "the password, the notes and one label");
    let bank = &book.entries()[0];
    assert_eq!(bank.kind(), EntryKind::Other);
    assert_eq!(
        (value(bank, "user"), owns(bank).len()),
        (Some("sam"), 1),
        "the rest stay"
    );
}

#[test]
fn a_blank_label_is_refused_and_an_empty_custom_string_is_not_counted() {
    let (book, done) = fresh(vec![KeepassEntry {
        custom: vec![custom("  ", "v"), custom("", "v"), custom("Empty", "")],
        ..entry("Bank", "sam", "")
    }]);
    assert_eq!(done, counts(1, 2));
    assert_eq!(book.entries()[0].fields().len(), 1);
}

#[test]
fn fields_past_max_fields_are_dropped_and_counted() {
    let limits = Limits {
        max_fields: 3,
        ..LIMITS
    };
    let mut book = Notebook::default();
    let done = into_book(
        &mut book,
        vec![KeepassEntry {
            custom: vec![custom("a", "1"), custom("b", "2"), custom("c", "3")],
            ..entry("Bank", "sam", VALUE)
        }],
        &limits,
    );
    assert_eq!(done, counts(1, 2));
    let bank = &book.entries()[0];
    assert_eq!(bank.fields().len(), 3);
    assert_eq!(owns(bank), [("a".to_owned(), "1".to_owned())]);
    assert_eq!(
        bank.kind(),
        EntryKind::Login,
        "the standard fields go first"
    );
}

#[test]
fn an_entry_left_with_no_field_is_skipped_and_counted() {
    let (book, done) = fresh(vec![
        entry("Only a title", "", ""),
        entry("", "", ""),
        entry("Refused", "", "pass\nword"),
        entry("Bank", "sam", VALUE),
    ]);
    assert_eq!(
        done,
        counts(1, 4),
        "three empty entries and the refused password"
    );
    assert_eq!(book.len(), 1);
}

#[test]
fn a_title_the_rules_refuse_leaves_the_entry_unnamed_and_counted() {
    let (book, done) = fresh(vec![entry("Ba\u{202e}nk", "sam", VALUE)]);
    assert_eq!(done, counts(1, 1));
    let bank = &book.entries()[0];
    assert!(bank.name().expose().is_empty());
    assert_eq!(
        value(bank, "password"),
        Some(VALUE),
        "its values still come"
    );
}

#[test]
fn a_keepass_import_never_shows_a_value_in_debug() {
    let import: KeepassImport = vec![KeepassEntry {
        address: secret(VALUE),
        notes: secret(VALUE),
        custom: vec![custom(VALUE, VALUE)],
        ..entry(VALUE, VALUE, VALUE)
    }]
    .into();
    let shown = format!("{import:?}");
    assert!(!shown.contains(VALUE) && shown.contains(REDACTED_MARKER));
    let (draft, _) = mapped(import.entries[0].clone(), &LIMITS);
    let shown = format!("{draft:?}");
    assert!(!shown.contains(VALUE) && shown.contains(REDACTED_MARKER));
}

#[test]
fn the_table_names_the_extension_and_known_labels() {
    let sheet = sheet();
    assert!(!sheet.ext.is_empty() && !sheet.ext.contains('.'));
    let labels = [&sheet.user, &sheet.password, &sheet.website, &sheet.text];
    assert!(labels.iter().all(|id| tables().label(id).is_some()));
    assert!(tables().label(&sheet.password).is_some_and(|l| l.hidden));
    assert_eq!(keepass_ext(), sheet.ext);
}
