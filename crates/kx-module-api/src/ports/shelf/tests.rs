//! The sealed shelf port: its gate, and the folder names the history uses.

use super::*;

#[test]
fn the_shelf_is_named_and_gated_by_secret_store() {
    assert_eq!(SealedShelfKey::ID.as_str(), "sealed-shelf");
    assert_eq!(SealedShelfKey::CAPABILITY, Some(Capability::SecretStore));
}

#[test]
fn a_day_and_the_pinned_folder_are_blob_names() {
    for folder in ["2026-10-30", "pinned"] {
        assert_eq!(BlobName::new(folder).unwrap().as_str(), folder);
    }
}
