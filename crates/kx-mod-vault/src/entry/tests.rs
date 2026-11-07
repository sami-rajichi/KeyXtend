//! The entry rules: starter fields, empty fields, the limits with their notes, one-line values, the hidden marks,
//! Arabic with its joiners, and `Debug` that never shows what the user wrote.

use super::*;
use crate::fixture::{LIMITS, draft, field, own, secret};
use kx_module_api::REDACTED_MARKER;

/// The time the tests save at.
const NOW: Stamp = Stamp::from_unix_ms(1_700_000_000_000);

/// An entry of `kind` named `name` with `fields`, checked against `limits`.
fn entry_of(fields: Vec<Field>, limits: &Limits) -> Result<Entry, EntryError> {
    Entry::new(1, draft("Bank", EntryKind::Other, fields), NOW, limits)
}

/// The one kept value of a field holding `text`, shown or hidden.
fn kept(text: &str, hidden: bool) -> Result<String, EntryError> {
    let entry = entry_of(vec![field("text", text, hidden)], &LIMITS)?;
    Ok(entry.fields()[0].value.expose().to_string())
}

#[test]
fn each_kind_starts_with_its_fields_and_their_hidden_flags() {
    let starts = |kind| {
        let d = Draft::new(kind);
        let shown: Vec<(Label, bool)> = d
            .fields
            .iter()
            .map(|f| (f.label.clone(), f.hidden))
            .collect();
        assert!(
            d.fields.iter().all(|f| f.value.expose().is_empty()),
            "{kind:?}"
        );
        assert_eq!(d.kind, kind);
        shown
    };
    let table = |id: &str, hidden| (Label::Table(id.to_owned()), hidden);
    assert_eq!(
        starts(EntryKind::Login),
        [
            table("user", false),
            table("password", true),
            table("website", false)
        ]
    );
    assert_eq!(
        starts(EntryKind::Wifi),
        [table("network", false), table("password", true)]
    );
    assert_eq!(
        starts(EntryKind::Card),
        [table("number", true), table("full-name", false)]
    );
    assert_eq!(starts(EntryKind::Person).len(), 4);
    assert!(starts(EntryKind::Person).iter().all(|(_, hidden)| !hidden));
    assert_eq!(starts(EntryKind::Note), [table("text", false)]);
    assert!(starts(EntryKind::Other).is_empty());
}

#[test]
fn empty_fields_are_not_kept() {
    let mut login = Draft::new(EntryKind::Login);
    login.fields[0].value = secret("alex");
    let entry = Entry::new(4, login, NOW, &LIMITS).unwrap();
    assert_eq!(entry.fields().len(), 1);
    assert_eq!(entry.fields()[0].label, Label::Table("user".to_owned()));
    let marks_only = entry_of(vec![field("text", "\u{200e}\u{200f}", false)], &LIMITS).unwrap();
    assert!(
        marks_only.fields().is_empty(),
        "a value of stripped marks is empty"
    );
}

#[test]
fn a_value_holds_at_most_field_chars_characters_not_bytes() {
    let limits = Limits {
        field_chars: 4,
        ..LIMITS
    };
    assert!(entry_of(vec![field("text", "سلام", true)], &limits).is_ok());
    let long = entry_of(vec![field("text", "سلامٌ", true)], &limits);
    assert_eq!(long.err(), Some(EntryError::Refused));
    assert_eq!(EntryError::Refused.note(), Some(FillNote::Refused));
}

#[test]
fn an_entry_holds_at_most_max_fields_kept_fields() {
    let limits = Limits {
        max_fields: 2,
        ..LIMITS
    };
    let full = vec![
        field("user", "a", false),
        field("pin", "1", true),
        field("text", "", false),
    ];
    assert_eq!(entry_of(full, &limits).map(|e| e.fields().len()), Ok(2));
    let over = vec![
        field("user", "a", false),
        field("pin", "1", true),
        field("text", "b", false),
    ];
    assert_eq!(entry_of(over, &limits).err(), Some(EntryError::Full));
    assert_eq!(EntryError::Full.note(), Some(FillNote::Full));
}

#[test]
fn a_value_with_a_line_break_is_refused_shown_or_hidden() {
    for text in [
        "a\nb",
        "a\r\nb",
        "a\rb",
        "a\u{85}b",
        "a\u{2028}b",
        "a\u{2029}b",
        "a\tb",
    ] {
        for hidden in [false, true] {
            assert_eq!(kept(text, hidden), Err(EntryError::Refused), "{text:?}");
        }
    }
}

