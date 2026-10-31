//! The view: pages of the configured size, tick numbers, the question's count, Unpin, rounds, the preview and the
//! facts beside; `rows` tests each row's fields.

mod rows;

use kx_module_api::Redacted;
use kx_module_api::ports::{ClipAct, ClipPage, ClipRow, Filter, Pages, PanelKey};

use super::*;
use crate::fixture::{CFG, LIMITS, MB, files, text};
use crate::panel::rig::{PANEL, chars, history, id, now, opened, opened_with};
use crate::panel::{Do, PanelConfig};

/// The storage line's facts: 182 MB kept, 412 GB free on `D:`.
fn facts() -> Facts {
    let room = Room {
        used: 182 * MB,
        free: 412 * 1024 * MB,
        drive: "D:".to_owned(),
    };
    Facts {
        now: now(),
        room,
        note: None,
    }
}

/// The page `p` shows over `h`, with no thumbnails.
fn page(p: &mut Panel, h: &History) -> ClipPage {
    build(p, h, &facts(), &LIMITS, |_| None).0.as_ref().clone()
}

/// The row of entry `id` on `page`.
fn row(page: &ClipPage, id: u64) -> &ClipRow {
    page.rows.iter().find(|r| r.id == id).unwrap()
}

/// Twelve text copies named 1 to 12.
fn twelve() -> History {
    let names: Vec<String> = (1..=12).map(|n| n.to_string()).collect();
    history(&names.iter().map(String::as_str).collect::<Vec<_>>())
}

#[test]
fn a_page_holds_five_rows_and_the_last_one_the_rest() {
    let h = twelve();
    let mut p = opened();
    let first = page(&mut p, &h);
    assert_eq!(first.rows.len(), PANEL.rows);
    assert_eq!(first.pages, Pages { at: 0, count: 3 });
    assert_eq!(first.highlight, Some(0));
    for _ in 0..10 {
        p.key(&PanelKey::Down, &h);
    }
    let last = page(&mut p, &h);
    assert_eq!(last.pages, Pages { at: 2, count: 3 });
    assert_eq!(last.rows.len(), 2);
    assert_eq!(last.highlight, Some(0));
    let ids: Vec<u64> = last.rows.iter().map(|r| r.id).collect();
    assert_eq!(ids, [id(&h, "2"), id(&h, "1")]);
}

#[test]
fn the_page_size_comes_from_the_config() {
    let h = twelve();
    let three = PanelConfig { rows: 3, ..PANEL };
    let v = page(&mut opened_with(three), &h);
    assert_eq!((v.rows.len(), v.pages.count), (3, 4));
    let zero = PanelConfig { rows: 0, ..PANEL };
    let v = page(&mut opened_with(zero), &h);
    assert_eq!((v.rows.len(), v.pages.count), (1, 12), "0 counts as 1");
    let none = page(&mut opened(), &History::default());
    assert_eq!(none.pages, Pages { at: 0, count: 1 });
    assert!(none.rows.is_empty() && none.highlight.is_none() && none.empty);
}

#[test]
fn rows_carry_tick_numbers_and_the_count_holds_in_every_view() {
    let image = ClipPart::Image {
        codec: kx_module_api::ports::ImageCodec::Png,
        bytes: Redacted::new(vec![7]),
    };
    let mut h = history(&["a", "b"]);
    h.add(Entry::new(&[image], None, &now(), &CFG));
    let pic = h.entries()[0].id;
    let mut p = opened();
    p.key(&PanelKey::Tick, &h);
    p.key(&PanelKey::Down, &h);
    p.key(&PanelKey::Down, &h);
    p.key(&PanelKey::Tick, &h);
    let v = page(&mut p, &h);
    assert_eq!(row(&v, pic).tick, Some(1));
    assert_eq!(row(&v, id(&h, "a")).tick, Some(2));
    assert_eq!(row(&v, id(&h, "b")).tick, None);
    p.act(ClipAct::Filter(Filter::Text), &h);
    let v = page(&mut p, &h);
    assert_eq!((v.ticked, v.rows.len()), (2, 2));
    assert_eq!(row(&v, id(&h, "a")).tick, Some(2));
}

