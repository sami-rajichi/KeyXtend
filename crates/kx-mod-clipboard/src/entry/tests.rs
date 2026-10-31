//! Entries: the kind rules, the one-line title, the folded search text, the sizes and the copy day.

use kx_module_api::ports::ImageCodec;

use super::*;
use crate::fixture::{CFG, PARIS, app, at, date, files, text};
use crate::hash;

/// An HTML part.
fn html(bytes: &[u8]) -> ClipPart {
    ClipPart::Html(Redacted::new(bytes.to_vec()))
}

/// A PNG image part.
fn png(bytes: &[u8]) -> ClipPart {
    ClipPart::Image {
        codec: ImageCodec::Png,
        bytes: Redacted::new(bytes.to_vec()),
    }
}

/// 2026-10-31 at 16:32 in Paris.
fn now() -> LocalTime {
    at(date(2026, 10, 31), 16, 32)
}

/// The entry for `parts`, copied by no known app at `now`.
fn entry(parts: &[ClipPart]) -> Entry {
    Entry::new(parts, None, &now(), &CFG)
}

#[test]
fn text_is_text_with_its_lines_on_one_title_line() {
    let e = entry(&[text("  Hello\r\nworld\n\n\tagain \u{2028}end\n")]);
    assert_eq!(e.kind, ClipKind::Text);
    assert_eq!(e.title.expose(), "Hello world again end");
}

#[test]
fn a_title_keeps_the_first_title_chars_characters() {
    let e = entry(&[text(&"é".repeat(CFG.title_chars + 50))]);
    assert_eq!(e.title.expose().chars().count(), CFG.title_chars);
}

#[test]
fn one_web_address_is_a_link() {
    for link in [
        "https://example.org/a?b=c",
        " HTTP://x.y/z\r\n",
        "http://localhost:8080",
    ] {
        assert_eq!(entry(&[text(link)]).kind, ClipKind::Link, "{link}");
    }
    for plain in [
        "see https://x.y",
        "https://",
        "ftp://x.y",
        "https://a b",
        "www.x.y",
    ] {
        assert_eq!(entry(&[text(plain)]).kind, ClipKind::Text, "{plain}");
    }
}

#[test]
fn an_image_with_no_text_is_an_image() {
    assert_eq!(entry(&[html(b"<img>"), png(b"px")]).kind, ClipKind::Image);
    assert_eq!(entry(&[png(b"px")]).title.expose(), "");
    let blank = vec![text(" \r\n"), png(b"px")];
    assert_eq!(
        keep_parts(blank.clone()),
        Some(blank.clone()),
        "blank text is no text"
    );
    assert_eq!(entry(&blank).kind, ClipKind::Image);
}

#[test]
fn text_beside_an_image_is_text_and_the_image_goes() {
    let rtf = ClipPart::Rtf(Redacted::new(b"{\\rtf1}".to_vec()));
    let office = vec![text("A1\tB1"), html(b"<td>"), rtf.clone(), png(b"px")];
    let kept = keep_parts(office).unwrap();
    assert_eq!(kept, [text("A1\tB1"), html(b"<td>"), rtf]);
    let cells = entry(&kept);
    assert_eq!(
        (cells.kind, cells.title.expose().as_str()),
        (ClipKind::Text, "A1 B1")
    );
    assert_eq!(cells.stored, 5 + 4 + 7, "the image is not kept");
    assert_eq!(
        entry(&[text("https://x.y"), png(b"px")]).kind,
        ClipKind::Link
    );
}

#[test]
fn a_copy_with_no_part_keeps_no_row() {
    assert_eq!(keep_parts(Vec::new()), None);
    assert_eq!(keep_parts(vec![png(b"px")]), Some(vec![png(b"px")]));
}

#[test]
fn files_are_media_only_when_every_extension_is_listed() {
    let kind = |paths: &[&str]| entry(&[files(paths)]).kind;
    assert_eq!(kind(&["C:/v/Trip.MP4", "C:/a/song.mp3"]), ClipKind::Media);
    assert_eq!(kind(&["C:/v/trip.mp4", "C:/d/notes.txt"]), ClipKind::File);
    assert_eq!(kind(&["C:/v/folder"]), ClipKind::File);
    assert_eq!(kind(&["C:/src/main.ts"]), ClipKind::File);
    assert_eq!(kind(&[]), ClipKind::File);
    let mixed = entry(&[text("x"), png(b"px"), files(&["C:/d/a.txt"])]);
    assert_eq!(mixed.kind, ClipKind::File, "files win");
}