#[test]
fn a_shown_value_loses_the_bidi_marks_and_a_hidden_one_is_refused_with_them() {
    let marked = "\u{200f}+33 6 12\u{200e} 34\u{61c}";
    assert_eq!(kept(marked, false), Ok("+33 6 12 34".to_owned()));
    assert_eq!(kept(marked, true), Err(EntryError::Refused));
}

#[test]
fn any_other_hidden_mark_is_refused_shown_or_hidden() {
    let others = [
        '\u{202e}',
        '\u{2066}',
        '\u{200b}',
        '\u{feff}',
        '\u{ad}',
        '\u{1b}',
        '\u{e0041}',
    ];
    for mark in others {
        for hidden in [false, true] {
            assert_eq!(
                kept(&format!("ab{mark}c"), hidden),
                Err(EntryError::Refused),
                "{mark:?}"
            );
        }
    }
}

#[test]
fn arabic_and_persian_keep_their_joiners_shown_or_hidden() {
    for text in [
        "عبد\u{200c}الله",
        "می\u{200c}خواهم",
        "لا\u{200d}",
        "كَلِمَةُ السِّرِّ",
    ] {
        for hidden in [false, true] {
            assert_eq!(kept(text, hidden), Ok(text.to_owned()), "{text:?}");
        }
    }
}

#[test]
fn the_name_and_an_own_label_follow_the_shown_rules() {
    let named = |name: &str| Entry::new(2, draft(name, EntryKind::Note, vec![]), NOW, &LIMITS);
    assert_eq!(named("\u{200f}بنك\u{200e}").unwrap().name().expose(), "بنك");
    assert_eq!(named("Bank\nX").err(), Some(EntryError::Refused));
    assert_eq!(
        named(&"n".repeat(LIMITS.field_chars + 1)).err(),
        Some(EntryError::Refused)
    );
    let with_own = |label: &str| entry_of(vec![own(label, "1234", true)], &LIMITS);
    let label = with_own("\u{200e}رمز البطاقة").unwrap().fields()[0]
        .label
        .clone();
    assert_eq!(label, Label::Own(Redacted::new("رمز البطاقة".to_owned())));
    assert_eq!(with_own("PIN\u{202e}").err(), Some(EntryError::Refused));
}

#[test]
fn a_table_label_must_be_in_the_table() {
    assert_eq!(
        entry_of(vec![field("colour", "red", false)], &LIMITS).err(),
        Some(EntryError::Refused)
    );
}

#[test]
fn a_new_entry_keeps_its_id_kind_and_change_time_and_was_never_used() {
    let entry = Entry::new(
        9,
        draft("Wi-Fi maison", EntryKind::Wifi, vec![]),
        NOW,
        &LIMITS,
    )
    .unwrap();
    assert_eq!(
        (entry.id(), entry.kind(), entry.changed(), entry.used()),
        (9, EntryKind::Wifi, NOW, None)
    );
}

#[test]
fn an_entry_gives_back_its_draft() {
    let fields = vec![field("user", "alex", false), own("Code", "4321", true)];
    let entry = entry_of(fields.clone(), &LIMITS).unwrap();
    assert_eq!(entry.draft(), draft("Bank", EntryKind::Other, fields));
}

#[test]
fn no_debug_shows_a_value_a_name_or_an_own_label() {
    let fields = vec![
        field("user", "zebra-quartz", false),
        own("كلمة", "hunter2", true),
    ];
    let entry = Entry::new(3, draft("Banque", EntryKind::Login, fields), NOW, &LIMITS).unwrap();
    let shown = format!("{entry:?}{:?}", entry.draft());
    for secret in ["zebra-quartz", "hunter2", "كلمة", "Banque"] {
        assert!(!shown.contains(secret), "{secret} in {shown}");
    }
    assert!(shown.contains(REDACTED_MARKER));
}

#[test]
fn the_no_random_error_has_no_panel_note() {
    assert_eq!(
        EntryError::from(kx_module_api::ports::RandomError),
        EntryError::NoRandom
    );
    assert_eq!(EntryError::NoRandom.note(), None);
}

#[test]
fn an_own_label_of_blanks_alone_is_refused() {
    for label in ["", "  ", "\u{200e} \u{a0}"] {
        let saved = entry_of(vec![own(label, "1234", true)], &LIMITS);
        assert_eq!(saved.err(), Some(EntryError::Refused), "{label:?}");
    }
    assert!(entry_of(vec![own(" PIN ", "1234", true)], &LIMITS).is_ok());
}
