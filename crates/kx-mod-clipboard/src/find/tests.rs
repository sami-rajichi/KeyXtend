//! Search and filters: the query folds as the entries' search text does, and each filter shows its kinds.

use kx_module_api::ports::{ClipPart, ImageCodec, LocalTime};

use super::*;
use crate::fixture::{CFG, app, at, copied, date, files, text};

/// 2026-10-31 at 16:32 in Paris.
fn now() -> LocalTime {
    at(date(2026, 10, 31), 16, 32)
}

/// True when `query` under the All filter finds a text copy of `copy`.
fn finds(query: &str, copy: &str) -> bool {
    Find::new(Filter::All, query).keeps(&copied(copy, &now()))
}

/// One entry of each kind: text, link, image, files and video, in that order.
fn one_of_each() -> Vec<Entry> {
    let image = ClipPart::Image {
        codec: ImageCodec::Png,
        bytes: Redacted::new(vec![1, 2, 3]),
    };
    let parts = [
        vec![text("a note")],
        vec![text("https://example.com/note")],
        vec![image],
        vec![files(&["C:/Docs/note.pdf"])],
        vec![files(&["C:/Films/note.mp4"])],
    ];
    parts
        .iter()
        .map(|p| Entry::new(p, None, &now(), &CFG))
        .collect()
}

#[test]
fn capitals_find_accents_and_accents_find_capitals() {
    assert!(finds("ECOLE", "Une école de musique"));
    assert!(finds("école", "ÉCOLE"));
    assert!(!finds("ecoles", "école"));
}

#[test]
fn arabic_finds_past_the_alef_forms_harakat_and_tatweel() {
    assert!(finds("ارسلت", "أرسلت الملف"));
    assert!(finds("أرسلت", "ارسلت"));
    assert!(finds("ارسلت", "أَرْسَلْتُ"));
    assert!(finds("ارسلت", "أرســلت"));
    assert!(finds("على", "علي الطريق"));
    assert!(!finds("ارسل", "رسالة"));
}

#[test]
fn a_pasted_query_with_line_breaks_matches_as_one_line() {
    assert!(finds("hello\r\n\n world", "hello world"));
    assert!(finds("hello world", "hello\n\n\tworld"));
}

#[test]
fn blanks_around_the_query_do_not_count_and_a_blank_one_finds_all() {
    assert!(finds("  hello \n", "hello"));
    assert!(finds("", "anything"));
    assert!(finds(" \n ", "anything"));
}

#[test]
fn app_and_file_names_are_found_but_never_across_two_pieces() {
    let note = Entry::new(&[text("rapport final")], Some(app("Notepad")), &now(), &CFG);
    assert!(Find::new(Filter::All, "NOTEPAD").keeps(&note));
    assert!(!Find::new(Filter::All, "final notepad").keeps(&note));
    let doc = Entry::new(&[files(&["C:/Docs/Bilan.pdf"])], None, &now(), &CFG);
    assert!(Find::new(Filter::All, "bilan").keeps(&doc));
}

#[test]
fn each_filter_shows_its_kinds() {
    use ClipKind::{File, Image, Link, Media, Text};
    let kinds = [Text, Link, Image, File, Media];
    let shown = |f| {
        kinds
            .into_iter()
            .filter(|&k| shows(f, k))
            .collect::<Vec<_>>()
    };
    assert_eq!(shown(Filter::All), kinds);
    assert_eq!(shown(Filter::Text), [Text, Link]);
    assert_eq!(shown(Filter::Images), [Image]);
    assert_eq!(shown(Filter::Files), [File]);
    assert_eq!(shown(Filter::Media), [Media]);
}

#[test]
fn a_find_lists_what_its_filter_and_query_keep_in_order() {
    let all = one_of_each();
    let kinds = |f: Filter, q: &str| {
        let found = Find::new(f, q).list(&all);
        found.iter().map(|e| e.kind).collect::<Vec<_>>()
    };
    let every = all.iter().map(|e| e.kind).collect::<Vec<_>>();
    assert_eq!(kinds(Filter::All, ""), every);
    assert_eq!(
        kinds(Filter::All, "NOTE"),
        [every[0], every[1], every[3], every[4]]
    );
    assert_eq!(
        kinds(Filter::Text, "note"),
        [ClipKind::Text, ClipKind::Link]
    );
    assert_eq!(kinds(Filter::Media, "mp4"), [ClipKind::Media]);
    assert!(kinds(Filter::Images, "note").is_empty());
}