#[test]
fn a_file_entry_keeps_its_paths_and_shows_its_names() {
    let e = entry(&[files(&["C:/x/Report.pdf", "C:/y/École.txt"])]);
    assert_eq!(e.files.expose().len(), 2);
    assert_eq!(e.title.expose(), "Report.pdf École.txt");
    assert!(e.search.expose().contains("ecole.txt"));
    assert!(!e.search.expose().contains("y/"), "names only, not folders");
    assert!(entry(&[text("x")]).files.expose().is_empty());
}

#[test]
fn search_folds_the_text_the_names_and_the_app_apart() {
    let parts = [text("École\nأرسلت")];
    let e = Entry::new(
        &parts,
        Some(app("Bloc-Notes")),
        &at(date(2026, 10, 31), 9, 0),
        &CFG,
    );
    assert_eq!(e.search.expose(), "ecole ارسلت\nbloc-notes");
    assert_eq!(entry(&[text("Hi \t there")]).search.expose(), "hi there");
}

#[test]
fn search_reads_only_the_first_search_chars_characters() {
    let long = format!("{}zz", "a".repeat(CFG.search_chars));
    assert!(!entry(&[text(&long)]).search.expose().contains('z'));
    let names: Vec<String> = (0..CFG.search_chars).map(|n| format!("f{n}.txt")).collect();
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    let many = entry(&[files(&names)]);
    assert!(many.search.expose().chars().count() <= CFG.search_chars);
}

#[test]
fn stored_counts_every_part_and_shown_starts_the_same() {
    let rtf = ClipPart::Rtf(Redacted::new(vec![0; 7]));
    let e = entry(&[text("abc"), html(b"<b>x</b>"), rtf, png(b"12345")]);
    assert_eq!(e.stored, 3 + 8 + 7 + 5);
    assert_eq!(e.shown, e.stored);
}

#[test]
fn a_file_row_shows_the_size_of_the_files_still_there() {
    let mut e = entry(&[files(&["a.txt", "bb.txt", "c.txt"])]);
    assert_eq!(e.stored, 5 + 6 + 5, "the history keeps only the paths");
    e.size_files(&[FileFact::Size(10 * 1024), FileFact::Gone, FileFact::Size(5)]);
    assert_eq!(e.shown, 10 * 1024 + 5);
}

#[test]
fn a_new_entry_sits_in_its_copy_day_unnumbered() {
    let now = at(date(2026, 10, 31), 23, 30);
    let e = Entry::new(&[text("x")], None, &now, &CFG);
    assert_eq!(e.place, Place::Day(date(2026, 10, 31)));
    assert_eq!((e.at.unix_ms, e.at.offset_min), (now.unix_ms, PARIS));
    assert_eq!(e.at.local(), Some(now));
    assert_eq!(Stamp::of(&now), e.at);
    assert!(!e.pinned());
    assert_eq!((e.id, e.seq, e.parts, e.thumb), (0, 0, 0, false));
    assert_eq!(e.hash, hash::of(&[text("x")]));
}

#[test]
fn a_pinned_entry_says_so() {
    let mut e = entry(&[text("x")]);
    e.place = Place::Pinned;
    assert!(e.pinned());
}

#[test]
fn debug_hides_the_text_the_paths_the_app_and_the_hash() {
    let parts = [text("hunter2"), files(&["C:/secret/plan.txt"])];
    let e = Entry::new(&parts, Some(app("vault.exe")), &now(), &CFG);
    let shown = format!("{e:?}");
    let hash = hash::of(&parts).to_string();
    for hidden in ["hunter2", "secret", "plan", "vault", hash.as_str()] {
        assert!(!shown.contains(hidden), "{hidden} leaked");
    }
    assert!(shown.contains("stored"), "the other fields still show");
}

#[test]
fn files_give_the_title_and_search_even_beside_text() {
    let e = entry(&[text("private words"), files(&["C:/x/a.txt", "C:/x/b.txt"])]);
    assert_eq!(e.title.expose(), "a.txt b.txt");
    assert_eq!(e.search.expose(), "a.txt b.txt");
}

#[test]
fn formatted_text_takes_its_title_from_its_plain_text() {
    let rtf = ClipPart::Rtf(Redacted::new(b"{\\rtf1 Bonjour}".to_vec()));
    let e = entry(&[html(b"<p>Bonjour</p>"), rtf.clone(), text("Bonjour")]);
    assert_eq!(
        (e.title.expose().as_str(), e.search.expose().as_str()),
        ("Bonjour", "bonjour")
    );
    let bare = Entry::new(&[html(b"<p>x</p>"), rtf], Some(app("Writer")), &now(), &CFG);
    assert_eq!(bare.kind, ClipKind::Text);
    assert_eq!(
        bare.title.expose(),
        "",
        "the view shows the kind's name instead"
    );
    assert_eq!(bare.search.expose(), "writer");
}
