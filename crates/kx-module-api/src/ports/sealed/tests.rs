//! The sealed store port: the blob name's form, the store's gate and the errors.

use super::*;
use proptest::prelude::*;
use std::collections::HashSet;

/// Names the form takes, the learner's file among them.
const GOOD: [&str; 4] = ["predict-en", "a", "5", "emails-2"];
/// Names it refuses: empty, capitals, spaces, other signs and non-ASCII letters.
const BAD: [&str; 7] = [
    "",
    "Predict-en",
    "predict en",
    "predict_en",
    "a/b",
    "é",
    "ع",
];

/// The form, written out as the port states it.
fn well_formed(text: &str) -> bool {
    let chars = text
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    (MIN_BLOB_LEN..=MAX_BLOB_LEN).contains(&text.len()) && chars
}

#[test]
fn the_store_is_named_and_gated_by_secret_store() {
    assert_eq!(SealedStoreKey::ID.as_str(), "sealed-store");
    assert_eq!(SealedStoreKey::CAPABILITY, Some(Capability::SecretStore));
}

#[test]
fn a_name_takes_the_good_form_and_refuses_the_bad() {
    for text in GOOD {
        assert_eq!(BlobName::new(text).unwrap().as_str(), text);
    }
    for text in BAD {
        assert_eq!(BlobName::new(text), Err(SealedError::BadName), "{text:?}");
    }
}

#[test]
fn a_name_is_at_most_max_blob_len_long() {
    let longest = "a".repeat(MAX_BLOB_LEN);
    assert!(BlobName::new(&longest).is_ok());
    let over = "a".repeat(MAX_BLOB_LEN + 1);
    assert_eq!(BlobName::new(&over), Err(SealedError::BadName));
}

proptest! {
    #[test]
    fn any_text_is_taken_just_when_it_has_the_form(text in "[a-z0-9_ A-Z-]{0,40}") {
        prop_assert_eq!(BlobName::new(&text).is_ok(), well_formed(&text));
    }
}

#[test]
fn each_error_says_something_else() {
    let errors = [
        SealedError::BadName,
        SealedError::Damaged,
        SealedError::Locked,
        SealedError::Io,
    ];
    let texts: HashSet<_> = errors.iter().map(ToString::to_string).collect();
    assert_eq!(texts.len(), errors.len());
}
