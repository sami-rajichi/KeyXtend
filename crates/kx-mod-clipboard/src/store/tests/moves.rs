//! Moves: a pin, a raise from an older day and an unpin carry the blobs and both lists; a move whose content went by
//! hand says so; a failed copy stays pending while reads still find the content; two copies after a crash merge.

use super::*;

#[test]
fn a_pin_carries_the_blobs_and_both_lists() {
    let shelf = FakeShelf::new();
    let (mut s, mut h) = (open(&shelf), History::default());
    let a = keep(&mut h, &mut s, "a", &noon(1));
    let b = keep(&mut h, &mut s, "b", &noon(1));
    s.put_thumb(h.get(a).unwrap(), &grey(1)).unwrap();
    h.update(a, |e| e.thumb = true);
    s.save(&h).unwrap();
    h.pin(a);
    assert_eq!(s.moved(&h, a, on(1)), Ok(true));
    let pinned = [format!("{a:x}-0"), format!("{a:x}-t"), "list".into()];
    assert_eq!(names(&shelf, Place::Pinned), pinned);
    assert_eq!(names(&shelf, on(1)), [format!("{b:x}-0"), "list".into()]);
    assert_eq!(reload(&shelf).history, h);
    assert_eq!(s.parts(h.get(a).unwrap()), Ok(Some(vec![text("a")])));
}

#[test]
fn a_raise_from_an_older_day_and_an_unpin_move_to_their_day() {
    let shelf = FakeShelf::new();
    let (mut s, mut h) = (open(&shelf), History::default());
    let a = keep(&mut h, &mut s, "a", &noon(1));
    let b = keep(&mut h, &mut s, "b", &noon(1));
    s.save(&h).unwrap();
    let from = h.raise(a, &noon(2)).unwrap();
    assert_eq!(s.moved(&h, a, from), Ok(true));
    h.pin(b);
    s.moved(&h, b, on(1)).unwrap();
    let from = h.unpin(b, &noon(3)).unwrap();
    assert_eq!(s.moved(&h, b, from), Ok(true));
    let left = vec![folder(on(2)), folder(on(3))];
    assert_eq!(shelf.folders(), Ok(left), "emptied folders drop");
    assert_eq!(reload(&shelf).history, h);
}

#[test]
fn a_move_whose_folder_went_by_hand_says_its_content_is_gone() {
    let shelf = FakeShelf::new();
    let (mut s, mut h) = (open(&shelf), History::default());
    let a = keep(&mut h, &mut s, "a", &noon(1));
    s.save(&h).unwrap();
    shelf.delete_by_hand(&folder(on(1)));
    h.pin(a);
    assert_eq!(s.moved(&h, a, on(1)), Ok(false));
    assert_eq!(s.missing(&h), Ok(vec![a]));
}

#[test]
fn a_failed_copy_stays_pending_and_reads_find_the_content() {
    let shelf = FakeShelf::new();
    let (mut s, mut h) = (open(&shelf), History::default());
    let a = keep(&mut h, &mut s, "a", &noon(1));
    let b = keep(&mut h, &mut s, "b", &noon(1));
    s.save(&h).unwrap();
    h.pin(a);
    shelf.fail(ShelfCall::Save, SealedError::Io);
    assert_eq!(s.moved(&h, a, on(1)), Err(SealedError::Io));
    assert_eq!(s.parts(h.get(a).unwrap()), Ok(Some(vec![text("a")])));
    assert_eq!(s.missing(&h), Ok(vec![]));
    s.drop_past(day(8), 7).unwrap();
    assert!(
        shelf.folders().unwrap().contains(&folder(on(1))),
        "its blobs wait there"
    );
    s.save(&h).unwrap();
    assert_eq!(reload(&shelf).history, h);
    assert_eq!(names(&shelf, on(1)), [format!("{b:x}-0"), "list".into()]);
}

#[test]
fn a_delete_during_a_pending_move_removes_the_blobs_where_they_wait() {
    let shelf = FakeShelf::new();
    let (mut s, mut h) = (open(&shelf), History::default());
    let a = keep(&mut h, &mut s, "a", &noon(1));
    let b = keep(&mut h, &mut s, "b", &noon(1));
    s.save(&h).unwrap();
    h.pin(a);
    shelf.fail(ShelfCall::Save, SealedError::Io);
    assert!(s.moved(&h, a, on(1)).is_err());
    s.delete(h.get(a).unwrap()).unwrap();
    h.remove(a);
    s.save(&h).unwrap();
    assert_eq!(names(&shelf, on(1)), [format!("{b:x}-0"), "list".into()]);
    assert!(names(&shelf, Place::Pinned).is_empty());
}

#[test]
fn two_copies_after_a_crash_keep_the_newer_and_lose_the_other() {
    let shelf = FakeShelf::new();
    let (mut s, mut h) = (open(&shelf), History::default());
    let a = keep(&mut h, &mut s, "a", &noon(1));
    let b = keep(&mut h, &mut s, "b", &noon(1));
    s.save(&h).unwrap();
    let (day1, head) = (folder(on(1)), list::head_name().unwrap());
    let before = shelf.blob(&day1, &head).unwrap();
    h.pin(a);
    s.moved(&h, a, on(1)).unwrap();
    // Day 1's old list and a's blob there, as a crash between the two lists leaves them.
    shelf.plant(&day1, &head, before);
    shelf.plant(
        &day1,
        &part_name(a, 0).unwrap(),
        parts::encode(&[text("a")]),
    );
    let mut s = open(&shelf);
    let loaded = s.load().unwrap();
    assert_eq!(loaded.history, h, "the pinned copy wins");
    s.save(&loaded.history).unwrap();
    assert_eq!(names(&shelf, on(1)), [format!("{b:x}-0"), "list".into()]);
}
