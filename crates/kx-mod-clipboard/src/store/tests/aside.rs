//! Lists set aside: a list from a newer version fails the load before anything changes; a list that will not read at
//! ours is kept aside in its folder with its content, told once, spared by later loads and saves, and dropped with
//! its day past `keep_days`.

use std::mem::size_of;

use super::*;

/// The name of a folder's first aside blob.
const ASIDE: &str = "aside-1-0";
/// Where a list's entry count starts: after the head and the counter.
const COUNT_AT: usize = list::HEAD_LEN + size_of::<u64>();

/// Every blob on `shelf` with its folder and bytes, read without a load.
fn every_blob(shelf: &FakeShelf) -> Vec<(BlobName, BlobName, Option<Vec<u8>>)> {
    let mut all = Vec::new();
    for f in shelf.folders().unwrap() {
        for n in shelf.names(&f).unwrap() {
            let bytes = shelf.blob(&f, &n);
            all.push((f.clone(), n, bytes));
        }
    }
    all
}

/// The writes asked of `shelf` so far.
fn writes(shelf: &FakeShelf) -> usize {
    let calls = [ShelfCall::Save, ShelfCall::Remove, ShelfCall::DropFolder];
    calls.iter().map(|c| shelf.calls(*c)).sum()
}

/// A checked blob name.
fn blob(name: &str) -> BlobName {
    BlobName::new(name).unwrap()
}

#[test]
fn a_newer_list_fails_the_load_and_changes_nothing() {
    let shelf = FakeShelf::new();
    let (mut s, mut h) = (open(&shelf), History::default());
    keep(&mut h, &mut s, "a", &noon(1));
    keep(&mut h, &mut s, "b", &noon(2));
    s.save(&h).unwrap();
    let head = list::head_name().unwrap();
    shelf.plant(&folder(on(1)), &blob("ff-0"), vec![1]);
    shelf.plant(&folder(on(2)), &head, vec![list::VERSION]);
    // Pinned loads last, after a folder to clean and one to set aside.
    shelf.plant(&folder(Place::Pinned), &head, vec![list::VERSION + 1]);
    let (before, done) = (every_blob(&shelf), writes(&shelf));
    let load = open(&shelf).load().map(|_| ());
    assert_eq!(load, Err(StoreError::Newer));
    assert_eq!(every_blob(&shelf), before);
    assert_eq!(writes(&shelf), done, "not one write");
}

#[test]
fn a_damaged_list_sets_its_day_aside_with_its_content_once() {
    let shelf = FakeShelf::new();
    let (mut s, mut h) = (open(&shelf), History::default());
    let lost = keep(&mut h, &mut s, "a", &noon(1));
    let b = keep(&mut h, &mut s, "b", &noon(2));
    let c = keep(&mut h, &mut s, "c", &noon(3));
    s.save(&h).unwrap();
    // Folders load in order, so the first load is day 1's list.
    shelf.fail(ShelfCall::Load, SealedError::Damaged);
    let first = reload(&shelf);
    assert_eq!(first.aside, [on(1)]);
    assert_eq!(ids(&first.history), [c, b], "the other days stay");
    let kept = [format!("{lost:x}-0"), ASIDE.into()];
    assert_eq!(names(&shelf, on(1)), kept, "its content stays");
    let mut s = open(&shelf);
    let second = s.load().unwrap();
    assert!(second.aside.is_empty(), "told once");
    s.save(&second.history).unwrap();
    assert_eq!(names(&shelf, on(1)), kept, "spared by loads and saves");
}

#[test]
fn a_list_that_will_not_read_is_kept_aside_once() {
    let shelf = FakeShelf::new();
    let (mut s, mut h) = (open(&shelf), History::default());
    let a = keep(&mut h, &mut s, "a", &noon(1));
    let b = keep(&mut h, &mut s, "b", &noon(2));
    s.save(&h).unwrap();
    let bad = [&[list::VERSION][..], b"not a list"].concat();
    shelf.plant(&folder(on(1)), &list::head_name().unwrap(), bad.clone());
    let first = reload(&shelf);
    assert_eq!((first.aside, ids(&first.history)), (vec![on(1)], vec![b]));
    assert_eq!(names(&shelf, on(1)), [format!("{a:x}-0"), ASIDE.into()]);
    assert_eq!(shelf.blob(&folder(on(1)), &blob(ASIDE)), Some(bad));
    assert!(reload(&shelf).aside.is_empty(), "told once");
}

#[test]
fn a_split_list_that_will_not_read_is_kept_whole_in_pieces() {
    let shelf = FakeShelf::new();
    let (mut s, mut h) = (
        Store::new(Arc::new(shelf.clone()), SMALL),
        History::default(),
    );
    keep(&mut h, &mut s, "a", &noon(1));
    keep(&mut h, &mut s, "b", &noon(1));
    s.save(&h).unwrap();
    let (day1, head) = (folder(on(1)), list::head_name().unwrap());
    let mut bad = shelf.blob(&day1, &head).unwrap();
    bad[COUNT_AT] = u8::MAX;
    shelf.plant(&day1, &head, bad.clone());
    let chunk = |n| shelf.blob(&day1, &list::chunk_name(0, n).unwrap());
    let whole: Vec<u8> = (1..).map_while(chunk).fold(bad, |w, c| [w, c].concat());
    let loaded = Store::new(Arc::new(shelf.clone()), SMALL).load().unwrap();
    assert_eq!(loaded.aside, [on(1)]);
    let piece = |k| shelf.blob(&day1, &blob(&format!("aside-1-{k}")));
    let kept: Vec<Vec<u8>> = (0..).map_while(piece).collect();
    assert!(kept.len() > 1, "split under the blob size");
    assert_eq!(kept.concat(), whole);
}

#[test]
fn a_load_counts_past_orphans_and_an_emptied_aside_folder_stays() {
    let shelf = FakeShelf::new();
    let day1 = folder(on(1));
    shelf.plant(&day1, &blob(ASIDE), vec![]);
    shelf.plant(&day1, &blob("ff-0"), vec![1]);
    let mut s = open(&shelf);
    let mut h = s.load().unwrap().history;
    assert!(h.next() > 0xff, "a new entry never takes an orphan's name");
    let a = keep(&mut h, &mut s, "a", &noon(1));
    s.save(&h).unwrap();
    s.delete(h.get(a).unwrap()).unwrap();
    h.remove(a);
    s.save(&h).unwrap();
    assert_eq!(names(&shelf, on(1)), [ASIDE, "ff-0"]);
}

#[test]
fn a_day_set_aside_drops_past_keep_days_and_pinned_keeps_it() {
    let shelf = FakeShelf::new();
    for place in [on(1), Place::Pinned] {
        shelf.plant(&folder(place), &blob(ASIDE), vec![]);
        shelf.plant(&folder(place), &blob("5-0"), vec![1]);
    }
    let mut s = open(&shelf);
    s.load().unwrap();
    s.drop_past(day(8), 7).unwrap();
    assert_eq!(shelf.folders(), Ok(vec![folder(Place::Pinned)]));
    assert_eq!(names(&shelf, Place::Pinned), ["5-0", ASIDE]);
}
