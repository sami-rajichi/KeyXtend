//! The store over a fake shelf: lists round-trip, hand deletions and damage are survived, deletes leave nothing, blobs
//! no list names are cleaned, and only changed lists save.

mod aside;
mod big;
mod cut;
mod fault;
mod held;
mod moves;

use std::sync::Arc;

use kx_module_api::ports::{LocalTime, Pixels, RGBA_BYTES};
use kx_platform_fake::{FakeShelf, ShelfCall};

use super::*;
use crate::entry::Stamp;
use crate::fixture::{STORE, app, at, copied, date, text};

/// A blob size so small that a short text takes several blobs, and a list more.
const SMALL: StoreConfig = StoreConfig {
    blob_bytes: 24,
    ..STORE
};

/// Test day `n`: Monday 2026-10-26 is day 1.
fn day(n: u8) -> Date {
    let start = date(2026, 10, 25).day_number();
    Date::from_day_number(start + i64::from(n)).unwrap()
}

/// The place of test day `n`.
fn on(n: u8) -> Place {
    Place::Day(day(n))
}

/// Noon on test day `n`.
fn noon(n: u8) -> LocalTime {
    at(day(n), 12, 0)
}

/// A store over `shelf`.
fn open(shelf: &FakeShelf) -> Store {
    Store::new(Arc::new(shelf.clone()), STORE)
}

/// What a fresh store finds on `shelf`.
fn reload(shelf: &FakeShelf) -> Loaded {
    open(shelf).load().unwrap()
}

/// Copies `s` at `now` into `h`, saving its content in `store`; gives its id.
fn keep(h: &mut History, store: &mut Store, s: &str, now: &LocalTime) -> u64 {
    let id = h.add(copied(s, now));
    let n = store.put(h.get(id).unwrap(), &[text(s)]).unwrap();
    h.update(id, |e| e.parts = n);
    id
}

/// The folder of `place`.
fn folder(place: Place) -> BlobName {
    folder_of(place).unwrap()
}

/// The blob names in `place`'s folder.
fn names(shelf: &FakeShelf, place: Place) -> Vec<String> {
    let names = shelf.names(&folder(place)).unwrap();
    names.iter().map(|n| n.as_str().to_owned()).collect()
}

/// The ids of `h` in order.
fn ids(h: &History) -> Vec<u64> {
    h.entries().iter().map(|e| e.id).collect()
}

/// A grey thumbnail `side` pixels wide and one high.
fn grey(side: u32) -> Pixels {
    let n = usize::try_from(side).unwrap() * RGBA_BYTES;
    Pixels::new(side, 1, vec![128; n], side).unwrap()
}

#[test]
fn lists_round_trip_with_the_counter_times_and_places() {
    let shelf = FakeShelf::new();
    let (mut s, mut h) = (open(&shelf), History::default());
    let a = keep(&mut h, &mut s, "a", &noon(1));
    let b = keep(&mut h, &mut s, "b", &noon(2));
    keep(&mut h, &mut s, "c", &noon(2));
    h.pin(a);
    assert_eq!(s.moved(&h, a, on(1)), Ok(true));
    h.update(b, |e| e.app = Some(app("notepad.exe")));
    s.save(&h).unwrap();
    let loaded = reload(&shelf);
    assert!(loaded.aside.is_empty());
    assert_eq!(loaded.history, h);
    assert_eq!(loaded.history.next(), h.next());
    let back = loaded.history.get(b).unwrap();
    assert_eq!((back.at, back.place), (Stamp::of(&noon(2)), on(2)));
}

#[test]
fn a_hand_deleted_day_leaves_the_list() {
    let shelf = FakeShelf::new();
    let (mut s, mut h) = (open(&shelf), History::default());
    let a = keep(&mut h, &mut s, "a", &noon(1));
    let b = keep(&mut h, &mut s, "b", &noon(2));
    s.save(&h).unwrap();
    shelf.delete_by_hand(&folder(on(1)));
    assert_eq!(s.missing(&h), Ok(vec![a]), "seen while running");
    assert_eq!(s.parts(h.get(a).unwrap()), Ok(None));
    let loaded = reload(&shelf);
    assert_eq!(ids(&loaded.history), [b]);
    assert!(
        loaded.aside.is_empty(),
        "a folder gone by hand is no damage"
    );
}

