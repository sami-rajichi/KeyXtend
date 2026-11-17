//! The cloud keys: one sealed blob per service in `keys`, removed with its blob, refused when a header could not carry
//! it, never in a print or an error, and never read or written in a hidden run (edge 11). Every key here is made up at
//! run time, so no file holds one.

use super::*;
use crate::store::mem::Mem;
use kx_module_api::ports::{MAX_KEY_BYTES, Room};
use std::sync::atomic::{AtomicUsize, Ordering};

/// A key made up at run time from `seed`: 48 letters and digits, as a service's key looks.
fn made_key(seed: u8) -> String {
    let alphabet: Vec<char> = ('a'..='z').chain('0'..='9').collect();
    (0..48_usize)
        .map(|i| alphabet[(i * 7 + usize::from(seed)) % alphabet.len()])
        .collect()
}

/// `text` as a secret.
fn secret(text: &str) -> Secret {
    Redacted::new(Zeroizing::new(text.to_owned()))
}

/// Keys over a fresh shelf in memory, and the shelf.
fn keys() -> (Keys, Arc<Mem>) {
    let shelf = Arc::new(Mem::default());
    (Keys::new(shelf.clone(), false), shelf)
}

/// A shelf that counts every call and holds nothing, for the hidden run.
#[derive(Default)]
struct Spy(AtomicUsize);

impl Spy {
    fn touch(&self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

impl SealedShelf for Spy {
    fn load(&self, _: &BlobName, _: &BlobName) -> Result<Option<Vec<u8>>, SealedError> {
        self.touch();
        Ok(None)
    }
    fn save(&self, _: &BlobName, _: &BlobName, _: &[u8]) -> Result<(), SealedError> {
        self.touch();
        Ok(())
    }
    fn remove(&self, _: &BlobName, _: &BlobName) -> Result<(), SealedError> {
        self.touch();
        Ok(())
    }
    fn drop_folder(&self, _: &BlobName) -> Result<(), SealedError> {
        self.touch();
        Ok(())
    }
    fn folders(&self) -> Result<Vec<BlobName>, SealedError> {
        self.touch();
        Ok(Vec::new())
    }
    fn names(&self, _: &BlobName) -> Result<Vec<BlobName>, SealedError> {
        self.touch();
        Ok(Vec::new())
    }
    fn room(&self) -> Result<Room, SealedError> {
        self.touch();
        Err(SealedError::Io)
    }
}

#[test]
fn each_service_seals_its_own_key_in_the_keys_folder() {
    let (keys, shelf) = keys();
    let (groq, custom) = (made_key(1), made_key(2));
    keys.set("groq", secret(&groq)).unwrap();
    keys.set("custom", secret(&custom)).unwrap();
    assert_eq!(
        shelf.names_in(KEYS),
        ["custom", "groq"],
        "one blob per service"
    );
    assert_eq!(shelf.all().len(), 2, "nothing outside the keys folder");
    let read = |id| keys.get(id).unwrap().map(|k| k.expose().to_string());
    assert_eq!(
        (read("groq"), read("custom")),
        (Some(groq), Some(custom.clone()))
    );
    assert_eq!(read("openai"), None, "no key set");
    let next = made_key(3);
    keys.set("custom", secret(&next)).unwrap();
    assert_eq!(
        read("custom"),
        Some(next),
        "a new key takes the old one's place"
    );
    assert!(keys.has("groq").unwrap() && !keys.has("openai").unwrap());
}

#[test]
fn removing_a_key_deletes_its_blob_alone() {
    let (keys, shelf) = keys();
    keys.set("groq", secret(&made_key(4))).unwrap();
    keys.set("custom", secret(&made_key(5))).unwrap();
    keys.remove("groq").unwrap();
    assert_eq!(shelf.names_in(KEYS), ["custom"]);
    assert!(keys.get("groq").unwrap().is_none());
    assert_eq!(keys.remove("groq"), Ok(()), "a missing key is no error");
}

#[test]
fn a_hidden_run_reads_and_writes_no_key() {
    let spy = Arc::new(Spy::default());
    let keys = Keys::new(spy.clone(), true);
    assert_eq!(keys.get("groq").map(|k| k.is_none()), Ok(true));
    assert_eq!(keys.has("groq"), Ok(false));
    assert_eq!(
        keys.set("groq", secret(&made_key(6))),
        Err(KeyError::Hidden)
    );
    assert_eq!(keys.remove("groq"), Err(KeyError::Hidden));
    assert_eq!(
        spy.0.load(Ordering::SeqCst),
        0,
        "the shelf is never touched"
    );
}

#[test]
fn a_key_no_header_could_carry_is_refused_and_nothing_is_saved() {
    let (keys, shelf) = keys();
    let key = made_key(7);
    let long = "k".repeat(MAX_KEY_BYTES + 1);
    for bad in [
        String::new(),
        format!("{key} x"),
        format!("{key}\n"),
        format!("{key}é"),
        long,
    ] {
        assert_eq!(
            keys.set("groq", secret(&bad)),
            Err(KeyError::Unfit),
            "{} bytes",
            bad.len()
        );
    }
    assert!(shelf.is_empty());
}

#[test]
fn an_odd_service_id_or_a_failed_shelf_is_an_error() {
    let (keys, shelf) = keys();
    for odd in ["Groq", "a/b", "nul", ""] {
        let refused = Err(KeyError::Shelf(SealedError::BadName));
        assert_eq!(keys.set(odd, secret(&made_key(8))), refused, "{odd:?}");
    }
    shelf.saves(Some(0));
    assert_eq!(
        keys.set("groq", secret(&made_key(9))),
        Err(KeyError::Shelf(SealedError::Io))
    );
    assert!(shelf.is_empty());
}

#[test]
fn a_blob_that_is_no_key_reads_as_damaged() {
    let (keys, shelf) = keys();
    let damaged = Err(KeyError::Shelf(SealedError::Damaged));
    for bytes in [vec![0xff, 0xfe], b"two words".to_vec(), Vec::new()] {
        shelf.plant(KEYS, "groq", &bytes);
        assert_eq!(keys.get("groq").map(|k| k.is_some()), damaged, "{bytes:?}");
    }
}

#[test]
fn no_key_reaches_a_print_or_an_error() {
    let (keys, _) = keys();
    let key = made_key(10);
    keys.set("groq", secret(&key)).unwrap();
    let got = keys.get("groq").unwrap().unwrap();
    let errors = [
        KeyError::Hidden,
        KeyError::Unfit,
        KeyError::Shelf(SealedError::Damaged),
    ];
    let shown = format!(
        "{keys:?}{got:?}{got}{errors:?}{}",
        errors.map(|e| e.to_string()).join(" ")
    );
    assert!(!shown.contains(&key), "{shown}");
    assert!(shown.contains("Keys"), "{shown}");
}
