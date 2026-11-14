//! The store over a memory shelf (plan task 18): texts and pins come back after a reopen; a newer file is never read
//! or written over; a damaged text goes and is counted; failed writes are tried again; Delete all removes what it
//! asked about; nothing prints content. The recordings and the numbers have their own files.

use super::mem::Mem;
use super::*;
use crate::counts::{Board, Checked, Score};
use crate::flow::{Count, Kept};
use crate::history::Stamp;
use kx_module_api::ports::{
    CardId, Date, Fingerprint, Hundredths, MAX_BLOB_BYTES, MB, SpeechDone, Verdict,
};
use kx_module_api::{Redacted, Secret, Zeroizing};

mod figures;
mod props;
mod sound;

/// A row of the models table.
const ROW: &str = "parakeet-v3-q8";

fn secret(text: &str) -> Secret {
    Redacted::new(Zeroizing::new(text.to_owned()))
}

fn kept(take: u32, text: &str) -> Kept {
    let model = Redacted::new(ROW.into());
    Kept {
        take,
        text: secret(text),
        lang: "ar".into(),
        model,
    }
}

/// History on, 50 texts, Keep recordings with `voice` bytes.
fn rules(voice: u64) -> Rules {
    Rules {
        texts: true,
        max: 50,
        voice: true,
        voice_bytes: voice,
    }
}

fn at(n: u32) -> Stamp {
    Stamp {
        unix_ms: 1_763_000_000_000 + i64::from(n),
        offset_min: 60,
    }
}

/// A store over `shelf` with blobs of `blob` bytes at most.
fn open(shelf: &Arc<Mem>, blob: usize) -> Opened {
    Store::open(shelf.clone(), blob).unwrap()
}

/// `len` bytes counting up, handed over in pieces of `piece`.
fn pieces(len: usize, piece: usize) -> Vec<SecretBytes> {
    let bytes: Vec<u8> = (0..len).map(|n| u8::try_from(n % 251).unwrap()).collect();
    let one = |c: &[u8]| Redacted::new(Zeroizing::new(c.to_vec()));
    bytes.chunks(piece).map(one).collect()
}

/// The bytes of `blobs`, joined.
fn joined(blobs: &[SecretBytes]) -> Vec<u8> {
    blobs.iter().flat_map(|b| b.expose().to_vec()).collect()
}

/// Keeps `text` for `take` and writes it.
fn keep(o: &mut Opened, take: u32, text: &str, r: &Rules) {
    let edit = o.history.keep(kept(take, text), at(take), r);
    o.store.apply(&o.history, edit).unwrap();
}

/// Keeps `pieces` for `take`; true when kept.
fn voice(o: &mut Opened, take: u32, pieces: &[SecretBytes], r: &Rules) -> bool {
    o.store.keep_sound(&mut o.history, take, pieces, r).unwrap()
}

fn day(n: u8) -> Date {
    Date::new(2026, 11, n).unwrap()
}

fn print(n: u8) -> Fingerprint {
    Fingerprint::new([n; Fingerprint::BYTES])
}

#[test]
fn texts_pins_and_recordings_come_back_after_a_reopen() {
    let shelf = Arc::new(Mem::default());
    let (mut o, r) = (open(&shelf, MAX_BLOB_BYTES), rules(MB));
    for (take, text) in [(1, "مرحبا"), (2, "شكرا"), (3, "إلى اللقاء")] {
        keep(&mut o, take, text, &r);
    }
    assert!(voice(&mut o, 3, &pieces(5_000, 1_000), &r));
    let first = o.history.entries().last().unwrap().id;
    let edit = o.history.pin(&[first]);
    o.store.apply(&o.history, edit).unwrap();
    let back = open(&shelf, MAX_BLOB_BYTES);
    assert_eq!(back.history.entries(), o.history.entries());
    assert!(back.history.next() >= o.history.next());
    assert_eq!(back.damaged, 0);
    let voiced = back.history.entries().iter().find(|e| e.sound.is_some());
    let sound = back.store.sound(voiced.unwrap()).unwrap().unwrap();
    assert_eq!(joined(&sound), joined(&pieces(5_000, 1_000)));
}

#[test]
fn a_newer_file_is_never_read_or_written_over() {
    for (folder, name, mark) in [
        ("texts", "1", text::MARK),
        ("numbers", "counts", numbers::COUNTS_MARK),
    ] {
        let shelf = Arc::new(Mem::default());
        let mut newer = mark.to_vec();
        newer.extend([u8::MAX, 9, 9]);
        shelf.plant(folder, name, &newer);
        shelf.plant("sounds", "7-0", b"orphan");
        let before = shelf.all();
        let opened = Store::open(shelf.clone(), MAX_BLOB_BYTES);
        assert!(matches!(opened, Err(StoreError::Newer)));
        assert_eq!(shelf.all(), before);
    }
}

