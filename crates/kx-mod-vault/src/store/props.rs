//! Property tests of the notebook's format: random bytes, a good notebook's bytes cut or changed anywhere, never
//! panic, and any notebook the rules allow comes back as it was saved.

use super::codec::{MARK, decode, encode};
use crate::book::Notebook;
use crate::entry::{Draft, Field, Stamp};
use crate::fixture::{LIMITS, own, secret};
use crate::kinds::tables;
use kx_module_api::Redacted;
use kx_module_api::ports::{EntryKind, Label};
use kx_platform_fake::FakeRandom;
use proptest::prelude::*;

/// Texts of letters, Arabic, joiners and spaces, which the rules keep.
fn kept_text() -> impl Strategy<Value = String> {
    let chars = vec![
        'a', 'Z', 'é', 'ع', 'ى', 'ـ', ' ', '\u{200c}', '\u{200d}', '7', '@',
    ];
    prop::collection::vec(prop::sample::select(chars), 0..8).prop_map(|v| v.into_iter().collect())
}

/// A field under a table label or an own one.
fn any_field() -> impl Strategy<Value = Field> {
    let label = 0..=tables().labels().len();
    (label, kept_text(), kept_text(), any::<bool>()).prop_map(|(at, mine, text, hidden)| {
        match tables().labels().get(at) {
            Some(row) => Field {
                label: Label::Table(row.id.clone()),
                value: secret(&text),
                hidden,
            },
            None => own(&mine, &text, hidden),
        }
    })
}

/// A draft of any kind.
fn any_draft() -> impl Strategy<Value = Draft> {
    let kind = prop::sample::select(EntryKind::ALL.to_vec());
    let fields = prop::collection::vec(any_field(), 0..5);
    (kept_text(), kind, fields).prop_map(|(name, kind, fields)| Draft {
        name: Redacted::new(name),
        kind,
        fields,
    })
}

/// The notebook of `drafts`, each saved, and every other one filled.
fn book_of(drafts: Vec<Draft>, seed: u64) -> Notebook {
    let (mut book, random) = (Notebook::default(), FakeRandom::new(seed));
    for (n, d) in drafts.into_iter().enumerate() {
        let at = Stamp::from_unix_ms(i64::try_from(n).unwrap_or_default());
        if let Ok(id) = book.add(d, at, &random, &LIMITS)
            && n % 2 == 0
        {
            book.use_at(id, at);
        }
    }
    book
}

proptest! {
    #[test]
    fn random_bytes_never_panic(bytes in prop::collection::vec(any::<u8>(), 0..256)) {
        let _ = decode(&bytes);
        let headed = [MARK.as_slice(), &[1], &bytes].concat();
        let _ = decode(&headed);
    }

    #[test]
    fn any_notebook_comes_back_as_it_was_saved(drafts in prop::collection::vec(any_draft(), 0..6), seed: u64) {
        let book = book_of(drafts, seed);
        let back = decode(&encode(book.entries())).map(Notebook::new);
        prop_assert_eq!(back, Ok(book));
    }

    #[test]
    fn a_cut_or_changed_notebook_never_panics(
        drafts in prop::collection::vec(any_draft(), 1..4),
        at: prop::sample::Index,
        byte: u8,
    ) {
        let bytes = encode(book_of(drafts, 1).entries()).to_vec();
        let at = at.index(bytes.len());
        prop_assert!(decode(&bytes[..at]).is_err());
        let mut changed = bytes.clone();
        changed[at] = byte;
        let _ = decode(&changed);
    }
}
