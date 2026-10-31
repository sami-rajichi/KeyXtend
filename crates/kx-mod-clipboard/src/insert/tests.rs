//! The insert plan: tick order, the join only after a written text, links as text, entries gone meanwhile, file facts
//! sorted into kept, gone and away, and the parts a file entry writes.

use std::path::PathBuf;

use kx_module_api::Redacted;
use kx_module_api::ports::{ClipPart, FileFact};

use super::*;
use crate::entry::Entry;
use crate::fixture::{CFG, at, date, files, image, text};
use crate::order::History;

/// 31 October 2026 at noon in Paris.
fn noon() -> kx_module_api::ports::LocalTime {
    at(date(2026, 10, 31), 12, 0)
}

/// Adds a copy of `parts` to `h`; gives its id.
fn add(h: &mut History, parts: &[ClipPart]) -> u64 {
    h.add(Entry::new(parts, None, &noon(), &CFG))
}

/// Every step of an insert of `ids`, each written as it comes, nothing sorted out.
fn steps(ids: &[u64], h: &History) -> Vec<Step> {
    let mut insert = Insert::new(ids.to_vec(), Sorted::default());
    let mut all = Vec::new();
    while let Some(step) = insert.next(h) {
        insert.written(&step);
        all.push(step);
    }
    all
}

/// Whether each step asks for the join first.
fn joins(steps: &[Step]) -> Vec<bool> {
    steps.iter().map(|s| s.join).collect()
}

/// `paths` as the insert keeps them.
fn kept(paths: &[&str]) -> Redacted<Vec<PathBuf>> {
    Redacted::new(paths.iter().map(PathBuf::from).collect())
}

#[test]
fn the_join_is_the_breaks_then_the_rule_and_the_breaks_again() {
    let plain = JoinConfig {
        breaks: 2,
        rule: String::new(),
    };
    assert_eq!(plain.text(), "\n\n");
    let ruled = JoinConfig {
        breaks: 1,
        rule: "—— ✦ ——".to_owned(),
    };
    assert_eq!(ruled.text(), "\n—— ✦ ——\n");
}

#[test]
fn texts_go_in_tick_order_and_each_after_a_written_text_asks_the_join() {
    let mut h = History::default();
    let (a, b, c) = (
        add(&mut h, &[text("a")]),
        add(&mut h, &[text("b")]),
        add(&mut h, &[text("c")]),
    );
    let all = steps(&[c, a, b], &h);
    let ids: Vec<u64> = all.iter().map(|s| s.id).collect();
    assert_eq!((ids, joins(&all)), (vec![c, a, b], vec![false, true, true]));
}

#[test]
fn images_and_files_get_no_join_and_links_count_as_text() {
    let mut hist = History::default();
    let words = add(&mut hist, &[text("words")]);
    let pic = add(&mut hist, &[image()]);
    let doc = add(&mut hist, &[files(&["C:\\a.txt"])]);
    let link = add(&mut hist, &[text("https://example.com")]);
    assert_eq!(joins(&steps(&[words, pic, words], &hist)), [false; 3]);
    assert_eq!(joins(&steps(&[words, doc, link], &hist)), [false; 3]);
    assert_eq!(joins(&steps(&[link, words], &hist)), [false, true]);
}

#[test]
fn only_a_written_entry_counts_for_the_join() {
    let mut h = History::default();
    let (a, b, c) = (
        add(&mut h, &[text("a")]),
        add(&mut h, &[text("b")]),
        add(&mut h, &[text("c")]),
    );
    let mut insert = Insert::new(vec![a, b, c], Sorted::default());
    let first = insert.next(&h).unwrap();
    insert.written(&first);
    assert!(
        insert.next(&h).unwrap().join,
        "b, found gone, is never written"
    );
    assert!(insert.next(&h).unwrap().join, "one join before c, after a");
    assert_eq!(insert.next(&h), None);
}

#[test]
fn an_entry_gone_from_the_history_is_skipped() {
    let mut h = History::default();
    let (a, b, c) = (
        add(&mut h, &[text("a")]),
        add(&mut h, &[image()]),
        add(&mut h, &[text("c")]),
    );
    let mut insert = Insert::new(vec![a, b, c], Sorted::default());
    let first = insert.next(&h).unwrap();
    insert.written(&first);
    h.remove(b);
    let next = insert.next(&h).unwrap();
    assert_eq!(
        (next.id, next.join),
        (c, true),
        "a and c are now side by side"
    );
    assert_eq!(insert.next(&h), None);
}

