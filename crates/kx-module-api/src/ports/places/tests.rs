//! The places port: its gate, the finder's name and mark rules, `Debug` that never shows a path or a name, and the
//! errors.

use super::*;
use crate::REDACTED_MARKER;
use crate::ports::{BOX_VERSION, box_file};
use proptest::prelude::*;
use std::collections::HashSet;
use std::path::PathBuf;

/// Texts that appear nowhere else in the output: a user's folder and a drive's Arabic label ("my files").
const SECRETS: [&str; 2] = [r"C:\Users\zebra-quartz\OneDrive", "ملفاتي"];

/// A place named and found at the secrets.
fn place(kind: PlaceKind) -> Place {
    Place {
        kind,
        name: Redacted::new(SECRETS[1].to_owned()),
        path: Redacted::new(PathBuf::from(SECRETS[0])),
    }
}

#[test]
fn places_need_vault_data() {
    assert_eq!(PlacesKey::ID.as_str(), "places");
    assert_eq!(PlacesKey::CAPABILITY, Some(Capability::VaultData));
}

#[test]
fn a_box_name_holds_keyxtend_in_any_case_and_ends_with_the_box_extension() {
    let ours = box_file();
    let found = [
        ours.as_str(),
        "my KEYXTEND copy.kxbox",
        "keyxtend backup (1).kxbox",
        "KeyXtend Backup.KXBOX",
    ];
    for name in found {
        assert!(is_box_name(name), "{name}");
    }
    let other = [
        "keyxtend.txt",
        "Backup.kxbox",
        "keyxtend.kxbox.txt",
        "KeyXtend Backup",
        "kxbox",
        "",
    ];
    for name in other {
        assert!(!is_box_name(name), "{name}");
    }
}

proptest! {
    #[test]
    fn the_box_file_in_any_case_is_a_box_name(flips in prop::collection::vec(any::<bool>(), box_file().len())) {
        let name: String = box_file()
            .chars()
            .zip(flips)
            .map(|(c, up)| if up { c.to_ascii_uppercase() } else { c.to_ascii_lowercase() })
            .collect();
        prop_assert!(is_box_name(&name), "{}", name);
    }
}

#[test]
fn a_box_starts_with_the_mark() {
    let head = [&BOX_MARK[..], &[BOX_VERSION]].concat();
    assert!(is_box_head(&head));
    assert!(is_box_head(&BOX_MARK));
    assert!(!is_box_head(&BOX_MARK[1..]));
    assert!(!is_box_head(&head[..BOX_MARK.len() - 1]));
    assert!(!is_box_head(&[]));
}

#[test]
fn debug_never_shows_a_path_or_a_name() {
    let found = Found {
        place: place(PlaceKind::Cloud),
        path: Redacted::new(PathBuf::from(SECRETS[0]).join(box_file())),
        modified_ms: 1,
    };
    for shown in [
        format!("{:?}", place(PlaceKind::Removable)),
        format!("{found:?}"),
    ] {
        assert!(shown.contains(REDACTED_MARKER), "{shown}");
        for secret in SECRETS {
            assert!(!shown.contains(secret), "{shown} shows {secret}");
        }
    }
}

#[test]
fn every_kind_is_listed_once_in_declaration_order() {
    for (at, kind) in PlaceKind::ALL.into_iter().enumerate() {
        assert_eq!(kind as usize, at, "{kind:?}");
    }
}

#[test]
fn each_error_says_something_else() {
    let errors = [
        PlaceError::Missing,
        PlaceError::Full,
        PlaceError::ReadOnly,
        PlaceError::Refused,
        PlaceError::TooLarge,
        PlaceError::Io,
    ];
    let texts: HashSet<_> = errors.iter().map(ToString::to_string).collect();
    assert_eq!(texts.len(), errors.len());
}

#[test]
fn each_state_is_its_own() {
    let states = [
        PlaceState::Ready,
        PlaceState::Missing,
        PlaceState::Full,
        PlaceState::Refused,
    ];
    for (at, a) in states.iter().enumerate() {
        assert!(states[at + 1..].iter().all(|b| a != b), "{a:?}");
    }
}
