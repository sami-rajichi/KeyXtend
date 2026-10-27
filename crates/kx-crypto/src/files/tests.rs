use super::*;
use crate::consts::{DAMAGED_EXT, MAX_BLOB_BYTES, OVERHEAD};
use crate::guard::GuardError;
use kx_test_support::tempdir::TempDir;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::SeqCst};
use zeroize::Zeroizing;

mod agree;
mod keys;

/// A test guard that wraps by reversing the bytes, refuses on cue and counts its calls.
#[derive(Clone, Default)]
struct Mirror {
    refuse: Arc<AtomicBool>,
    calls: Arc<AtomicUsize>,
}

impl Mirror {
    fn refusing() -> Self {
        let guard = Self::default();
        guard.refuse.store(true, SeqCst);
        guard
    }

    fn calls(&self) -> usize {
        self.calls.load(SeqCst)
    }

    fn answer(&self, error: GuardError, bytes: &[u8]) -> Result<Vec<u8>, GuardError> {
        self.calls.fetch_add(1, SeqCst);
        if self.refuse.load(SeqCst) {
            return Err(error);
        }
        Ok(bytes.iter().rev().copied().collect())
    }
}

impl KeyGuard for Mirror {
    fn wrap(&self, key: &[u8]) -> Result<Vec<u8>, GuardError> {
        self.answer(GuardError::Unavailable, key)
    }

    fn unwrap(&self, wrapped: &[u8]) -> Result<Zeroizing<Vec<u8>>, GuardError> {
        self.answer(GuardError::Refused, wrapped)
            .map(Zeroizing::new)
    }
}

/// A store over a data folder that does not exist yet, inside a temp folder removed at the end.
struct Case {
    _temp: TempDir,
    dir: PathBuf,
    guard: Mirror,
    store: SealedFiles,
}

fn case() -> Case {
    let temp = TempDir::new("crypto").unwrap();
    let dir = temp.path().join("data");
    let guard = Mirror::default();
    let store = SealedFiles::new(dir.clone(), Box::new(guard.clone()));
    Case {
        _temp: temp,
        dir,
        guard,
        store,
    }
}

impl Case {
    /// A store for a later run on the same folder, with `guard`.
    fn reopen(&self, guard: &Mirror) -> SealedFiles {
        SealedFiles::new(self.dir.clone(), Box::new(guard.clone()))
    }

    fn file(&self, name: &str, ext: &str) -> PathBuf {
        self.dir.join(format!("{name}.{ext}"))
    }

    fn key_file(&self) -> PathBuf {
        self.dir.join(KEY_FILE)
    }

