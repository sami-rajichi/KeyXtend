//! Property tests of the import: any `KeepassImport`, over any notebook and limits, keeps every notebook rule, never
//! panics, and counts each entry that came in.

use super::*;
use crate::entry::text::held;
use crate::fixture::{LIMITS, draft, field, secret};
use crate::store::{HEAD_LEN, Store};
use kx_module_api::ports::{Dropped, KeepassField};
use kx_module_api::{Secret, is_hidden};
use kx_platform_fake::FakeRandom;
use proptest::prelude::*;
use std::collections::BTreeSet;

/// When the import runs.
const NOW: Stamp = Stamp::from_unix_ms(1_700_000_000_000);

/// Characters a file meets: any at all, and often letters, Arabic, breaks and marks.
fn any_char() -> impl Strategy<Value = char> {
    let picked = vec![
        'a', 'A', 'é', 'ع', 'ى', ' ', '\u{200c}', '\u{200e}', '\u{200f}', '\u{61c}', '\u{202e}',
        '\n', '\r', '\u{2028}', '\u{85}',
    ];
    prop_oneof![1 => any::<char>(), 4 => prop::sample::select(picked)]
}

/// Texts of up to `most` such characters.
fn any_text(most: usize) -> impl Strategy<Value = String> {
    prop::collection::vec(any_char(), 0..=most).prop_map(|v| v.into_iter().collect())
}

/// Such a text as a secret.
fn any_secret(most: usize) -> impl Strategy<Value = Secret> {
    any_text(most).prop_map(|t| Redacted::new(Zeroizing::new(t)))
}

/// An entry the worker might give.
fn any_entry() -> impl Strategy<Value = KeepassEntry> {
    let field = (any_text(6), any_secret(6)).prop_map(|(label, value)| KeepassField {
        label: Redacted::new(label),
        value,
    });
    (
        any_secret(6),
        any_secret(4),
        any_secret(6),
        any_secret(6),
        any_secret(10),
        prop::collection::vec(field, 0..5),
    )
        .prop_map(
            |(title, user, password, address, notes, custom)| KeepassEntry {
                title,
                user,
                password,
                address,
                notes,
                custom,
            },
        )
}

/// What a file might give, with the worker's drops.
fn any_import() -> impl Strategy<Value = KeepassImport> {
    let dropped =
        (any::<u32>(), any::<u32>()).prop_map(|(entries, fields)| Dropped { entries, fields });
    (prop::collection::vec(any_entry(), 0..12), dropped)
        .prop_map(|(entries, dropped)| KeepassImport { entries, dropped })
}

/// Limits a user may set, down to small ones, with the saved size close enough to bite.
fn any_limits() -> impl Strategy<Value = Limits> {
    (1usize..8, 1usize..10, 1usize..6, HEAD_LEN..2_000).prop_map(
        |(field_chars, max_entries, max_fields, max_bytes)| Limits {
            field_chars,
            max_entries,
            max_fields,
            max_bytes,
        },
    )
}

/// Letters of a real value, ZWNJ among them, as Arabic and Persian words need it.
const LETTERS: [char; 6] = ['a', 'é', 'ع', 'ى', '٤', '\u{200c}'];

/// An entry holding `marked` as its password and as a custom string.
fn marked_entry(marked: &str) -> KeepassEntry {
    KeepassEntry {
        title: secret("Bank"),
        user: secret("sam"),
        password: secret(marked),
        address: secret(""),
        notes: secret(""),
        custom: vec![KeepassField {
            label: Redacted::new("Code".to_owned()),
            value: secret(marked),
        }],
    }
}

/// A notebook of up to `n` saved entries.
fn saved(n: usize) -> Notebook {
    let (mut book, random) = (Notebook::default(), FakeRandom::new(11));
    for i in 0..n {
        let fields = vec![field("user", "sam", false), field("password", "s3", true)];
        let name = format!("Saved {i}");
        book.add(
            draft(&name, EntryKind::Login, fields),
            NOW,
            &random,
            &LIMITS,
        )
        .unwrap();
    }
    book
}

proptest! {
    #[test]
    fn any_import_keeps_every_notebook_rule(
        import in any_import(),
        limits in any_limits(),
        have in 0usize..4,
    ) {
        let mut book = saved(have);
        let before = book.clone();
        let done = bring_in(&mut book, import, NOW, &FakeRandom::new(5), &limits);
        prop_assert_eq!(usize::try_from(done.added).unwrap(), book.len() - have);
        prop_assert_eq!(&book.entries()[..have], before.entries(), "what was saved stays");
        if done.added > 0 {
            prop_assert!(book.len() <= limits.max_entries);
            prop_assert!(book.bytes() <= limits.max_bytes);
        }
        let ids: BTreeSet<u64> = book.entries().iter().map(Entry::id).collect();
        prop_assert_eq!(ids.len(), book.len());
        let mut keys = HashSet::new();
        for entry in &book.entries()[have..] {
            let max = limits.field_chars;
            prop_assert!(held(entry.name().expose(), max).is_ok());
            prop_assert!(!entry.fields().is_empty());
            prop_assert!(entry.fields().len() <= limits.max_fields);
            let key = Key::of_entry(entry);
            prop_assert!(key.name.is_empty() || keys.insert(key), "no named duplicate comes in");
            for f in entry.fields() {
                prop_assert!(!f.value.expose().is_empty());
                prop_assert!(!f.value.expose().chars().any(is_hidden));
                prop_assert!(f.value.expose().chars().count() <= max);
                if let Label::Own(own) = &f.label {
                    prop_assert!(held(own.expose(), max).is_ok() && !text::blank(own.expose()));
                    prop_assert!(f.hidden, "a custom string starts hidden");
                }
            }
            let has = entry.fields().iter().any(|f| under(f, &sheet().password));
            let want = if has { EntryKind::Login } else { EntryKind::Other };
            prop_assert_eq!(entry.kind(), want);
        }
        let back = Store::read_part(&Store::part(&book)).ok();
        prop_assert_eq!(back.as_deref(), Some(book.entries()), "it saves and reads back");
    }

    #[test]
    fn direction_marks_cost_a_custom_value_nothing_and_always_cost_a_password(
        letters in prop::collection::vec(prop::sample::select(LETTERS.to_vec()), 1..8),
        marks in prop::collection::vec(
            (any::<prop::sample::Index>(), prop::sample::select(text::STRIPPED.to_vec())),
            1..4,
        ),
    ) {
        let clean: String = letters.iter().collect();
        let mut marked = letters;
        for (at, mark) in marks {
            let at = at.index(marked.len() + 1);
            marked.insert(at, mark);
        }
        let marked: String = marked.into_iter().collect();
        let mut book = Notebook::default();
        let import = vec![marked_entry(&marked)].into();
        let done = bring_in(&mut book, import, NOW, &FakeRandom::new(5), &LIMITS);
        prop_assert_eq!(done, NoteCounts { added: 1, skipped: 1 }, "the password alone");
        let fields = book.entries()[0].fields();
        prop_assert!(!fields.iter().any(|f| under(f, &sheet().password)));
        let own = fields.iter().find(|f| matches!(f.label, Label::Own(_)));
        prop_assert_eq!(own.map(|f| f.value.expose().as_str()), Some(clean.as_str()));
        prop_assert!(own.is_some_and(|f| f.hidden));
    }
}
