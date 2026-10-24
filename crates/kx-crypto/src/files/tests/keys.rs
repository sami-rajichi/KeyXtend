//! The data key's file: locked guards, lost key files and refused wraps.

use super::*;
use crate::consts::MAX_KEY_FILE_BYTES;

#[test]
fn a_locked_guard_gives_locked_and_leaves_every_file() {
    let c = case();
    c.store.save(&name("predict-en"), b"words").unwrap();
    let key = fs::read(c.key_file()).unwrap();
    let blob = fs::read(c.file("predict-en", SEALED_EXT)).unwrap();
    let later = c.reopen(&Mirror::refusing());
    assert_eq!(later.load(&name("predict-en")), Err(SealedError::Locked));
    assert_eq!(
        later.save(&name("predict-en"), b"new"),
        Err(SealedError::Locked)
    );
    assert_eq!(fs::read(c.key_file()).unwrap(), key);
    assert_eq!(fs::read(c.file("predict-en", SEALED_EXT)).unwrap(), blob);
    assert_eq!(c.names(), ["predict-en.sealed", KEY_FILE]);
}

#[test]
fn a_key_file_the_guard_cannot_read_stays_locked() {
    let c = case();
    c.store.save(&name("emails"), b"a@b.c").unwrap();
    let short = fs::read(c.key_file()).unwrap()[1..].to_vec();
    fs::write(c.key_file(), &short).unwrap();
    let later = c.reopen(&Mirror::default());
    assert_eq!(later.load(&name("emails")), Err(SealedError::Locked));
    assert_eq!(
        later.save(&name("emails"), b"new"),
        Err(SealedError::Locked)
    );
    assert_eq!(fs::read(c.key_file()).unwrap(), short);
}

#[test]
fn an_oversized_key_file_stays_locked_and_untouched() {
    let c = case();
    c.store.save(&name("emails"), b"a@b.c").unwrap();
    let big = vec![1; MAX_KEY_FILE_BYTES + 1];
    fs::write(c.key_file(), &big).unwrap();
    let guard = Mirror::default();
    let later = c.reopen(&guard);
    assert_eq!(later.load(&name("emails")), Err(SealedError::Locked));
    assert_eq!(
        later.save(&name("emails"), b"new"),
        Err(SealedError::Locked)
    );
    assert_eq!(fs::read(c.key_file()).unwrap(), big);
    assert!(c.file("emails", SEALED_EXT).is_file());
    assert_eq!(guard.calls(), 0, "the guard never sees an oversized file");
}

#[test]
fn a_refused_wrap_saves_nothing() {
    let c = case();
    let store = c.reopen(&Mirror::refusing());
    assert_eq!(
        store.save(&name("emails"), b"a@b.c"),
        Err(SealedError::Locked)
    );
    assert!(!c.key_file().exists());
    assert!(!c.file("emails", SEALED_EXT).exists());
}

#[test]
fn a_locked_run_unlocks_when_the_guard_comes_back() {
    let c = case();
    c.store.save(&name("predict-en"), b"words").unwrap();
    let guard = Mirror::refusing();
    let later = c.reopen(&guard);
    assert_eq!(later.load(&name("predict-en")), Err(SealedError::Locked));
    guard.refuse.store(false, SeqCst);
    let got = later.load(&name("predict-en")).unwrap();
    assert_eq!(got.as_deref(), Some(&b"words"[..]));
}

#[test]
fn a_lost_key_file_sets_old_blobs_aside() {
    let c = case();
    c.store.save(&name("predict-en"), b"en").unwrap();
    c.store.save(&name("predict-fr"), b"fr").unwrap();
    fs::remove_file(c.key_file()).unwrap();
    let later = c.reopen(&Mirror::default());
    assert_eq!(later.load(&name("predict-en")), Err(SealedError::Damaged));
    later.save(&name("emails"), b"a@b.c").unwrap();
    assert!(c.key_file().is_file());
    assert_eq!(later.load(&name("predict-fr")), Err(SealedError::Damaged));
    assert!(later.load(&name("emails")).unwrap().is_some());
    assert_eq!(later.load(&name("predict-en")), Ok(None));
}

#[test]
fn a_key_file_lost_while_running_is_made_again_on_save() {
    let c = case();
    c.store.save(&name("predict-en"), b"en").unwrap();
    fs::remove_file(c.key_file()).unwrap();
    c.store.save(&name("predict-fr"), b"fr").unwrap();
    let later = c.reopen(&Mirror::default());
    let got = later.load(&name("predict-fr")).unwrap();
    assert_eq!(got.as_deref(), Some(&b"fr"[..]));
    assert_eq!(later.load(&name("predict-en")), Err(SealedError::Damaged));
}

#[test]
fn the_key_is_unwrapped_once_per_run() {
    let c = case();
    c.store.save(&name("predict-en"), b"en").unwrap();
    let guard = Mirror::default();
    let later = c.reopen(&guard);
    for _ in 0..3 {
        assert!(later.load(&name("predict-en")).unwrap().is_some());
        later.save(&name("predict-en"), b"en").unwrap();
    }
    assert_eq!(guard.calls(), 1);
}
