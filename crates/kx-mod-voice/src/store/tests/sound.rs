//! The recordings on the shelf: a 10-minute recording is kept in blobs under the limits (edge 31), pieces are
//! regrouped into blobs, one past `voice_max_mb` or with history off writes nothing, and a failed or cut recording
//! leaves no blob.

use super::*;
use crate::VoiceSettings;
use kx_module_api::ports::{MAX_SOUND_BYTES, SAMPLE_BYTES, SOUND_HZ};

#[test]
fn edge_31_a_ten_minute_recording_is_kept_in_blobs_under_the_limits() {
    let settings = VoiceSettings::from_defaults().unwrap();
    let ten_min = usize::from(settings.record_max_s) * SOUND_HZ * SAMPLE_BYTES;
    let shelf = Arc::new(Mem::default());
    let mb = u64::from(settings.voice_max_mb) * MB;
    let (mut o, r) = (open(&shelf, MAX_BLOB_BYTES), rules(mb));
    keep(&mut o, 1, "dix minutes", &r);
    let sound = pieces(ten_min, MAX_SOUND_BYTES);
    assert!(voice(&mut o, 1, &sound, &r));
    let kept = o.history.entries()[0].sound.unwrap();
    assert_eq!(kept.bytes, u64::try_from(ten_min).unwrap());
    let blobs = u32::try_from(ten_min.div_ceil(MAX_BLOB_BYTES)).unwrap();
    assert!(blobs > 1, "more than one blob");
    assert_eq!(kept.blobs, blobs);
    assert!(shelf.sizes("sounds").iter().all(|n| *n <= MAX_BLOB_BYTES));
    let back = open(&shelf, MAX_BLOB_BYTES);
    let read = back.store.sound(&back.history.entries()[0]).unwrap();
    assert_eq!(joined(&read.unwrap()), joined(&sound));
}

#[test]
fn a_smaller_blob_splits_and_regroups_the_pieces() {
    let shelf = Arc::new(Mem::default());
    let (mut o, r) = (open(&shelf, 4_096), rules(MB));
    keep(&mut o, 1, "court", &r);
    assert!(voice(&mut o, 1, &pieces(10_000, 3_000), &r));
    assert_eq!(shelf.sizes("sounds"), [4_096, 4_096, 1_808]);
    let blobs = o.store.sound(&o.history.entries()[0]).unwrap().unwrap();
    assert_eq!(joined(&blobs), joined(&pieces(10_000, 3_000)));
}

#[test]
fn a_recording_past_voice_max_mb_or_with_history_off_writes_nothing() {
    let shelf = Arc::new(Mem::default());
    let (mut o, r) = (open(&shelf, MAX_BLOB_BYTES), rules(100));
    keep(&mut o, 1, "long", &r);
    assert!(!voice(&mut o, 1, &pieces(101, 50), &r));
    let off = Rules { voice: false, ..r };
    assert!(!voice(&mut o, 1, &pieces(10, 5), &off));
    assert!(shelf.names_in("sounds").is_empty());
    assert_eq!(o.history.entries()[0].sound, None);
}

#[test]
fn a_failed_recording_save_leaves_no_blob() {
    let shelf = Arc::new(Mem::default());
    let (mut o, r) = (open(&shelf, 4_096), rules(MB));
    keep(&mut o, 1, "raté", &r);
    shelf.saves(Some(1));
    let failed = o
        .store
        .keep_sound(&mut o.history, 1, &pieces(10_000, 4_096), &r);
    assert_eq!(failed, Err(SealedError::Io));
    assert!(shelf.names_in("sounds").is_empty());
    assert_eq!(o.history.entries()[0].sound, None);
}

#[test]
fn a_crash_s_leftovers_are_cleaned() {
    let shelf = Arc::new(Mem::default());
    let (mut o, r) = (open(&shelf, 4_096), rules(MB));
    keep(&mut o, 1, "coupé", &r);
    assert!(voice(&mut o, 1, &pieces(10_000, 4_096), &r));
    let id = o.history.entries()[0].id;
    shelf.lose("sounds", &format!("{id}-2"));
    shelf.plant("sounds", "90-0", b"no text claims it");
    shelf.plant("sounds", "odd", b"not a recording's");
    let back = open(&shelf, 4_096);
    assert_eq!(back.history.entries()[0].sound, None);
    assert!(back.history.next() > 90, "no id a blob still named is used");
    assert_eq!(shelf.names_in("sounds"), ["odd"]);
    let again = open(&shelf, 4_096);
    let sound = again.history.entries()[0].sound;
    assert_eq!(sound, None, "the text was written again");
}
