//! The fake sealed store, held to the port: saved bytes load back, one failure on cue each way, a damaged blob set
//! aside, and calls counted.

use super::*;
use std::thread;

const OLD: &[u8] = b"old words";
const NEW: &[u8] = "كلمات".as_bytes();

/// A checked test name.
fn name(text: &str) -> BlobName {
    BlobName::new(text).unwrap()
}

#[test]
fn save_then_load_returns_the_bytes_kept_apart_by_name() {
    let store = FakeSealedStore::new();
    let (en, fr) = (name("predict-en"), name("predict-fr"));
    assert_eq!(store.load(&en), Ok(None), "nothing saved yet");
    assert_eq!(store.save(&en, OLD), Ok(()));
    assert_eq!(store.save(&fr, NEW), Ok(()));
    assert_eq!(store.load(&en), Ok(Some(OLD.to_vec())));
    assert_eq!(store.load(&fr), Ok(Some(NEW.to_vec())));
    assert_eq!(store.save(&en, NEW), Ok(()));
    assert_eq!(store.blob(&en), Some(NEW.to_vec()), "a save replaces");
}

#[test]
fn a_scripted_locked_load_comes_back_once_then_the_store_works_again() {
    let store = FakeSealedStore::new();
    let en = name("predict-en");
    store.plant(&en, OLD.to_vec());
    store.fail_load(SealedError::Locked);
    assert_eq!(store.load(&en), Err(SealedError::Locked));
    assert_eq!(store.load(&en), Ok(Some(OLD.to_vec())));
}

#[test]
fn a_scripted_damaged_load_comes_back_once_and_sets_the_blob_aside() {
    let store = FakeSealedStore::new();
    let en = name("predict-en");
    store.plant(&en, OLD.to_vec());
    store.fail_load(SealedError::Damaged);
    assert_eq!(store.load(&en), Err(SealedError::Damaged));
    assert_eq!(store.load(&en), Ok(None), "the adapter set the file aside");
    assert_eq!(store.blob(&en), None);
}

#[test]
fn a_scripted_save_failure_keeps_the_old_blob_once() {
    let store = FakeSealedStore::new();
    let en = name("predict-en");
    store.plant(&en, OLD.to_vec());
    store.fail_save(SealedError::Io);
    assert_eq!(store.save(&en, NEW), Err(SealedError::Io));
    assert_eq!(store.blob(&en), Some(OLD.to_vec()));
    assert_eq!(store.save(&en, NEW), Ok(()));
    assert_eq!(store.blob(&en), Some(NEW.to_vec()));
}

#[test]
fn loads_and_saves_are_counted_refused_ones_too() {
    let store = FakeSealedStore::new();
    let en = name("predict-en");
    store.plant(&en, OLD.to_vec());
    assert_eq!((store.loads(), store.saves()), (0, 0), "a plant is no save");
    store.fail_save(SealedError::Locked);
    store.fail_load(SealedError::Locked);
    let _ = store.save(&en, NEW);
    let _ = store.save(&en, NEW);
    let _ = store.load(&en);
    assert_eq!((store.loads(), store.saves()), (1, 2));
}

#[test]
fn clones_share_the_blobs_and_serve_as_the_port_from_another_thread() {
    let store = FakeSealedStore::new();
    let port: Arc<dyn SealedStore> = Arc::new(store.clone());
    let saved = thread::spawn(move || port.save(&name("predict-en"), OLD));
    assert_eq!(saved.join().unwrap(), Ok(()));
    assert_eq!(store.blob(&name("predict-en")), Some(OLD.to_vec()));
}

#[test]
fn debug_never_prints_the_bytes() {
    let store = FakeSealedStore::new();
    store.plant(&name("predict-en"), OLD.to_vec());
    let text = format!("{store:?}");
    assert!(text.contains("predict-en"));
    assert!(!text.contains(&*String::from_utf8_lossy(OLD)));
}