#[test]
fn a_locked_or_unlisted_shelf_fails_the_load() {
    let shelf = FakeShelf::new();
    let (mut s, mut h) = (open(&shelf), History::default());
    keep(&mut h, &mut s, "a", &noon(1));
    s.save(&h).unwrap();
    shelf.fail(ShelfCall::Load, SealedError::Locked);
    assert_eq!(
        open(&shelf).load().map(|_| ()),
        Err(StoreError::Shelf(SealedError::Locked))
    );
    shelf.fail(ShelfCall::Folders, SealedError::Io);
    assert_eq!(
        open(&shelf).load().map(|_| ()),
        Err(StoreError::Shelf(SealedError::Io))
    );
    assert_eq!(ids(&reload(&shelf).history), ids(&h), "nothing was lost");
}

#[test]
fn deletes_remove_every_part_and_the_thumbnail() {
    let shelf = FakeShelf::new();
    let (mut s, mut h) = (
        Store::new(Arc::new(shelf.clone()), SMALL),
        History::default(),
    );
    let id = keep(
        &mut h,
        &mut s,
        "a text longer than one small blob",
        &noon(1),
    );
    let other = keep(&mut h, &mut s, "b", &noon(1));
    assert!(h.get(id).unwrap().parts > 1);
    s.put_thumb(h.get(id).unwrap(), &grey(2)).unwrap();
    h.update(id, |e| e.thumb = true);
    s.save(&h).unwrap();
    s.delete(h.get(id).unwrap()).unwrap();
    let mine = |n: &String| n.starts_with(&format!("{id:x}-"));
    assert!(!names(&shelf, on(1)).iter().any(mine));
    h.remove(id);
    s.save(&h).unwrap();
    assert_eq!(ids(&reload(&shelf).history), [other]);
}

#[test]
fn only_changed_lists_save_and_an_emptied_day_drops() {
    let shelf = FakeShelf::new();
    let (mut s, mut h) = (open(&shelf), History::default());
    let a = keep(&mut h, &mut s, "a", &noon(1));
    let b = keep(&mut h, &mut s, "b", &noon(2));
    s.save(&h).unwrap();
    let saves = shelf.calls(ShelfCall::Save);
    s.save(&h).unwrap();
    assert_eq!(shelf.calls(ShelfCall::Save), saves, "nothing changed");
    h.raise(b, &noon(2));
    s.save(&h).unwrap();
    assert_eq!(shelf.calls(ShelfCall::Save), saves + 1, "one list");
    s.delete(h.get(a).unwrap()).unwrap();
    h.remove(a);
    s.save(&h).unwrap();
    assert_eq!(shelf.folders(), Ok(vec![folder(on(2))]));
}

#[test]
fn days_past_keep_days_drop_whole_and_pinned_stays() {
    let shelf = FakeShelf::new();
    let (mut s, mut h) = (open(&shelf), History::default());
    let a = keep(&mut h, &mut s, "a", &noon(1));
    keep(&mut h, &mut s, "b", &noon(1));
    keep(&mut h, &mut s, "c", &noon(2));
    h.pin(a);
    s.moved(&h, a, on(1)).unwrap();
    s.save(&h).unwrap();
    s.drop_past(day(8), 7).unwrap();
    let left = vec![folder(on(2)), folder(Place::Pinned)];
    assert_eq!(shelf.folders(), Ok(left), "day 1 is 7 days before day 8");
}