#[test]
fn a_written_text_deleted_meanwhile_still_gives_no_join_before_an_image() {
    let mut h = History::default();
    let (a, b, c) = (
        add(&mut h, &[text("a")]),
        add(&mut h, &[text("b")]),
        add(&mut h, &[image()]),
    );
    let mut insert = Insert::new(vec![a, b, c], Sorted::default());
    let first = insert.next(&h).unwrap();
    insert.written(&first);
    let second = insert.next(&h).unwrap();
    insert.written(&second);
    h.remove(b);
    let third = insert.next(&h).unwrap();
    assert_eq!((third.id, third.join), (c, false));
}

#[test]
fn file_facts_sort_into_kept_gone_and_away() {
    let mut h = History::default();
    let paths = ["C:\\a", "C:\\b"];
    let ids: Vec<u64> = (0..5).map(|_| add(&mut h, &[files(&paths)])).collect();
    let (size, gone, away) = (FileFact::Size(3), FileFact::Gone, FileFact::Away);
    let facts = vec![
        (ids[0], vec![size, size]),
        (ids[1], vec![gone, size]),
        (ids[2], vec![gone, gone]),
        (ids[3], vec![away, away]),
        (ids[4], vec![gone, away]),
    ];
    let sorted = sort(&facts, &h);
    assert_eq!(sorted.keep.get(&ids[0]), Some(&kept(&paths)));
    assert_eq!(sorted.keep.get(&ids[1]), Some(&kept(&["C:\\b"])));
    assert_eq!(sorted.gone, [ids[2]], "only every file gone deletes it");
    assert_eq!(
        sorted.away,
        [ids[3], ids[4]],
        "a file out of reach keeps the row"
    );
}

#[test]
fn an_insert_pastes_the_files_still_there_and_skips_gone_and_away_entries() {
    let mut hist = History::default();
    let words = add(&mut hist, &[text("a")]);
    let doc = add(&mut hist, &[files(&["C:\\a", "C:\\b"])]);
    let (gone, away) = (
        add(&mut hist, &[files(&["C:\\gone"])]),
        add(&mut hist, &[files(&["E:\\away"])]),
    );
    let facts = vec![
        (doc, vec![FileFact::Gone, FileFact::Size(9)]),
        (gone, vec![FileFact::Gone]),
        (away, vec![FileFact::Away]),
    ];
    let mut insert = Insert::new(vec![gone, doc, away, words], sort(&facts, &hist));
    let first = insert.next(&hist).unwrap();
    assert_eq!((first.id, first.files), (doc, Some(kept(&["C:\\b"]))));
    let second = insert.next(&hist).map(|s| (s.id, s.files));
    assert_eq!(second, Some((words, None)));
    assert_eq!(insert.next(&hist), None);
}

#[test]
fn the_file_entries_are_those_the_insert_checks_first() {
    let mut h = History::default();
    let t = add(&mut h, &[text("a")]);
    let f = add(&mut h, &[files(&["C:\\a.txt"])]);
    let m = add(&mut h, &[files(&["C:\\song.mp3"])]);
    let looked = files_of(&[t, m, f], &h);
    let ids: Vec<u64> = looked.iter().map(|(id, _)| *id).collect();
    assert_eq!(ids, [m, f]);
    assert_eq!(looked[1].1, kept(&["C:\\a.txt"]));
}

#[test]
fn a_file_entry_writes_only_its_kept_paths_and_its_other_parts() {
    let parts = vec![files(&["C:\\a", "C:\\b"]), text("names")];
    let cut = paste_parts(parts.clone(), Some(&kept(&["C:\\b"])));
    assert_eq!(cut, [files(&["C:\\b"]), text("names")]);
    assert_eq!(paste_parts(parts.clone(), None), parts);
}

#[test]
fn no_debug_shows_a_path() {
    let step = Step {
        id: 1,
        files: Some(kept(&["C:\\secret.txt"])),
        join: false,
        text: false,
    };
    let sorted = Sorted {
        keep: [(1, kept(&["C:\\secret.txt"]))].into(),
        ..Sorted::default()
    };
    for shown in [format!("{step:?}"), format!("{sorted:?}")] {
        assert!(!shown.contains("secret"), "{shown}");
    }
}
