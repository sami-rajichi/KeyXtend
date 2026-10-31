//! The list form: every field round-trips, the head tells the round and the length, and a wrong version,
//! round, length, kind or trailing byte is refused.

use std::path::PathBuf;

use super::super::parts::owner;
use super::*;
use crate::fixture::{app, at, copied, date};

/// An entry with every field set, none at its default.
fn full() -> Entry {
    let mut e = copied("École أرسلت", &at(date(2026, 10, 31), 9, 5));
    (e.id, e.seq, e.kind, e.app) = (7, 9, ClipKind::Media, Some(app("vlc.exe")));
    (e.stored, e.shown, e.hash, e.parts, e.thumb) = (11, 12, 13, 2, true);
    e.files = Redacted::new(vec![PathBuf::from("C:\\v\\film.mkv")]);
    e
}

/// The list of `entries` in round `round` with counter `next`.
fn bytes(round: u8, next: u64, entries: &[&Entry]) -> Vec<u8> {
    frame(round, next, &records(entries))
}

#[test]
fn every_field_round_trips_in_the_folder_place() {
    let (a, mut b) = (full(), copied("b", &at(date(2026, 10, 30), 8, 0)));
    b.place = Place::Pinned;
    let list = decode(&bytes(1, 40, &[&a, &b]), Place::Pinned).unwrap();
    assert_eq!((list.round, list.next), (1, 40));
    let pinned = |mut e: Entry| {
        e.place = Place::Pinned;
        e
    };
    assert_eq!(list.entries, [pinned(a), b]);
    assert!(
        decode(&bytes(0, 0, &[]), Place::Pinned)
            .unwrap()
            .entries
            .is_empty()
    );
}

#[test]
fn the_head_tells_the_round_and_the_whole_length() {
    let whole = bytes(1, 3, &[&full()]);
    assert_eq!(head(&whole[..HEAD_LEN]), Ok((1, whole.len())));
    assert_eq!(head(&whole[..HEAD_LEN - 1]), Err(Bad));
    assert_eq!(turn(0), 1);
    assert_eq!(turn(1), 0);
}

#[test]
fn a_wrong_version_round_length_kind_or_tail_is_refused() {
    let whole = bytes(0, 3, &[&full()]);
    let changed = |at: usize, to: u8| {
        let mut b = whole.clone();
        b[at] = to;
        decode(&b, Place::Pinned).map(|_| ())
    };
    assert_eq!(changed(0, VERSION + 1), Err(Bad));
    assert_eq!(changed(1, ROUNDS), Err(Bad));
    assert_eq!(
        changed(2, whole[2] ^ 1),
        Err(Bad),
        "the total length is wrong"
    );
    let kind_at = HEAD_LEN + 2 * size_of::<u64>() + 2 * size_of::<u64>();
    assert_eq!(changed(kind_at, MEDIA + 1), Err(Bad));
    let mut tail = whole.clone();
    tail.push(0);
    assert_eq!(decode(&tail, Place::Pinned).map(|_| ()), Err(Bad));
    let short = &whole[..whole.len() - 1];
    assert_eq!(decode(short, Place::Pinned).map(|_| ()), Err(Bad));
}

#[test]
fn list_names_are_told_apart_from_entry_names() {
    assert_eq!(head_name().unwrap().as_str(), "list");
    let chunk = chunk_name(1, 12).unwrap();
    assert_eq!(chunk.as_str(), "list-1-12");
    assert!(is_list(&head_name().unwrap()) && is_list(&chunk));
    for other in ["lists", "ab-1", "ab-t", "listx-1"] {
        assert!(!is_list(&BlobName::new(other).unwrap()), "{other}");
    }
}

#[test]
fn aside_names_are_neither_list_nor_entry_names() {
    let aside = aside_name(3, 12).unwrap();
    assert_eq!(aside.as_str(), "aside-3-12");
    assert_eq!(aside_of(&aside), Some(3));
    assert!(!is_list(&aside) && owner(&aside).is_none());
    for other in ["aside", "aside-1", "aside-x-0", "list", "a-0"] {
        assert_eq!(aside_of(&BlobName::new(other).unwrap()), None, "{other}");
    }
}

#[test]
fn only_a_higher_first_byte_is_newer() {
    assert!(newer(&[VERSION + 1]));
    assert!(!newer(&[VERSION]) && !newer(&[0]) && !newer(&[]));
}