    /// The file names in the data folder, sorted.
    fn names(&self) -> Vec<String> {
        let mut names: Vec<_> = fs::read_dir(&self.dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }
}

fn name(text: &str) -> BlobName {
    BlobName::new(text).unwrap()
}

#[test]
fn save_then_load_gives_the_bytes() {
    let c = case();
    c.store.save(&name("predict-en"), b"hello world").unwrap();
    let got = c.store.load(&name("predict-en")).unwrap();
    assert_eq!(got.as_deref(), Some(&b"hello world"[..]));
    let file = fs::read(c.file("predict-en", SEALED_EXT)).unwrap();
    assert!(!file.windows(5).any(|w| w == b"hello"));
}

#[test]
fn the_debug_text_shows_no_folder() {
    let c = case();
    assert!(!format!("{:?}", c.store).contains("data"));
}

#[test]
fn a_missing_blob_loads_as_none_without_the_guard() {
    let c = case();
    assert_eq!(c.store.load(&name("predict-en")), Ok(None));
    assert_eq!(c.guard.calls(), 0);
    assert!(!c.dir.exists());
}

#[test]
fn the_first_save_makes_the_folder_and_one_key() {
    let c = case();
    c.store.save(&name("emails"), b"a@b.c").unwrap();
    assert_eq!(c.names(), ["emails.sealed", KEY_FILE]);
}

#[test]
fn save_twice_keeps_only_the_latest() {
    let c = case();
    c.store.save(&name("predict-fr"), b"first").unwrap();
    c.store.save(&name("predict-fr"), b"second").unwrap();
    let got = c.store.load(&name("predict-fr")).unwrap();
    assert_eq!(got.as_deref(), Some(&b"second"[..]));
    assert_eq!(c.names(), ["predict-fr.sealed", KEY_FILE]);
}

#[test]
fn a_leftover_temp_file_does_not_block_a_save() {
    let c = case();
    c.store.save(&name("predict-ar"), b"one").unwrap();
    fs::write(c.dir.join("predict-ar.sealed.tmp"), b"half a file").unwrap();
    c.store.save(&name("predict-ar"), b"two").unwrap();
    assert_eq!(c.names(), ["predict-ar.sealed", KEY_FILE]);
}

#[test]
fn a_damaged_blob_is_set_aside_once_then_reads_as_none() {
    let c = case();
    c.store.save(&name("predict-en"), b"words").unwrap();
    let path = c.file("predict-en", SEALED_EXT);
    let mut bytes = fs::read(&path).unwrap();
    *bytes.last_mut().unwrap() ^= 1;
    fs::write(&path, &bytes).unwrap();
    assert_eq!(c.store.load(&name("predict-en")), Err(SealedError::Damaged));
    assert_eq!(c.names(), ["predict-en.damaged", KEY_FILE]);
    assert_eq!(fs::read(c.file("predict-en", DAMAGED_EXT)).unwrap(), bytes);
    assert_eq!(c.store.load(&name("predict-en")), Ok(None));
}

#[test]
fn a_newer_damaged_file_replaces_the_older_one() {
    let c = case();
    c.store.save(&name("emails"), b"x@y.z").unwrap();
    fs::write(c.file("emails", DAMAGED_EXT), b"older").unwrap();
    fs::write(c.file("emails", SEALED_EXT), b"newer").unwrap();
    assert_eq!(c.store.load(&name("emails")), Err(SealedError::Damaged));
    assert_eq!(fs::read(c.file("emails", DAMAGED_EXT)).unwrap(), b"newer");
}

#[test]
fn a_blob_copied_to_another_name_is_damaged() {
    let c = case();
    c.store.save(&name("predict-en"), b"words").unwrap();
    let from = c.file("predict-en", SEALED_EXT);
    fs::copy(from, c.file("predict-fr", SEALED_EXT)).unwrap();
    assert_eq!(c.store.load(&name("predict-fr")), Err(SealedError::Damaged));
    assert!(c.store.load(&name("predict-en")).unwrap().is_some());
}

#[test]
fn a_blob_that_cannot_be_read_is_io_and_stays() {
    let c = case();
    fs::create_dir_all(c.file("predict-en", SEALED_EXT)).unwrap();
    assert_eq!(c.store.load(&name("predict-en")), Err(SealedError::Io));
    assert!(c.file("predict-en", SEALED_EXT).is_dir());
}

#[test]
fn a_set_aside_that_fails_is_io_and_keeps_the_blob() {
    let c = case();
    c.store.save(&name("emails"), b"x@y.z").unwrap();
    let blocker = c.file("emails", DAMAGED_EXT);
    fs::create_dir_all(blocker.join("inside")).unwrap();
    fs::write(c.file("emails", SEALED_EXT), b"junk").unwrap();
    assert_eq!(c.store.load(&name("emails")), Err(SealedError::Io));
    assert!(c.file("emails", SEALED_EXT).is_file());
}

#[test]
fn an_oversized_blob_is_refused_on_save() {
    let c = case();
    let big = vec![0; MAX_BLOB_BYTES + 1];
    assert_eq!(
        c.store.save(&name("predict-en"), &big),
        Err(SealedError::Io)
    );
    assert!(!c.dir.exists());
}

#[test]
fn an_oversized_file_is_set_aside_on_load() {
    let c = case();
    c.store.save(&name("predict-en"), b"words").unwrap();
    let big = vec![0; MAX_BLOB_BYTES + OVERHEAD + 1];
    fs::write(c.file("predict-en", SEALED_EXT), &big).unwrap();
    assert_eq!(c.store.load(&name("predict-en")), Err(SealedError::Damaged));
    assert_eq!(c.names(), ["predict-en.damaged", KEY_FILE]);
}
