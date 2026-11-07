//! A merge: the newer copy of one id wins whole, keeping the later last use; an entry on one side only is kept; and a
//! merge past the entry or byte limit brings nothing in.

use super::*;
use crate::fixture::{LIMITS, draft, field};
use kx_module_api::ports::EntryKind;

/// Three times, a day apart.
const MON: Stamp = Stamp::from_unix_ms(1_700_000_000_000);
const TUE: Stamp = Stamp::from_unix_ms(1_700_086_400_000);
const WED: Stamp = Stamp::from_unix_ms(1_700_172_800_000);

/// Entry `id`, a note named `name`, last changed at `changed`.
fn note(id: u64, name: &str, changed: Stamp) -> Entry {
    let text = vec![field("text", name, false)];
    Entry::new(id, draft(name, EntryKind::Note, text), changed, &LIMITS).unwrap()
}

/// Each entry's id and name, by id.
fn names(book: &Notebook) -> Vec<(u64, String)> {
    let mut all: Vec<_> = book
        .entries()
        .iter()
        .map(|e| (e.id(), e.name().expose().clone()))
        .collect();
    all.sort();
    all
}

/// The default limits with at most `max_bytes` saved bytes.
fn sized(max_bytes: usize) -> Limits {
    Limits {
        max_bytes,
        ..LIMITS
    }
}

/// The default limits with at most `max_entries` entries.
fn counted(max_entries: usize) -> Limits {
    Limits {
        max_entries,
        ..LIMITS
    }
}

#[test]
fn the_newer_copy_of_an_id_wins_either_way_and_a_tie_keeps_ours() {
    let mut book = Notebook::new(vec![
        note(1, "ours-1", MON),
        note(2, "ours-2", WED),
        note(3, "ours-3", TUE),
    ]);
    let theirs = vec![
        note(1, "theirs-1", TUE),
        note(2, "theirs-2", MON),
        note(3, "theirs-3", TUE),
    ];
    let added = book.merge(theirs, &LIMITS);
    let want = [(1, "theirs-1"), (2, "ours-2"), (3, "ours-3")].map(|(i, n)| (i, n.to_owned()));
    assert_eq!(names(&book), want);
    assert_eq!(added, Some(1), "only the newer copy came in");
}

#[test]
fn an_entry_on_one_side_only_is_kept() {
    let mut book = Notebook::new(vec![note(1, "ours", MON)]);
    let added = book.merge(vec![note(2, "theirs", MON)], &LIMITS);
    let want = [(1, "ours"), (2, "theirs")].map(|(i, n)| (i, n.to_owned()));
    assert_eq!(names(&book), want);
    assert_eq!(added, Some(1));
}

#[test]
fn the_later_last_use_stays_with_the_winning_copy() {
    let mut used = note(1, "ours", MON);
    used.use_at(WED);
    let mut book = Notebook::new(vec![used]);
    assert_eq!(book.merge(vec![note(1, "theirs", TUE)], &LIMITS), Some(1));
    let entry = book.get(1).unwrap();
    assert_eq!(
        (entry.name().expose().as_str(), entry.used()),
        ("theirs", Some(WED))
    );
}

#[test]
fn past_max_entries_nothing_comes_in_and_exactly_at_it_all_does() {
    let ours = Notebook::new(vec![note(1, "a", MON), note(2, "b", MON)]);
    let theirs = || {
        vec![
            note(1, "a2", TUE),
            note(3, "c", MON),
            note(4, "d", MON),
            note(5, "e", MON),
        ]
    };
    let mut book = ours.clone();
    assert_eq!(book.merge(theirs(), &counted(4)), None);
    assert_eq!(book, ours, "not even what would fit");
    assert_eq!(
        book.merge(theirs(), &counted(5)),
        Some(4),
        "a newer copy takes no new room"
    );
    assert_eq!(book.len(), 5);
}

#[test]
fn past_the_saved_size_nothing_comes_in_and_exactly_at_it_all_does() {
    let ours = Notebook::new(vec![note(1, "a", MON)]);
    let theirs = || vec![note(2, "b", MON), note(1, "a much longer name", TUE)];
    let mut whole = ours.clone();
    assert_eq!(whole.merge(theirs(), &LIMITS), Some(2));
    let mut book = ours.clone();
    assert_eq!(book.merge(theirs(), &sized(whole.bytes() - 1)), None);
    assert_eq!(book, ours);
    assert_eq!(book.merge(theirs(), &sized(whole.bytes())), Some(2));
    assert_eq!(book, whole);
}

#[test]
fn a_notebook_past_lowered_limits_still_takes_what_does_not_grow_it() {
    let ours = Notebook::new(vec![note(1, "a long name", MON), note(2, "b", MON)]);
    let smaller = sized(ours.bytes() - 1);
    let mut book = ours.clone();
    assert_eq!(book.merge(vec![note(2, "b", TUE)], &counted(1)), Some(1));
    assert_eq!(book.merge(vec![note(1, "a", TUE)], &smaller), Some(1));
    assert_eq!(book.merge(vec![note(3, "c", MON)], &counted(1)), None);
    assert_eq!(
        book.merge(vec![note(2, "a long name", WED)], &smaller),
        None
    );
    assert_eq!(names(&book)[0], (1, "a".to_owned()));
}

#[test]
fn a_merge_of_two_thousand_entries_keeps_its_byte_total_true() {
    let ours: Vec<_> = (0..1000).map(|i| note(i, "ours", MON)).collect();
    let theirs = || (500..2500).map(|i| note(i, "theirs", TUE)).collect();
    let roomy = |max_bytes| Limits {
        max_bytes,
        ..counted(2500)
    };
    let mut whole = Notebook::new(ours.clone());
    assert_eq!(whole.merge(theirs(), &counted(2500)), Some(2000));
    assert_eq!(whole.len(), 2500);
    let mut book = Notebook::new(ours);
    assert_eq!(book.merge(theirs(), &roomy(whole.bytes() - 1)), None);
    assert_eq!(book.merge(theirs(), &roomy(whole.bytes())), Some(2000));
    assert_eq!(book, whole);
}
