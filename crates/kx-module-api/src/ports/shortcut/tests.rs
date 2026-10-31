//! The shortcuts port: the id and key-name form, their redacted `Debug` and serde, the key's gate and the errors.

use super::*;
use crate::REDACTED_MARKER;
use proptest::prelude::*;
use std::collections::HashSet;

/// Ids and names the form takes.
const GOOD: [&str; 6] = ["copy-all", "f5", "ctrl", "numplus", "a", "5"];
/// Text the form refuses: empty, a dash at an edge, capitals, spaces, other signs and non-ASCII letters.
const BAD: [&str; 9] = [
    "", "-copy", "copy-", "Copy", "copy all", "copy_all", "f5!", "é", "٥",
];

/// The form, written out as the port states it.
fn well_formed(text: &str) -> bool {
    let chars = text
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    let edges = !text.starts_with('-') && !text.ends_with('-');
    !text.is_empty() && text.len() <= MAX_ID_LEN && chars && edges
}

#[test]
fn ids_and_names_take_the_good_form_and_refuse_the_bad() {
    for text in GOOD {
        assert_eq!(ShortcutId::new(text).unwrap().as_str(), text);
        assert_eq!(KeyName::new(text).unwrap().as_str(), text);
    }
    for text in BAD {
        assert!(ShortcutId::new(text).is_none(), "{text:?}");
        assert!(KeyName::new(text).is_none(), "{text:?}");
    }
}

#[test]
fn an_id_is_at_most_max_id_len_long() {
    let longest = "a".repeat(MAX_ID_LEN);
    assert!(ShortcutId::new(&longest).is_some());
    assert!(KeyName::new(&longest).is_some());
    let over = "a".repeat(MAX_ID_LEN + 1);
    assert!(ShortcutId::new(&over).is_none());
    assert!(KeyName::new(&over).is_none());
}

proptest! {
    #[test]
    fn any_text_is_taken_just_when_it_has_the_form(text in "[a-z0-9_ A-Z-]{0,40}") {
        prop_assert_eq!(ShortcutId::new(&text).is_some(), well_formed(&text));
        prop_assert_eq!(KeyName::new(&text).is_some(), well_formed(&text));
    }

    #[test]
    fn a_good_id_survives_json(text in "[a-z0-9]([a-z0-9-]{0,30}[a-z0-9])?") {
        let id = ShortcutId::new(&text).unwrap();
        let json = serde_json::to_string(&id).unwrap();
        prop_assert_eq!(&json, &format!("\"{text}\""));
        prop_assert_eq!(serde_json::from_str::<ShortcutId>(&json).unwrap(), id);
    }
}

#[test]
fn serde_refuses_a_bad_form_without_naming_it() {
    for text in BAD {
        let json = serde_json::to_string(text).unwrap();
        let id = serde_json::from_str::<ShortcutId>(&json);
        let name = serde_json::from_str::<KeyName>(&json);
        assert!(id.is_err() && name.is_err(), "{text:?}");
    }
    let err = serde_json::from_str::<ShortcutId>(r#""Secret-Key""#).unwrap_err();
    assert!(!err.to_string().contains("Secret"), "{err}");
    let name: KeyName = toml::from_str::<Named>("key = \"numplus\"").unwrap().key;
    assert_eq!(name.as_str(), "numplus");
}

/// A data file that names a key.
#[derive(serde::Deserialize)]
struct Named {
    key: KeyName,
}

#[test]
fn debug_never_shows_an_id_or_a_key() {
    let id = ShortcutId::new("copy-all").unwrap();
    let ctrl = KeyName::new("ctrl").unwrap();
    let step = Step {
        held: vec![ctrl.clone()],
        key: KeyName::new("a").unwrap(),
    };
    let info = ShortcutInfo {
        id: id.clone(),
        steps: vec![step],
    };
    let shown = format!("{id:?} {ctrl:?} {info:?}");
    for secret in ["copy", "ctrl", "\"a\""] {
        assert!(!shown.contains(secret), "{shown} shows {secret}");
    }
    assert_eq!(format!("{id:?}"), REDACTED_MARKER);
    assert_eq!(format!("{ctrl:?}"), REDACTED_MARKER);
}

#[test]
fn the_shortcuts_need_inject_input() {
    assert_eq!(ShortcutsKey::ID.as_str(), "shortcuts");
    assert_eq!(ShortcutsKey::CAPABILITY, Some(Capability::InjectInput));
}

#[test]
fn each_error_says_something_else_and_a_refusal_counts() {
    let errors = [
        ShortcutError::Unknown,
        ShortcutError::Refused { sent: 3, total: 8 },
        ShortcutError::Failed,
    ];
    let texts: HashSet<_> = errors.iter().map(ToString::to_string).collect();
    assert_eq!(texts.len(), errors.len());
    let refused = errors[1].to_string();
    assert!(refused.contains('3') && refused.contains('8'), "{refused}");
}

#[test]
fn the_paste_the_keyboard_sends_on_request_is_a_well_formed_id() {
    assert_eq!(
        ShortcutId::new(PASTE_ID).map(|id| id.as_str().to_owned()),
        Some(PASTE_ID.to_owned())
    );
}
