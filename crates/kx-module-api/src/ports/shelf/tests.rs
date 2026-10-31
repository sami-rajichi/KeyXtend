//! The sealed shelf port: its gate, the folder names the history uses, and the free bytes from the room.

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

#[test]
fn free_defaults_to_the_rooms_free_bytes() {
    struct Roomy;
    impl SealedShelf for Roomy {
        fn load(&self, _: &BlobName, _: &BlobName) -> Result<Option<Vec<u8>>, SealedError> {
            Ok(None)
        }
        fn save(&self, _: &BlobName, _: &BlobName, _: &[u8]) -> Result<(), SealedError> {
            Ok(())
        }
        fn remove(&self, _: &BlobName, _: &BlobName) -> Result<(), SealedError> {
            Ok(())
        }
        fn drop_folder(&self, _: &BlobName) -> Result<(), SealedError> {
            Ok(())
        }
        fn folders(&self) -> Result<Vec<BlobName>, SealedError> {
            Ok(Vec::new())
        }
        fn names(&self, _: &BlobName) -> Result<Vec<BlobName>, SealedError> {
            Ok(Vec::new())
        }
        fn room(&self) -> Result<Room, SealedError> {
            let drive = "D:".to_owned();
            Ok(Room {
                used: 1,
                free: 5,
                drive,
            })
        }
    }
    assert_eq!(Roomy.free(), Ok(5));
}
