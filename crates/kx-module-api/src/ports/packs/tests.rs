//! The word packs port: its name, and no gate, as packs hold no user data.

use super::*;

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
fn a_pack_file_is_named_with_its_extension() {
    assert_eq!(PACK_EXT, "kxw");
}