#[test]
fn the_question_names_every_unpinned_entry_whatever_the_view() {
    let mut h = history(&["a1", "b2", "c3", "d4"]);
    h.pin(id(&h, "d4"));
    let mut p = opened();
    p.key(&chars("b2"), &h);
    assert_eq!(page(&mut p, &h).confirm, None);
    p.act(ClipAct::DeleteAll, &h);
    let v = page(&mut p, &h);
    assert_eq!((v.confirm, v.rows.len()), (Some(3), 1));
}

#[test]
fn unpin_shows_when_every_ticked_or_previewed_entry_is_pinned() {
    let mut h = history(&["a", "b"]);
    let mut p = opened();
    assert!(!page(&mut p, &h).unpin, "no tick");
    h.pin(id(&h, "b"));
    p.key(&PanelKey::Tick, &h);
    assert!(page(&mut p, &h).unpin);
    p.key(&PanelKey::Down, &h);
    p.key(&PanelKey::Tick, &h);
    assert!(!page(&mut p, &h).unpin);
    p.show(id(&h, "b"), Preview::Image(None));
    let v = page(&mut p, &h);
    assert!(v.unpin);
    assert_eq!(v.preview, Some(Preview::Image(None)));
}

#[test]
fn the_facts_limits_and_search_pass_through() {
    let h = history(&["a"]);
    let mut p = opened();
    p.key(&chars("zz"), &h);
    let mut f = facts();
    f.note = Some(PanelNote::Disk);
    let v = build(&mut p, &h, &f, &LIMITS, |_| None);
    assert_eq!(
        (v.0.room.clone(), v.0.note),
        (f.room, Some(PanelNote::Disk))
    );
    assert_eq!(
        (v.0.cap, v.0.kept_days),
        (LIMITS.max_bytes, LIMITS.keep_days)
    );
    assert_eq!(v.0.search.expose(), "zz");
    assert_eq!(v.0.filter, Filter::All);
    assert!(v.0.rows.is_empty() && v.0.highlight.is_none() && !v.0.empty);
    assert!(v.0.preview.is_none());
}

#[test]
fn each_view_counts_a_round_and_a_click_ticks_the_row_it_saw() {
    let h = history(&["a", "b", "c"]);
    let mut p = opened();
    let old = page(&mut p, &h);
    let new = page(&mut p, &h);
    assert_eq!(new.round, old.round.wrapping_add(1));
    let tick = |round| ClipAct::Tick { round, row: 1 };
    assert_eq!(p.act(tick(old.round), &h), Do::Stay);
    assert_eq!(p.act(tick(new.round), &h), Do::Show);
    assert_eq!(p.ticks(), [new.rows[1].id]);
}

#[test]
fn a_text_preview_is_cut_to_its_length_and_counts_every_character() {
    let parts = [text("مرحبا بالعالم")];
    let e = Entry::new(&parts, None, &now(), &CFG);
    let cut = Preview::Text {
        text: Redacted::new("مرحبا".to_owned()),
        chars: 13,
    };
    assert_eq!(preview(&e, &parts, 5), cut);
    let paths = [files(&["C:/a.txt", "C:/b.txt"])];
    let f = Entry::new(&paths, None, &now(), &CFG);
    assert_eq!(preview(&f, &paths, 5), Preview::Files(f.files.clone()));
    let html = [ClipPart::Html(Redacted::new(b"<i>x</i>".to_vec()))];
    let blank = Preview::Text {
        text: Redacted::new(String::new()),
        chars: 0,
    };
    assert_eq!(
        preview(&Entry::new(&html, None, &now(), &CFG), &html, 5),
        blank
    );
}

#[test]
fn an_image_preview_waits_for_its_drawing() {
    let image = [ClipPart::Image {
        codec: kx_module_api::ports::ImageCodec::Dib,
        bytes: Redacted::new(vec![1]),
    }];
    let e = Entry::new(&image, None, &now(), &CFG);
    assert_eq!(preview(&e, &image, 5), Preview::Image(None));
}
