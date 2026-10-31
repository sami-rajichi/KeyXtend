//! What a look gathers from a copy, over a made-up clipboard that notes each format it fetches.

use std::cell::RefCell;

use super::*;
use crate::clip::gather::{Found, gather};

/// A made-up clipboard: each format with its size and part, and the formats fetched so far.
struct Fake {
    held: Vec<(u32, u64, ClipPart)>,
    fetched: RefCell<Vec<u32>>,
}

impl Fake {
    fn new(held: Vec<(u32, u64, ClipPart)>) -> Self {
        let fetched = RefCell::new(Vec::new());
        Self { held, fetched }
    }

    /// Gathers what `F` picks from this clipboard under `cap`.
    fn gather(&self, cap: u64) -> Result<Found, u64> {
        let ids: Vec<u32> = self.held.iter().map(|h| h.0).collect();
        let find = |f: u32| {
            self.fetched.borrow_mut().push(f);
            self.held.iter().find(|h| h.0 == f)
        };
        let size = |f| find(f).map(|h| h.1);
        gather(&F.pick(&ids), cap, size, |w| {
            find(w.format).map(|h| h.2.clone())
        })
    }

    fn fetched(&self, format: u32) -> bool {
        self.fetched.borrow().contains(&format)
    }
}

/// Gathers from formats whose size may be unknown (`None`); hands back what was found and the formats read.
fn gather_sized(held: &[(u32, Option<u64>, ClipPart)], cap: u64) -> (Result<Found, u64>, Vec<u32>) {
    let ids: Vec<u32> = held.iter().map(|h| h.0).collect();
    let find = |f: u32| held.iter().find(|h| h.0 == f);
    let read = RefCell::new(Vec::new());
    let size = |f| find(f).and_then(|h| h.1);
    let found = gather(&F.pick(&ids), cap, size, |w| {
        read.borrow_mut().push(w.format);
        find(w.format).map(|h| h.2.clone())
    });
    (found, read.into_inner())
}

fn text(t: &str) -> ClipPart {
    ClipPart::Text(Redacted::new(t.to_owned()))
}

fn html() -> ClipPart {
    ClipPart::Html(Redacted::new(b"<b>x</b>".to_vec()))
}

fn image() -> ClipPart {
    ClipPart::Image {
        codec: ImageCodec::Dib,
        bytes: Redacted::new(vec![1, 2, 3]),
    }
}

fn parts(found: Result<Found, u64>) -> (Vec<ClipPart>, bool) {
    let found = found.expect("not too big");
    (found.parts, found.blanked)
}

#[test]
fn a_picture_beside_text_is_neither_fetched_nor_counted() {
    let clip = Fake::new(vec![(F.text, 5, text("kx")), (F.dib, 1_000, image())]);
    assert_eq!(parts(clip.gather(10)), (vec![text("kx")], false));
    assert!(!clip.fetched(F.dib) && !clip.fetched(F.dib_v5));
}

#[test]
fn a_picture_beside_blank_text_is_read_instead() {
    let clip = Fake::new(vec![(F.text, 4, text(" \r\n")), (F.dib, 3, image())]);
    assert_eq!(parts(clip.gather(10)), (vec![image()], true));
}

#[test]
fn blank_text_alone_leaves_no_part() {
    let clip = Fake::new(vec![(F.text, 4, text("\r\n"))]);
    assert_eq!(parts(clip.gather(10)), (Vec::new(), true));
}

#[test]
fn the_size_sum_stops_once_past_the_cap_and_nothing_more_is_fetched() {
    let held = vec![
        (F.html, 8, html()),
        (F.rtf, 8, html()),
        (F.text, 8, text("kx")),
    ];
    let clip = Fake::new(held);
    assert_eq!(clip.gather(10).err(), Some(16));
    assert!(!clip.fetched(F.text), "past the cap");
}

#[test]
fn an_unsized_format_is_skipped_and_the_rest_kept() {
    let held = [(F.html, None, html()), (F.text, Some(5), text("kx"))];
    let (found, read) = gather_sized(&held, 10);
    assert_eq!(parts(found), (vec![text("kx")], false));
    assert_eq!(read, [F.text], "the unsized HTML is never read");
}

#[test]
fn an_unsized_sole_format_gives_no_part() {
    let (found, read) = gather_sized(&[(F.html, None, html())], 10);
    assert_eq!(parts(found), (Vec::new(), false));
    assert!(read.is_empty(), "nothing is read");
}

#[test]
fn a_picture_alone_over_the_cap_is_too_big() {
    let clip = Fake::new(vec![(F.dib, 100, image())]);
    assert_eq!(clip.gather(10).err(), Some(100));
}

#[test]
fn files_beside_text_are_read_too() {
    let files = ClipPart::Files(Redacted::new(vec![PathBuf::from(r"C:\a.txt")]));
    let clip = Fake::new(vec![(F.files, 30, files.clone()), (F.text, 5, text("kx"))]);
    assert_eq!(parts(clip.gather(100)), (vec![files, text("kx")], false));
}

#[test]
fn a_picture_without_text_keeps_its_place_among_the_parts() {
    let first = Fake::new(vec![(F.dib, 3, image()), (F.html, 5, html())]);
    assert_eq!(parts(first.gather(100)).0, [image(), html()]);
    let last = Fake::new(vec![(F.html, 5, html()), (F.dib, 3, image())]);
    assert_eq!(parts(last.gather(100)).0, [html(), image()]);
}
