//! The order: pinned first, newest first; a duplicate, an insert or an unpin moves up with the new time; the counter
//! survives a reload, and a clock set back changes nothing.

use super::*;
use crate::fixture::{at, copied, date};

/// The ids in order.
fn ids(h: &History) -> Vec<u64> {
    h.entries().iter().map(|e| e.id).collect()
}

/// A history of `texts` copied a minute apart on 2026-10-31, oldest first; gives it and their ids.
fn history(texts: &[&str]) -> (History, Vec<u64>) {
    let mut h = History::default();
    let day = date(2026, 10, 31);
    let added = (0u8..)
        .zip(texts)
        .map(|(m, s)| h.add(copied(s, &at(day, 9, m))));
    let ids = added.collect();
    (h, ids)
}

#[test]
fn pinned_come_first_then_the_newest() {
    let (mut h, id) = history(&["a", "b", "c"]);
    assert_eq!(ids(&h), [id[2], id[1], id[0]]);
    assert_eq!(h.pin(id[0]), Some(Place::Day(date(2026, 10, 31))));
    assert_eq!(ids(&h), [id[0], id[2], id[1]]);
    h.pin(id[1]);
    assert_eq!(
        ids(&h),
        [id[1], id[0], id[2]],
        "pins keep their own newest-first order"
    );
}

#[test]
fn a_duplicate_moves_up_with_the_new_time_and_day() {
    let (mut h, id) = history(&["a", "b"]);
    let hash = h.get(id[0]).unwrap().hash;
    assert_eq!(h.twin(hash, |_| true), Some(id[0]));
    let now = at(date(2026, 11, 2), 8, 0);
    assert_eq!(h.raise(id[0], &now), Some(Place::Day(date(2026, 10, 31))));
    assert_eq!(ids(&h), [id[0], id[1]]);
    let top = h.get(id[0]).unwrap();
    assert_eq!(
        (top.at.unix_ms, top.place),
        (now.unix_ms, Place::Day(now.date))
    );
}

#[test]
fn a_pinned_duplicate_moves_up_and_stays_pinned() {
    let (mut h, id) = history(&["a", "b", "c"]);
    h.pin(id[0]);
    h.pin(id[1]);
    let now = at(date(2026, 11, 1), 10, 0);
    assert_eq!(h.raise(id[0], &now), Some(Place::Pinned));
    assert_eq!(ids(&h), [id[0], id[1], id[2]]);
    let top = h.get(id[0]).unwrap();
    assert_eq!((top.at.unix_ms, top.place), (now.unix_ms, Place::Pinned));
}

#[test]
fn a_twin_needs_the_same_hash_and_the_same_content() {
    let (h, id) = history(&["a", "b"]);
    let hash = h.get(id[1]).unwrap().hash;
    assert_eq!(h.twin(hash, |_| false), None, "a hash collision is no twin");
    let mut asked = 0;
    assert_eq!(
        h.twin(hash ^ 1, |_| {
            asked += 1;
            true
        }),
        None
    );
    assert_eq!(asked, 0, "content is compared only on an equal hash");
}

#[test]
fn unpin_counts_as_copied_now() {
    let (mut h, id) = history(&["a", "b"]);
    h.pin(id[0]);
    let now = at(date(2026, 11, 6), 7, 0);
    assert_eq!(h.unpin(id[0], &now), Some(Place::Pinned));
    assert_eq!(ids(&h), [id[0], id[1]], "it tops the unpinned entries");
    let e = h.get(id[0]).unwrap();
    assert_eq!((e.at.unix_ms, e.place), (now.unix_ms, Place::Day(now.date)));
}

#[test]
fn pinning_or_unpinning_twice_changes_nothing() {
    let (mut h, id) = history(&["a", "b"]);
    let before = h.clone();
    let now = at(date(2026, 11, 6), 7, 0);
    assert_eq!(h.unpin(id[0], &now), Some(Place::Day(date(2026, 10, 31))));
    assert_eq!(h, before, "an unpinned entry keeps its time");
    h.pin(id[0]);
    let pinned = h.clone();
    assert_eq!(h.pin(id[0]), Some(Place::Pinned));
    assert_eq!(h, pinned);
}

#[test]
fn a_missing_id_moves_nothing() {
    let (mut h, _) = history(&["a"]);
    let before = h.clone();
    let now = at(date(2026, 11, 6), 7, 0);
    assert_eq!(h.raise(99, &now), None);
    assert_eq!(h.pin(99), None);
    assert_eq!(h.unpin(99, &now), None);
    assert!(!h.update(99, |e| e.thumb = true));
    assert_eq!(h.remove(99), None);
    assert_eq!(h, before);
}

#[test]
fn remove_and_update_reach_one_entry() {
    let (mut h, id) = history(&["a", "b"]);
    assert!(h.update(id[0], |e| e.thumb = true));
    assert!(h.get(id[0]).unwrap().thumb);
    assert_eq!(h.remove(id[1]).map(|e| e.id), Some(id[1]));
    assert_eq!(ids(&h), [id[0]]);
    assert_eq!((h.len(), h.is_empty()), (1, false));
}

#[test]
fn bytes_sum_what_the_entries_keep() {
    let (h, _) = history(&["ab", "cde"]);
    assert_eq!(h.bytes(), 5);
    assert_eq!(History::default().bytes(), 0);
}

#[test]
fn a_reload_sorts_and_numbers_past_every_id_in_use() {
    let (h, id) = history(&["a", "b", "c"]);
    let mut shuffled = h.entries().to_vec();
    shuffled.reverse();
    let mut back = History::new(shuffled, 0);
    assert_eq!(ids(&back), ids(&h));
    let fresh = back.add(copied("d", &at(date(2026, 10, 31), 10, 0)));
    assert!(fresh > id[2] && back.next() > fresh);
    let saved = History::new(Vec::new(), 40);
    assert_eq!(saved.next(), 40, "a saved counter is kept");
}

#[test]
fn a_clock_set_back_keeps_the_order_and_a_new_copy_still_tops_it() {
    let (mut h, id) = history(&["a", "b"]);
    let before = ids(&h);
    let back = at(date(2026, 10, 20), 9, 0);
    let fresh = h.add(copied("c", &back));
    assert_eq!(ids(&h), [fresh, before[0], before[1]]);
    h.raise(id[0], &at(date(2026, 10, 19), 9, 0));
    assert_eq!(ids(&h), [id[0], fresh, id[1]]);
}

#[test]
fn pop_oldest_takes_the_oldest_unpinned_and_never_a_pin() {
    let (mut h, id) = history(&["a", "b"]);
    h.pin(id[0]);
    assert_eq!(h.pop_oldest().map(|e| e.id), Some(id[1]));
    assert_eq!(h.pop_oldest(), None);
    assert_eq!(ids(&h), [id[0]]);
}

#[test]
fn take_where_takes_the_chosen_in_order() {
    let (mut h, id) = history(&["a", "b", "c"]);
    let gone: Vec<u64> = h
        .take_where(|e| e.id != id[1])
        .iter()
        .map(|e| e.id)
        .collect();
    assert_eq!(gone, [id[2], id[0]]);
    assert_eq!(ids(&h), [id[1]]);
}
