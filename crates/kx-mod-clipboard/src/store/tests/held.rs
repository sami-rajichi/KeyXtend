//! Holds: a pending move's blobs and the list a move left are kept until the move ends, through failed writes and
//! copies; a move found gone during a save is told through `gone`.

use super::fault::Cut;
use super::*;

/// The state the review traced: `y` was pinned, then pinning `x` failed its list, unpinning `x` failed its copy,
/// and unpinning `y` saved while `x`'s copy failed again.
fn stuck() -> (FakeShelf, Store, History, u64, u64) {
    let shelf = FakeShelf::new();
    let cut = Cut::idle(&shelf);
    let (mut s, mut h) = (Store::new(cut.clone(), STORE), History::default());
    let x = keep(&mut h, &mut s, "x", &noon(1));
    let y = keep(&mut h, &mut s, "y", &noon(1));
    s.save(&h).unwrap();
    h.pin(y);
    s.moved(&h, y, on(1)).unwrap();
    cut.arm(1, false);
    h.pin(x);
    assert_eq!(s.moved(&h, x, on(1)), Err(SealedError::Io), "pinned's list");
    cut.arm(0, false);
    let from = h.unpin(x, &noon(3)).unwrap();
    assert_eq!(s.moved(&h, x, from), Err(SealedError::Io), "x's copy");
    cut.arm(1, false);
    let from = h.unpin(y, &noon(3)).unwrap();
    assert_eq!(s.moved(&h, y, from), Err(SealedError::Io), "x's copy again");
    (shelf, s, h, x, y)
}

/// Asserts `x` and `y` load whole from `shelf`, as `h` has them.
fn whole(shelf: &FakeShelf, h: &History, x: u64, y: u64) {
    let mut s = open(shelf);
    assert_eq!(s.load().unwrap().history, *h);
    for (id, text_of) in [(x, "x"), (y, "y")] {
        let back = s.parts(h.get(id).unwrap());
        assert_eq!(back, Ok(Some(vec![text(text_of)])), "{text_of}");
    }
}

#[test]
fn a_failed_list_then_a_failed_copy_keep_the_entry_whole() {
    let (shelf, mut s, h, x, y) = stuck();
    let x0 = part_name(x, 0).unwrap();
    assert!(
        shelf.blob(&folder(Place::Pinned), &x0).is_some(),
        "home kept"
    );
    s.save(&h).unwrap();
    whole(&shelf, &h, x, y);
}

#[test]
fn a_move_whose_home_went_starts_again_from_the_folder_it_left() {
    let (shelf, mut s, h, x, y) = stuck();
    let pinned = folder(Place::Pinned);
    shelf.remove(&pinned, &part_name(x, 0).unwrap()).unwrap();
    s.save(&h).unwrap();
    assert!(s.gone().is_empty(), "day 1 still held it");
    whole(&shelf, &h, x, y);
}

#[test]
fn a_move_whose_home_and_left_folder_both_lost_a_blob_ends_and_is_told_gone() {
    let (shelf, mut s, mut h, x, y) = stuck();
    let x0 = part_name(x, 0).unwrap();
    shelf.remove(&folder(Place::Pinned), &x0).unwrap();
    shelf.remove(&folder(on(1)), &x0).unwrap();
    s.save(&h).unwrap();
    assert_eq!(s.gone(), [x], "neither folder holds it whole");
    s.delete(h.get(x).unwrap()).unwrap();
    h.remove(x);
    s.save(&h).unwrap();
    let mine = format!("{x:x}-");
    for place in [on(1), on(3), Place::Pinned] {
        let left = names(&shelf, place);
        assert!(left.iter().all(|n| !n.starts_with(&mine)), "{left:?}");
    }
    let back = open(&shelf).parts(h.get(y).unwrap());
    assert_eq!(back, Ok(Some(vec![text("y")])), "y stays whole");
}

#[test]
fn a_save_tells_the_moves_it_found_gone_once() {
    let shelf = FakeShelf::new();
    let (mut s, mut h) = (open(&shelf), History::default());
    let a = keep(&mut h, &mut s, "a", &noon(1));
    s.save(&h).unwrap();
    h.pin(a);
    shelf.fail(ShelfCall::Save, SealedError::Io);
    assert_eq!(s.moved(&h, a, on(1)), Err(SealedError::Io));
    shelf.delete_by_hand(&folder(on(1)));
    s.save(&h).unwrap();
    assert_eq!(s.gone(), [a]);
    assert!(s.gone().is_empty(), "told once");
}

#[test]
fn a_move_found_gone_by_moved_is_not_told_again() {
    let shelf = FakeShelf::new();
    let (mut s, mut h) = (open(&shelf), History::default());
    let a = keep(&mut h, &mut s, "a", &noon(1));
    s.save(&h).unwrap();
    shelf.delete_by_hand(&folder(on(1)));
    h.pin(a);
    assert_eq!(s.moved(&h, a, on(1)), Ok(false));
    assert!(s.gone().is_empty(), "moved answered for it");
}
