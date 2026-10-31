//! The insert plan: tick order, the join only between two texts, links as text, entries gone meanwhile, file facts
//! sorted into kept, gone and away, and the parts a file entry writes.

use std::path::PathBuf;

use kx_module_api::Redacted;
use kx_module_api::ports::{ClipPart, FileFact, ImageCodec};

use super::*;
use crate::entry::Entry;
use crate::fixture::{CFG, at, date, files, text};
use crate::order::History;

/// 31 October 2026 at noon in Paris.
fn noon() -> kx_module_api::ports::LocalTime {
    at(date(2026, 10, 31), 12, 0)
}

/// An image part.
fn image() -> ClipPart {
    ClipPart::Image {
        codec: ImageCodec::Png,
        bytes: Redacted::new(vec![1, 2, 3]),
    }
}

/// Adds a copy of `parts` to `h`; gives its id.
fn add(h: &mut History, parts: &[ClipPart]) -> u64 {
    h.add(Entry::new(parts, None, &noon(), &CFG))
}

/// Every step of an insert of `ids`, nothing sorted out.
fn steps(ids: &[u64], h: &History) -> Vec<Step> {
    let mut insert = Insert::new(ids.to_vec(), Sorted::default());
    std::iter::from_fn(|| insert.next(h)).collect()
}

/// The step that pastes `id` whole.
fn whole(id: u64) -> Step {
    Step::Entry { id, files: None }
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
fn texts_go_in_tick_order_with_a_join_between_each_two() {
    let mut h = History::default();
    let (a, b, c) = (
        add(&mut h, &[text("a")]),
        add(&mut h, &[text("b")]),
        add(&mut h, &[text("c")]),
    );
    let want = vec![whole(c), Step::Join, whole(a), Step::Join, whole(b)];
    assert_eq!(steps(&[c, a, b], &h), want);
}

#[test]
fn images_and_files_get_no_join_and_links_count_as_text() {
    let mut hist = History::default();
    let words = add(&mut hist, &[text("words")]);
    let pic = add(&mut hist, &[image()]);
    let doc = add(&mut hist, &[files(&["C:\\a.txt"])]);
    let link = add(&mut hist, &[text("https://example.com")]);
    assert_eq!(
        steps(&[words, pic, words], &hist),
        [whole(words), whole(pic), whole(words)]
    );
    assert_eq!(
        steps(&[words, doc, link], &hist),
        [whole(words), whole(doc), whole(link)]
    );
    assert_eq!(
        steps(&[link, words], &hist),
        [whole(link), Step::Join, whole(words)]
    );
}

#[test]
fn an_entry_gone_meanwhile_is_skipped_and_the_join_follows_what_is_pasted() {
    let mut h = History::default();
    let (a, b, c) = (
        add(&mut h, &[text("a")]),
        add(&mut h, &[image()]),
        add(&mut h, &[text("c")]),
    );
    let mut insert = Insert::new(vec![a, b, c], Sorted::default());
    assert_eq!(insert.next(&h), Some(whole(a)));
    h.remove(b);
    assert_eq!(
        insert.next(&h),
        Some(Step::Join),
        "a and c are now side by side"
    );
    assert_eq!(insert.next(&h), Some(whole(c)));
    assert_eq!(insert.next(&h), None);
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
    let keep = Some(kept(&["C:\\b"]));
    assert_eq!(
        insert.next(&hist),
        Some(Step::Entry {
            id: doc,
            files: keep
        })
    );
    assert_eq!(insert.next(&hist), Some(whole(words)));
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
    let step = Step::Entry {
        id: 1,
        files: Some(kept(&["C:\\secret.txt"])),
    };
    let sorted = Sorted {
        keep: [(1, kept(&["C:\\secret.txt"]))].into(),
        ..Sorted::default()
    };
    for shown in [format!("{step:?}"), format!("{sorted:?}")] {
        assert!(!shown.contains("secret"), "{shown}");
    }
}
