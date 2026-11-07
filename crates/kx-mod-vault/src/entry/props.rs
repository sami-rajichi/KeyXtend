//! Property tests of the entry rules: any text, draft or typing never panics, and what is kept always obeys the rules.

use super::text::{STRIPPED, fit};
use super::*;
use crate::fixture::{LIMITS, draft, field, own};
use kx_module_api::is_hidden;
use proptest::prelude::*;

/// Characters a value meets: any at all, and often letters, Arabic, the joiners, the stripped marks and other marks.
fn any_char() -> impl Strategy<Value = char> {
    let picked = vec![
        'a', 'é', 'ع', 'ى', ' ', '\u{200c}', '\u{200d}', '\u{200e}', '\u{200f}', '\u{61c}',
        '\u{202e}', '\n',
    ];
    prop_oneof![any::<char>(), prop::sample::select(picked)]
}

/// Texts of up to `most` such characters.
fn any_text(most: usize) -> impl Strategy<Value = String> {
    prop::collection::vec(any_char(), 0..=most).prop_map(|v| v.into_iter().collect())
}

proptest! {
    #[test]
    fn a_fitted_text_holds_no_mark_and_stays_within_its_length(
        text in any_text(12),
        hidden: bool,
        max in 0usize..10,
    ) {
        let mut fitted = text.clone();
        match fit(&mut fitted, hidden, max) {
            Ok(()) => {
                prop_assert!(!fitted.chars().any(is_hidden));
                prop_assert!(fitted.chars().count() <= max);
                let want: String = text.chars().filter(|c| hidden || !STRIPPED.contains(c)).collect();
                prop_assert_eq!(fitted, want);
            }
            Err(e) => prop_assert_eq!(e, EntryError::Refused),
        }
    }

    #[test]
    fn a_saved_entry_always_obeys_the_rules(
        name in any_text(8),
        texts in prop::collection::vec((any_text(6), any::<bool>(), any::<bool>()), 0..6),
        max_fields in 1usize..5,
        field_chars in 1usize..6,
    ) {
        let limits = Limits { field_chars, max_fields, ..LIMITS };
        let make = |(t, hidden, mine): &(String, bool, bool)| {
            if *mine { own("Mine", t, *hidden) } else { field("text", t, *hidden) }
        };
        let fields = texts.iter().map(make).collect();
        let saved = Entry::new(1, draft(&name, EntryKind::Note, fields), Stamp::from_unix_ms(0), &limits);
        if let Ok(entry) = saved {
            prop_assert!(entry.fields().len() <= max_fields);
            for f in entry.fields() {
                let v = f.value.expose();
                prop_assert!(!v.is_empty() && v.chars().count() <= field_chars);
                prop_assert!(!v.chars().any(is_hidden));
            }
        }
    }

    #[test]
    fn a_line_types_what_it_takes_and_never_passes_its_most(
        steps in prop::collection::vec(prop::option::of(any_char()), 0..40),
        max in 0usize..12,
    ) {
        let mut line = Line::new(max);
        let mut model = String::new();
        for step in steps {
            if let Some(c) = step {
                if line.push(c).is_ok() {
                    model.push(c);
                }
            } else {
                prop_assert_eq!(line.pop(), model.pop());
            }
            prop_assert_eq!(line.text(), model.as_str());
            let counted = model.chars().filter(|c| !STRIPPED.contains(c)).count();
            prop_assert!(line.chars() == counted && counted <= max);
        }
        let mut saved = model;
        prop_assert_eq!(fit(&mut saved, false, max), Ok(()), "a typed line saves as a shown text");
    }
}
