//! Property tests: random, changed or cut bytes never panic a reader; random entries round-trip through a list; and a
//! random list on the shelf, whole or split, loads with nothing of it kept.

use std::path::PathBuf;
use std::sync::Arc;

use kx_module_api::Redacted;
use kx_module_api::ports::{AppRef, ClipKind, Pixels, RGBA_BYTES};
use kx_platform_fake::FakeShelf;
use proptest::collection::vec;
use proptest::option;
use proptest::prelude::*;
use proptest::sample::Index;

use super::*;
use crate::entry::Stamp;
use crate::fixture::{STORE, date, files, text};

/// Random cases tried per property.
const CASES: u32 = 256;
/// The longest random byte run.
const MAX_BYTES: usize = 512;
/// The most entries in a random list.
const MAX_ENTRIES: usize = 5;
/// The most bytes changed in a sound form.
const MAX_EDITS: usize = 8;
/// The most paths in a random entry.
const MAX_FILES: usize = 3;
/// A random text: any characters but a line break, up to 12.
const TEXT: &str = ".{0,12}";
/// The longest thumbnail side the readers allow.
const SIDE: u32 = 4;
/// Every kind, for a random pick.
const KINDS: [ClipKind; 5] = [
    ClipKind::Text,
    ClipKind::Link,
    ClipKind::Image,
    ClipKind::File,
    ClipKind::Media,
];

/// A random entry in the pinned folder.
fn entry() -> impl Strategy<Value = Entry> {
    let ids = (
        any::<u64>(),
        any::<u64>(),
        0..KINDS.len(),
        any::<i64>(),
        any::<i16>(),
    );
    let texts = (
        option::of((TEXT, TEXT)),
        TEXT,
        TEXT,
        vec(TEXT, 0..MAX_FILES),
    );
    let sizes = (
        any::<u64>(),
        any::<u64>(),
        any::<u64>(),
        any::<u16>(),
        any::<bool>(),
    );
    (ids, texts, sizes).prop_map(built)
}

/// A random entry's numbers, kind and time.
type Ids = (u64, u64, usize, i64, i16);
/// A random entry's app, title, search text and paths.
type Texts = (Option<(String, String)>, String, String, Vec<String>);
/// A random entry's sizes, hash, blob count and thumbnail flag.
type Sizes = (u64, u64, u64, u16, bool);

/// The pinned entry these random fields make.
fn built((ids, texts, sizes): (Ids, Texts, Sizes)) -> Entry {
    let (id, seq, kind, unix_ms, offset_min) = ids;
    let (app, title, search, files) = texts;
    let (stored, shown, hash, parts, thumb) = sizes;
    Entry {
        id,
        seq,
        kind: KINDS[kind],
        place: Place::Pinned,
        at: Stamp {
            unix_ms,
            offset_min,
        },
        app: app.map(|(id, name)| AppRef { id, name }),
        stored,
        shown,
        title: Redacted::new(title),
        search: Redacted::new(search),
        hash,
        parts,
        thumb,
        files: Redacted::new(files.into_iter().map(PathBuf::from).collect()),
    }
}

/// A grey thumbnail at the readers' longest side, one pixel high.
fn grey() -> Pixels {
    let n = usize::try_from(SIDE).unwrap() * RGBA_BYTES;
    Pixels::new(SIDE, 1, vec![u8::MAX / 2; n], SIDE).unwrap()
}

/// `bytes` with each edit's byte written at its index, then cut at `cut`.
fn damage(mut bytes: Vec<u8>, edits: &[(Index, u8)], cut: Index) -> Vec<u8> {
    for (at, to) in edits {
        if !bytes.is_empty() {
            let i = at.index(bytes.len());
            bytes[i] = *to;
        }
    }
    bytes.truncate(cut.index(bytes.len() + 1));
    bytes
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn random_bytes_never_panic_a_reader(bytes in vec(any::<u8>(), 0..MAX_BYTES)) {
        let _ = list::head(&bytes);
        let _ = list::decode(&bytes, Place::Pinned);
        let _ = parts::decode(&bytes);
        let _ = parts::decode_thumb(&bytes, SIDE);
    }

    #[test]
    fn changed_or_cut_forms_never_panic_a_reader(
        entries in vec(entry(), 0..MAX_ENTRIES),
        edits in vec((any::<Index>(), any::<u8>()), 0..MAX_EDITS),
        cut in any::<Index>(),
    ) {
        let refs: Vec<&Entry> = entries.iter().collect();
        let list = damage(list::frame(1, 0, &list::records(&refs)), &edits, cut);
        let _ = list::decode(&list, Place::Pinned);
        let content = parts::encode(&[text("a"), files(&["b", "c"])]);
        let _ = parts::decode(&damage(content, &edits, cut));
        let thumb = parts::encode_thumb(&grey());
        let _ = parts::decode_thumb(&damage(thumb, &edits, cut), SIDE);
    }

    #[test]
    fn random_entries_round_trip_through_a_list(
        entries in vec(entry(), 0..MAX_ENTRIES),
        round in 0..2u8,
        next in any::<u64>(),
    ) {
        let refs: Vec<&Entry> = entries.iter().collect();
        let back = list::decode(&list::frame(round, next, &list::records(&refs)), Place::Pinned);
        let back = back.map(|l| (l.round, l.next, l.entries));
        prop_assert_eq!(back, Ok((round, next, entries)));
    }

    #[test]
    fn a_random_list_on_the_shelf_loads_with_nothing_kept(
        records in vec(any::<u8>(), 0..MAX_BYTES),
        raw in any::<bool>(),
        split in any::<Index>(),
        round in 0..2u8,
    ) {
        let shelf = FakeShelf::new();
        let day = folder_of(Place::Day(date(2026, 10, 31))).unwrap();
        let whole = if raw { records } else { list::frame(round, 0, &records) };
        let (head, rest) = whole.split_at(split.index(whole.len() + 1));
        let name = |n| list::chunk_name(round, n).unwrap();
        shelf.plant(&day, &list::head_name().unwrap(), head.to_vec());
        shelf.plant(&day, &name(1), rest.to_vec());
        let newer = head.first().is_some_and(|v| *v > list::VERSION);
        let loaded = Store::new(Arc::new(shelf.clone()), STORE).load();
        let loaded = loaded.map(|l| (l.history.is_empty(), l.aside.len() <= 1));
        let want = if newer { Err(StoreError::Newer) } else { Ok((true, true)) };
        prop_assert_eq!(loaded, want);
    }
}