#[test]
fn thumbnails_round_trip_within_their_side() {
    let shelf = FakeShelf::new();
    let (mut s, mut h) = (open(&shelf), History::default());
    let id = keep(&mut h, &mut s, "a", &noon(1));
    let entry = h.get(id).unwrap().clone();
    assert_eq!(s.thumb(&entry), Ok(None));
    s.put_thumb(&entry, &grey(STORE.thumb_px)).unwrap();
    assert_eq!(s.thumb(&entry), Ok(Some(grey(STORE.thumb_px))));
    let smaller = StoreConfig {
        thumb_px: STORE.thumb_px - 1,
        ..STORE
    };
    let s = Store::new(Arc::new(shelf.clone()), smaller);
    assert_eq!(s.thumb(&entry), Err(SealedError::Damaged), "drawn again");
}

#[test]
fn a_load_removes_blobs_no_list_names_and_keeps_others() {
    let shelf = FakeShelf::new();
    let (mut s, mut h) = (open(&shelf), History::default());
    let a = keep(&mut h, &mut s, "a", &noon(1));
    s.save(&h).unwrap();
    let blob = |n: &str| BlobName::new(n).unwrap();
    for n in ["ff-0", "ff-t", "list-1-3", "notes"] {
        shelf.plant(&folder(on(1)), &blob(n), vec![1]);
    }
    shelf.plant(&folder(on(3)), &blob("5-0"), vec![1]);
    let loaded = reload(&shelf);
    assert_eq!(ids(&loaded.history), [a]);
    let kept = [format!("{a:x}-0"), "list".into(), "notes".into()];
    assert_eq!(names(&shelf, on(1)), kept, "a name not ours stays");
    assert!(names(&shelf, on(3)).is_empty(), "a folder with no list");
}

#[test]
fn an_entry_missing_a_blob_leaves_and_a_lost_thumbnail_is_redrawn() {
    let shelf = FakeShelf::new();
    let (mut s, mut h) = (
        Store::new(Arc::new(shelf.clone()), SMALL),
        History::default(),
    );
    let a = keep(&mut h, &mut s, "a text that takes two blobs", &noon(1));
    let b = keep(&mut h, &mut s, "b", &noon(1));
    s.put_thumb(h.get(b).unwrap(), &grey(1)).unwrap();
    h.update(b, |e| e.thumb = true);
    s.save(&h).unwrap();
    let day1 = folder(on(1));
    shelf.remove(&day1, &part_name(a, 1).unwrap()).unwrap();
    shelf.remove(&day1, &thumb_name(b).unwrap()).unwrap();
    let mut s = open(&shelf);
    let loaded = s.load().unwrap();
    assert_eq!(ids(&loaded.history), [b]);
    assert!(!loaded.history.get(b).unwrap().thumb);
    assert!(
        names(&shelf, on(1))
            .iter()
            .all(|n| !n.starts_with(&format!("{a:x}-")))
    );
    s.save(&loaded.history).unwrap();
    assert_eq!(
        reload(&shelf).history,
        loaded.history,
        "the list saved again"
    );
}

#[test]
fn a_failed_list_skips_only_its_folder() {
    let shelf = FakeShelf::new();
    let (mut s, mut h) = (open(&shelf), History::default());
    keep(&mut h, &mut s, "a", &noon(1));
    keep(&mut h, &mut s, "b", &noon(2));
    shelf.fail(ShelfCall::Save, SealedError::Io);
    assert_eq!(s.save(&h), Err(SealedError::Io), "day 1's list failed");
    let day2 = names(&shelf, on(2));
    assert!(day2.iter().any(|n| n == "list"), "day 2 saved: {day2:?}");
    s.save(&h).unwrap();
    assert_eq!(reload(&shelf).history, h);
}

#[test]
fn a_damaged_content_blob_reads_as_damaged_once() {
    let shelf = FakeShelf::new();
    let (mut s, mut h) = (open(&shelf), History::default());
    let a = keep(&mut h, &mut s, "a", &noon(1));
    shelf.fail(ShelfCall::Load, SealedError::Damaged);
    assert_eq!(s.parts(h.get(a).unwrap()), Err(SealedError::Damaged));
    assert_eq!(s.parts(h.get(a).unwrap()), Ok(None), "set aside, so gone");
}