#[test]
fn a_damaged_text_goes_with_its_recording_and_is_counted() {
    let shelf = Arc::new(Mem::default());
    let (mut o, r) = (open(&shelf, MAX_BLOB_BYTES), rules(MB));
    keep(&mut o, 1, "bon", &r);
    shelf.plant("texts", "4", b"not a text");
    shelf.plant("sounds", "4-0", b"its sound");
    shelf.plant("texts", "5", b"sealed badly");
    shelf.damage("texts", "5");
    let back = open(&shelf, MAX_BLOB_BYTES);
    assert_eq!(back.damaged, 2);
    assert_eq!(back.history.len(), 1);
    let kept = o.history.entries()[0].id.to_string();
    assert_eq!(shelf.names_in("texts"), [kept]);
    assert!(shelf.names_in("sounds").is_empty());
    assert!(back.history.next() > 5, "no id a blob still named is used");
}

#[test]
fn a_failed_write_is_tried_again_with_the_next_change() {
    let shelf = Arc::new(Mem::default());
    let (mut o, r) = (open(&shelf, MAX_BLOB_BYTES), rules(MB));
    shelf.saves(Some(0));
    let edit = o.history.keep(kept(1, "plus tard"), at(1), &r);
    assert_eq!(o.store.apply(&o.history, edit), Err(SealedError::Io));
    assert!(shelf.names_in("texts").is_empty());
    shelf.saves(None);
    assert_eq!(o.store.apply(&o.history, Edit::default()), Ok(()));
    assert_eq!(open(&shelf, MAX_BLOB_BYTES).history.len(), 1);
}

#[test]
fn a_crash_between_the_writes_and_the_removes_loses_no_pin() {
    let shelf = Arc::new(Mem::default());
    let (mut o, r) = (
        open(&shelf, MAX_BLOB_BYTES),
        Rules {
            max: 2,
            ..rules(MB)
        },
    );
    keep(&mut o, 1, "épinglé", &r);
    assert!(voice(&mut o, 1, &pieces(300, 100), &r));
    let pinned = o.history.entries()[0].id;
    let edit = o.history.pin(&[pinned]);
    o.store.apply(&o.history, edit).unwrap();
    keep(&mut o, 2, "deux", &r);
    keep(&mut o, 3, "trois", &r);
    shelf.stuck(true);
    let edit = o.history.keep(kept(4, "quatre"), at(4), &r);
    assert_eq!(edit.gone.len(), 1, "the oldest unpinned goes");
    assert_eq!(o.store.apply(&o.history, edit), Err(SealedError::Io));
    shelf.stuck(false);
    let mut back = open(&shelf, MAX_BLOB_BYTES);
    let first = &back.history.entries()[0];
    assert!(first.id == pinned && first.pinned && first.sound.is_some());
    assert!(back.store.sound(first).unwrap().is_some());
    assert_eq!(back.history.unpinned(), 3, "a leftover, never a loss");
    keep(&mut back, 5, "cinq", &r);
    assert_eq!(shelf.names_in("texts").len(), 3);
}

#[test]
fn a_text_too_large_for_a_blob_is_refused_once_and_not_tried_again() {
    let shelf = Arc::new(Mem::default());
    let (mut o, r) = (open(&shelf, 64), rules(MB));
    let edit = o.history.keep(kept(1, &"mot ".repeat(40)), at(1), &r);
    assert_eq!(o.store.apply(&o.history, edit), Err(SealedError::Io));
    assert_eq!(o.store.apply(&o.history, Edit::default()), Ok(()));
    assert!(shelf.names_in("texts").is_empty());
}

#[test]
fn delete_all_removes_the_texts_and_recordings_it_asked_about() {
    let shelf = Arc::new(Mem::default());
    let (mut o, r) = (open(&shelf, MAX_BLOB_BYTES), rules(MB));
    keep(&mut o, 1, "garde", &r);
    let pinned = o.history.entries()[0].id;
    let edit = o.history.pin(&[pinned]);
    o.store.apply(&o.history, edit).unwrap();
    keep(&mut o, 2, "va", &r);
    assert!(voice(&mut o, 2, &pieces(100, 50), &r));
    let asked = o.history.ask_all().unwrap();
    let edit = o.history.delete_all(asked);
    o.store.apply(&o.history, edit).unwrap();
    assert_eq!(shelf.names_in("texts"), [pinned.to_string()]);
    assert!(shelf.names_in("sounds").is_empty());
}

#[test]
fn a_store_prints_no_content() {
    let shelf = Arc::new(Mem::default());
    let (mut o, r) = (open(&shelf, MAX_BLOB_BYTES), rules(MB));
    keep(&mut o, 1, "secret du jour", &r);
    let shown = format!("{o:?}");
    assert!(!shown.contains("secret du jour") && !shown.contains(ROW));
}
