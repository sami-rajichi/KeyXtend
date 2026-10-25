//! The word packs port: its name, no gate, and the one language-code rule that names a pack's file.

use super::*;
use crate::ports::RESERVED_NAMES;
use proptest::prelude::*;
use std::ffi::OsStr;
use std::path::Path;

/// Random codes tried by the property test.
const CASES: u32 = 1_024;

#[test]
fn the_packs_are_named_and_need_no_capability() {
    assert_eq!(WordPacksKey::ID.as_str(), "word-packs");
    assert_eq!(WordPacksKey::CAPABILITY, None);
}

#[test]
fn the_error_says_the_pack_is_unreadable() {
    assert!(PackError::Unreadable.to_string().contains("pack"));
}

#[test]
fn a_two_or_three_letter_code_names_its_pack_file() {
    for iso in ["en", "fr", "ar", "fil"] {
        assert!(fits_iso(iso), "{iso}");
        assert_eq!(pack_file(iso), Some(format!("{iso}.{PACK_EXT}")));
    }
}

#[test]
fn any_other_code_names_no_pack() {
    let bad = [
        "", "e", "engl", "EN", "En", "../x", "..", "e1", "é", "ع", "e n", "en\0", "en.", "a/b",
        "a\\b", "CON",
    ];
    for iso in bad {
        assert!(!fits_iso(iso), "{iso:?}");
        assert_eq!(pack_file(iso), None, "{iso:?}");
    }
}

#[test]
fn a_device_name_names_no_pack_though_it_has_the_form() {
    let short = RESERVED_NAMES.iter().filter(|n| n.len() <= MAX_ISO_LEN);
    for iso in short {
        assert_eq!(pack_file(iso), None, "{iso}");
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn only_short_lowercase_codes_that_name_no_device_name_a_file_in_its_folder(
        iso in prop_oneof![
            "[a-z]{2,3}",
            "[a-zA-Z./\\\\:]{0,5}",
            "\\PC{0,5}",
            prop::sample::select(RESERVED_NAMES).prop_map(str::to_owned),
        ],
    ) {
        let good = (MIN_ISO_LEN..=MAX_ISO_LEN).contains(&iso.len())
            && iso.bytes().all(|b| b.is_ascii_lowercase())
            && !RESERVED_NAMES.contains(&iso.as_str());
        prop_assert_eq!(fits_iso(&iso), good);
        let name = pack_file(&iso);
        prop_assert_eq!(name.is_some(), good);
        if let Some(name) = name {
            prop_assert_eq!(Path::new(&name).file_stem(), Some(OsStr::new(&iso)));
            let dir = std::env::temp_dir();
            let path = dir.join(&name);
            prop_assert_eq!(path.parent(), Some(dir.as_path()));
        }
    }
}

#[test]
fn the_index_is_named_apart_from_every_pack() {
    assert!(!PACK_INDEX.ends_with(PACK_EXT));
}
