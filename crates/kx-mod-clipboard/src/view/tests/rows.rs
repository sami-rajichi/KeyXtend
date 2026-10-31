//! The view's rows: local days, the expiry chip, the title's direction and font, the app, size, pin and thumbnail.

use std::cell::Cell;

use kx_module_api::ports::{ClipKind, LocalTime, Pixels};

use super::*;
use crate::fixture::{MB, app, at, date, files};

/// A history of `entries`, the first added first.
fn kept(entries: Vec<Entry>) -> History {
    let mut h = History::default();
    for e in entries {
        h.add(e);
    }
    h
}

/// A text copy of `s` made at `when`.
fn copy_at(s: &str, when: &LocalTime) -> Entry {
    Entry::new(&[text(s)], None, when, &CFG)
}

/// A video copied at `when`.
fn video_at(when: &LocalTime) -> Entry {
    Entry::new(&[files(&["C:/Films/clip.mp4"])], None, when, &CFG)
}

#[test]
fn times_read_by_local_day_and_a_clock_set_back_reads_today() {
    let days = [(31, 9, 5), (30, 18, 21), (27, 14, 12), (1, 8, 0)];
    let mut entries: Vec<Entry> = days
        .iter()
        .map(|&(d, hh, mm)| copy_at(&format!("{d}"), &at(date(2026, 10, d), hh, mm)))
        .collect();
    entries.push(copy_at("later", &at(date(2026, 11, 2), 7, 30)));
    let h = kept(entries);
    let v = page(&mut opened(), &h);
    let when = |t: &str| row(&v, id(&h, t)).when;
    let today = when("31");
    assert_eq!((today.days_ago, today.hour, today.minute), (0, 9, 5));
    assert_eq!(when("30").days_ago, 1);
    assert_eq!(
        (when("27").days_ago, when("27").date),
        (4, date(2026, 10, 27))
    );
    assert_eq!(when("1").days_ago, 30);
    assert_eq!((when("later").days_ago, when("later").hour), (0, 7));
}

#[test]
fn media_on_its_last_day_shows_the_expiry_chip() {
    let yesterday = at(date(2026, 10, 30), 10, 0);
    let mut h = kept(vec![
        video_at(&now()),
        video_at(&yesterday),
        copy_at("t", &now()),
    ]);
    let pinned = h.add(video_at(&now()));
    h.pin(pinned);
    let ids: Vec<u64> = h.entries().iter().map(|e| e.id).collect();
    let flags = |limits: &Limits| {
        let v = build(&mut opened(), &h, &facts(), limits, |_| None);
        ids.iter()
            .map(|&i| row(&v.0, i).expires)
            .collect::<Vec<_>>()
    };
    let all = [false, false, true, true];
    assert_eq!(flags(&LIMITS), all, "pinned, text, yesterday, today");
    let two = Limits {
        media_days: 2,
        ..*LIMITS
    };
    assert_eq!(flags(&two), [false, false, true, false]);
    let short = Limits {
        keep_days: 1,
        media_days: 7,
        ..*LIMITS
    };
    assert_eq!(flags(&short), all);
}

#[test]
fn an_html_only_copy_keeps_an_empty_title_for_the_kinds_name() {
    let html = ClipPart::Html(Redacted::new(b"<b>x</b>".to_vec()));
    let h = kept(vec![Entry::new(&[html], None, &now(), &CFG)]);
    let v = page(&mut opened(), &h);
    let r = &v.rows[0];
    assert_eq!(r.kind, ClipKind::Text);
    assert!(r.title.expose().is_empty() && !r.rtl && !r.arabic);
}

#[test]
fn titles_set_their_direction_and_font() {
    let h = history(&["مرحبا بالعالم", "123 שלום", "hello مرحبا", "hello"]);
    let v = page(&mut opened(), &h);
    let flags = |t: &str| (row(&v, id(&h, t)).rtl, row(&v, id(&h, t)).arabic);
    assert_eq!(flags("مرحبا بالعالم"), (true, true));
    assert_eq!(flags("123 שלום"), (true, false));
    assert_eq!(flags("hello مرحبا"), (false, true));
    assert_eq!(flags("hello"), (false, false));
}

#[test]
fn rows_carry_the_app_kind_size_and_pin() {
    let link = Entry::new(
        &[text("https://example.com")],
        Some(app("Edge")),
        &now(),
        &CFG,
    );
    let mut h = kept(vec![link]);
    let link_id = h.entries()[0].id;
    h.update(link_id, |e| e.shown = Some(3 * MB));
    h.pin(link_id);
    let v = page(&mut opened(), &h);
    let r = row(&v, link_id);
    assert_eq!(r.app.as_ref().map(|a| a.expose().as_str()), Some("Edge"));
    let want = (ClipKind::Link, Some(3 * MB), true);
    assert_eq!((r.kind, r.bytes, r.pinned), want);
    assert_eq!(r.title.expose(), "https://example.com");
}

#[test]
fn thumbnails_are_asked_only_for_rows_that_have_one() {
    let mut h = history(&["a", "b"]);
    let drawn = id(&h, "a");
    h.update(drawn, |e| e.thumb = true);
    let asked = Cell::new(0);
    let pixels = Pixels::new(1, 1, vec![0; 4], 1).unwrap();
    let v = build(&mut opened(), &h, &facts(), &LIMITS, |e| {
        asked.set(asked.get() + 1);
        (e.id == drawn).then(|| pixels.clone())
    });
    assert_eq!(asked.get(), 1);
    assert_eq!(row(&v.0, drawn).thumb, Some(pixels));
    assert_eq!(row(&v.0, id(&h, "b")).thumb, None);
}
