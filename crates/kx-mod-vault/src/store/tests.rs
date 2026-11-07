//! The store over the fake sealed store: a round trip, an empty start, a damaged blob, bytes that do not read set
//! aside, a newer version left alone, a locked or failing store, the format's own refusals and its pinned bytes.

use super::codec::{Bad, MARK, decode, encode};
use super::*;

use crate::entry::{Entry, Stamp};
use crate::fixture::{LIMITS, draft, field, own};
use kx_module_api::ports::{BlobName, EntryKind};
use kx_platform_fake::{FakeRandom, FakeSealedStore};

/// The time the tests save at.
const NOW: Stamp = Stamp::from_unix_ms(1_762_000_000_000);

/// The notebook blob's name, and the name of its set-aside copy.
fn names() -> (BlobName, BlobName) {
    (BlobName::new(BLOB).unwrap(), BlobName::new(ASIDE).unwrap())
}

/// A notebook of a Latin login, an Arabic person with joiners, a Wi-Fi with an own label, and a note filled once.
fn sample() -> Notebook {
    let drafts = [
        draft(
            "Bank",
            EntryKind::Login,
            vec![
                field("user", "alex", false),
                field("password", "s3cr3t!", true),
            ],
        ),
        draft(
            "عبد\u{200c}الله",
            EntryKind::Person,
            vec![field("full-name", "عبد\u{200c}الله الصغيّر", false)],
        ),
        draft(
            "Maison",
            EntryKind::Wifi,
            vec![
                own("Clé\u{200d}", "x7-y8", true),
                field("network", "Box", false),
            ],
        ),
        draft("Note", EntryKind::Note, vec![field("text", "a b", false)]),
    ];
    let (mut book, random) = (Notebook::default(), FakeRandom::new(3));
    let ids: Vec<u64> = drafts
        .into_iter()
        .map(|d| book.add(d, NOW, &random, &LIMITS).unwrap())
        .collect();
    book.use_at(ids[3], Stamp::from_unix_ms(NOW.unix_ms() + 1));
    book
}

/// A store over `sealed`.
fn store(sealed: &FakeSealedStore) -> Store {
    Store::new(Arc::new(sealed.clone()))
}

#[test]
fn the_notebook_comes_back_as_it_was_saved() {
    let sealed = FakeSealedStore::new();
    store(&sealed).save(&sample()).unwrap();
    let loaded = store(&sealed).load().unwrap();
    assert_eq!((loaded.book, loaded.damaged), (sample(), false));
    assert_eq!(sealed.saves(), 1);
}

#[test]
fn nothing_saved_starts_empty_and_says_nothing() {
    let loaded = store(&FakeSealedStore::new()).load().unwrap();
    assert!(loaded.book.is_empty() && !loaded.damaged);
}

#[test]
fn a_damaged_blob_set_aside_by_the_store_starts_empty_with_the_notice() {
    let sealed = FakeSealedStore::new();
    store(&sealed).save(&sample()).unwrap();
    sealed.fail_load(SealedError::Damaged);
    let loaded = store(&sealed).load().unwrap();
    assert!(loaded.book.is_empty() && loaded.damaged);
    assert_eq!(kx_module_api::keys::VAULT_DAMAGED, "notice.vault-damaged");
}

#[test]
fn bytes_that_do_not_read_are_set_aside_and_saves_go_on() {
    let (sealed, (blob, aside)) = (FakeSealedStore::new(), names());
    sealed.plant(&blob, b"not a notebook".to_vec());
    let mut store = store(&sealed);
    let loaded = store.load().unwrap();
    assert!(loaded.book.is_empty() && loaded.damaged);
    assert_eq!(sealed.blob(&aside), Some(b"not a notebook".to_vec()));
    store.save(&sample()).unwrap();
    assert!(sealed.blob(&blob).is_some_and(|b| b != b"not a notebook"));
}

#[test]
fn bytes_that_cannot_be_set_aside_are_never_written_over() {
    let (sealed, (blob, aside)) = (FakeSealedStore::new(), names());
    sealed.plant(&blob, b"not a notebook".to_vec());
    sealed.fail_save(SealedError::Io);
    let mut store = store(&sealed);
    assert!(store.load().unwrap().damaged);
    assert_eq!(store.save(&sample()), Err(StoreError::Held));
    assert_eq!(
        (sealed.blob(&blob), sealed.blob(&aside)),
        (Some(b"not a notebook".to_vec()), None)
    );
}

#[test]
fn a_newer_versions_notebook_is_refused_and_left_alone() {
    let (sealed, (blob, _)) = (FakeSealedStore::new(), names());
    let mut newer = encode(sample().entries()).to_vec();
    newer[MARK.len()] += 1;
    sealed.plant(&blob, newer.clone());
    assert_eq!(store(&sealed).load().err(), Some(StoreError::Newer));
    assert_eq!((sealed.blob(&blob), sealed.saves()), (Some(newer), 0));
}

#[test]
fn a_locked_or_failing_store_refuses_the_load_and_the_save() {
    let sealed = FakeSealedStore::new();
    for error in [SealedError::Locked, SealedError::Io] {
        sealed.fail_load(error);
        assert_eq!(store(&sealed).load().err(), Some(StoreError::Sealed(error)));
        sealed.fail_save(error);
        assert_eq!(
            store(&sealed).save(&sample()),
            Err(StoreError::Sealed(error))
        );
    }
}

#[test]
fn the_format_refuses_what_it_never_writes() {
    let good = encode(sample().entries()).to_vec();
    let mut twice = sample().entries().to_vec();
    twice.push(twice[0].clone());
    let cut = &good[..good.len() - 1];
    let longer = [good.as_slice(), &[0]].concat();
    for bad in [
        cut,
        longer.as_slice(),
        &encode(&twice),
        b"KXQF\x00\0\0\0\0",
        b"",
    ] {
        assert_eq!(decode(bad), Err(Bad::Damaged), "{} bytes", bad.len());
    }
}

#[test]
fn a_saved_buffer_is_sized_once() {
    let bytes = encode(sample().entries());
    assert_eq!(bytes.capacity(), bytes.len());
    assert_eq!(bytes.len(), sample().bytes());
}

/// Two entries with fixed ids and times: a login filled once with a table and an own label, and an Arabic note.
fn golden() -> Vec<Entry> {
    let login = draft(
        "Bank",
        EntryKind::Login,
        vec![field("user", "al", false), own("PIN", "12", true)],
    );
    let note = draft("ع", EntryKind::Note, vec![field("text", "x", false)]);
    let at = Stamp::from_unix_ms;
    vec![
        Entry::restored(0x0102_0304_0506_0708, login, at(1000), Some(at(2000))),
        Entry::restored(9, note, at(3), None),
    ]
}

/// The bytes `golden` saves as; a change here is a change of format, which needs a new version.
const GOLDEN: &[u8] = b"KXQF\x01\x02\0\0\0\
\x08\x07\x06\x05\x04\x03\x02\x01\x05\0\0\0login\xe8\x03\0\0\0\0\0\0\x01\xd0\x07\0\0\0\0\0\0\
\x04\0\0\0Bank\x02\0\0\0\
\x00\x04\0\0\0user\x00\x02\0\0\0al\
\x01\x03\0\0\0PIN\x01\x02\0\0\x0012\
\x09\0\0\0\0\0\0\0\x04\0\0\0note\x03\0\0\0\0\0\0\0\x00\x02\0\0\0\xd8\xb9\x01\0\0\0\
\x00\x04\0\0\0text\x00\x01\0\0\0x";

#[test]
fn the_format_is_pinned_byte_for_byte() {
    assert_eq!(encode(&golden()).as_slice(), GOLDEN);
    assert_eq!(decode(GOLDEN), Ok(golden()));
}

#[test]
fn an_own_label_of_blanks_does_not_read() {
    let blank = draft("Bank", EntryKind::Other, vec![own(" ", "1234", true)]);
    let bytes = encode(&[Entry::restored(1, blank, NOW, None)]);
    assert_eq!(decode(&bytes), Err(Bad::Damaged));
}
