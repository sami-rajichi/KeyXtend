//! The panel's ticks and footer: tick order across views, Delete all and its question, Pin and the preview.

use super::*;

#[test]
fn ticks_keep_their_order_across_searches_and_filters() {
    let h = history(&["first note", "second idea", "third note"]);
    let mut p = opened();
    p.key(&chars("IDEA"), &h);
    assert_eq!(p.key(&PanelKey::Tick, &h), Do::Show);
    p.key(&PanelKey::Esc, &h);
    p.key(&chars("first"), &h);
    p.key(&PanelKey::Tick, &h);
    p.act(ClipAct::Filter(Filter::Images), &h);
    p.act(ClipAct::Filter(Filter::All), &h);
    let want = ids(&h, &["second idea", "first note"]);
    assert_eq!(p.ticks(), want);
    assert_eq!(p.act(ClipAct::Insert, &h), Do::Insert(want));
}

#[test]
fn unticking_closes_the_gap_in_the_tick_order() {
    let h = history(&["a", "b", "c"]);
    let mut p = opened();
    for _ in 0..3 {
        p.key(&PanelKey::Tick, &h);
        p.key(&PanelKey::Down, &h);
    }
    assert_eq!(p.ticks(), ids(&h, &["c", "b", "a"]));
    p.key(&PanelKey::Up, &h);
    assert_eq!(p.key(&PanelKey::Tick, &h), Do::Show);
    assert_eq!(p.ticks(), ids(&h, &["c", "a"]));
}

#[test]
fn ticks_drop_when_their_entry_goes() {
    let mut h = history(&["a", "b", "c"]);
    let mut p = opened();
    p.key(&PanelKey::Tick, &h);
    p.key(&PanelKey::Down, &h);
    p.key(&PanelKey::Tick, &h);
    h.remove(id(&h, "c"));
    assert_eq!(p.act(ClipAct::Insert, &h), Do::Insert(ids(&h, &["b"])));
    assert_eq!(p.ticks(), ids(&h, &["b"]));
}

#[test]
fn a_click_ticks_the_row_of_the_view_it_saw_and_an_older_view_is_ignored() {
    let h = history(&["a", "b", "c"]);
    let mut p = opened();
    let round = p.shown(ids(&h, &["c", "b", "a"]));
    assert_eq!(p.act(ClipAct::Tick { round, row: 1 }, &h), Do::Show);
    assert_eq!(p.ticks(), ids(&h, &["b"]));
    assert_eq!(p.key(&PanelKey::Enter, &h), Do::Insert(ids(&h, &["b"])));
    let stale = ClipAct::Tick {
        round: round.wrapping_sub(1),
        row: 0,
    };
    assert_eq!(p.act(stale, &h), Do::Stay);
    assert_eq!(p.act(ClipAct::Tick { round, row: 3 }, &h), Do::Stay);
    let next = p.shown(ids(&h, &["c"]));
    assert_ne!(next, round);
    assert_eq!(p.act(ClipAct::Tick { round, row: 0 }, &h), Do::Stay);
}

#[test]
fn delete_all_takes_every_unpinned_entry_whatever_the_view() {
    let mut h = history(&["a1", "b2", "c3", "d4"]);
    h.pin(id(&h, "d4"));
    let mut p = opened();
    p.key(&chars("b2"), &h);
    p.act(ClipAct::Filter(Filter::Text), &h);
    assert_eq!(p.act(ClipAct::DeleteAll, &h), Do::Show);
    assert!(p.asking());
    let Do::Delete(mut gone) = p.act(ClipAct::Confirm(true), &h) else {
        panic!("Yes deletes");
    };
    gone.sort_unstable();
    let mut want = ids(&h, &["a1", "b2", "c3"]);
    want.sort_unstable();
    assert_eq!(gone, want);
    assert!(!p.asking());
}

#[test]
fn no_keeps_everything_and_a_history_of_pins_asks_nothing() {
    let mut h = history(&["a", "b"]);
    let mut p = opened();
    p.act(ClipAct::DeleteAll, &h);
    assert_eq!(p.act(ClipAct::Confirm(false), &h), Do::Show);
    assert!(!p.asking());
    assert_eq!(p.act(ClipAct::Confirm(true), &h), Do::Stay, "no question");
    h.pin(id(&h, "a"));
    h.pin(id(&h, "b"));
    assert_eq!(p.act(ClipAct::DeleteAll, &h), Do::Stay);
    assert!(!p.asking());
}

#[test]
fn yes_deletes_only_what_the_question_counted() {
    let mut h = history(&["a", "b", "c"]);
    let mut p = opened();
    p.act(ClipAct::DeleteAll, &h);
    assert_eq!(p.asked().map(<[u64]>::len), Some(3));
    h.add(copied("late", &at(date(2026, 10, 31), 16, 40)));
    h.remove(id(&h, "a"));
    h.pin(id(&h, "b"));
    p.sync(&h);
    let left = ids(&h, &["c"]);
    assert_eq!(p.asked(), Some(&left[..]), "gone and pinned leave it");
    assert_eq!(p.act(ClipAct::Confirm(true), &h), Do::Delete(left));
    p.act(ClipAct::DeleteAll, &h);
    h.remove(id(&h, "c"));
    h.remove(id(&h, "late"));
    p.sync(&h);
    assert!(!p.asking(), "nothing it counted is left");
}

#[test]
fn the_question_takes_only_a_click_on_its_answer_esc_and_close() {
    let h = history(&["a", "b"]);
    let mut p = opened();
    p.act(ClipAct::DeleteAll, &h);
    assert_eq!(p.act(ClipAct::Insert, &h), Do::Stay);
    assert_eq!(p.act(ClipAct::Tick { round: 0, row: 0 }, &h), Do::Stay);
    assert_eq!(p.act(ClipAct::Preview, &h), Do::Stay);
    assert_eq!(p.key(&PanelKey::Down, &h), Do::Stay);
    assert_eq!(p.key(&chars("x"), &h), Do::Stay);
    assert!(p.query().expose().is_empty() && p.ticks().is_empty());
    assert_eq!(p.key(&PanelKey::Enter, &h), Do::Stay, "a click says yes");
    assert!(p.asking());
    assert!(matches!(p.act(ClipAct::Confirm(true), &h), Do::Delete(ids) if ids.len() == 2));
    p.act(ClipAct::DeleteAll, &h);
    assert_eq!(p.key(&PanelKey::Esc, &h), Do::Show, "Esc says no");
    assert!(!p.asking());
    p.act(ClipAct::DeleteAll, &h);
    assert_eq!(p.act(ClipAct::Close, &h), Do::Close);
    assert!(!p.is_open() && !p.asking());
}

#[test]
fn pin_reads_unpin_when_every_target_is_pinned() {
    let mut h = history(&["a", "b"]);
    let mut p = opened();
    assert_eq!(p.act(ClipAct::Pin, &h), Do::Stay, "no tick");
    p.key(&PanelKey::Tick, &h);
    p.key(&PanelKey::Down, &h);
    p.key(&PanelKey::Tick, &h);
    let both = ids(&h, &["b", "a"]);
    h.pin(both[0]);
    assert!(!p.unpins(&h));
    assert_eq!(p.act(ClipAct::Pin, &h), Do::Pin(both.clone()));
    h.pin(both[1]);
    assert!(p.unpins(&h));
    assert_eq!(p.act(ClipAct::Pin, &h), Do::Unpin(both));
}

#[test]
fn the_preview_takes_the_last_tick_else_the_highlight_and_acts_on_it() {
    let h = history(&["a", "b", "c"]);
    let mut p = opened();
    assert_eq!(p.act(ClipAct::Preview, &h), Do::Preview(id(&h, "c")));
    assert!(p.preview().is_none(), "opens once shown");
    p.key(&PanelKey::Tick, &h);
    p.key(&PanelKey::Down, &h);
    p.key(&PanelKey::Tick, &h);
    let b = id(&h, "b");
    assert_eq!(p.act(ClipAct::Preview, &h), Do::Preview(b));
    p.show(b, content());
    assert_eq!(p.preview(), Some((b, &content())));
    assert!(!p.unpins(&h));
    assert_eq!(p.act(ClipAct::Insert, &h), Do::Insert(vec![b]));
    assert_eq!(p.act(ClipAct::Delete, &h), Do::Delete(vec![b]));
    assert_eq!(p.act(ClipAct::Pin, &h), Do::Pin(vec![b]));
    assert_eq!(p.key(&PanelKey::Enter, &h), Do::Insert(vec![b]));
    assert_eq!(p.act(ClipAct::Page { back: false }, &h), Do::Stay);
    assert_eq!(p.act(ClipAct::Filter(Filter::Text), &h), Do::Stay);
    assert_eq!(p.key(&PanelKey::Down, &h), Do::Stay);
    assert_eq!(p.key(&PanelKey::Tick, &h), Do::Stay);
    assert_eq!(p.key(&chars("x"), &h), Do::Stay);
    assert_eq!(p.act(ClipAct::Back, &h), Do::Show);
    assert_eq!(p.act(ClipAct::Back, &h), Do::Stay);
    assert_eq!(p.ticks(), ids(&h, &["c", "b"]));
}

#[test]
fn a_preview_closes_when_its_entry_goes_and_a_late_redraw_waits_for_it() {
    let mut h = history(&["a", "b"]);
    let mut p = opened();
    let a = id(&h, "a");
    assert!(!p.redraw(a, content()), "no preview");
    p.show(a, Preview::Image(None));
    assert!(!p.redraw(id(&h, "b"), content()));
    assert!(p.redraw(a, content()));
    assert_eq!(p.preview(), Some((a, &content())));
    h.remove(a);
    p.sync(&h);
    assert!(p.preview().is_none());
}

#[test]
fn the_uis_rows_page_the_list_keep_the_highlight_and_hold_while_closed() {
    let h = history(&["a", "b", "c", "d", "e", "f", "g"]);
    let mut p = opened();
    for _ in 0..3 {
        p.key(&PanelKey::Down, &h);
    }
    assert_eq!(p.act(ClipAct::Rows(2), &h), Do::Show);
    assert_eq!(p.rows(), 2);
    let list = p.list(&h);
    assert_eq!(
        p.highlight(&list),
        Some(3),
        "d stays highlighted, on page 2"
    );
    assert_eq!(p.act(ClipAct::Rows(2), &h), Do::Stay, "the same rows");
    assert_eq!(p.act(ClipAct::Rows(0), &h), Do::Show);
    assert_eq!(p.rows(), 1, "at least one");
    p.act(ClipAct::DeleteAll, &h);
    assert_eq!(
        p.act(ClipAct::Rows(5), &h),
        Do::Show,
        "under the question too"
    );
    assert!(p.asking());
    p.close();
    assert_eq!(p.act(ClipAct::Rows(3), &h), Do::Stay, "nothing to draw");
    p.open();
    assert_eq!(p.rows(), 3, "kept for the next open");
}

#[test]
fn fewer_rows_move_the_page_to_the_highlighted_entry() {
    use crate::view::{Facts, build};
    use kx_module_api::ports::Room;
    let names: Vec<String> = (1..=12).map(|n| n.to_string()).collect();
    let h = history(&names.iter().map(String::as_str).collect::<Vec<_>>());
    let mut p = opened();
    for _ in 0..7 {
        p.key(&PanelKey::Down, &h);
    }
    let room = Room {
        used: 0,
        free: 0,
        drive: String::new(),
    };
    let facts = Facts {
        now: crate::panel::rig::now(),
        room,
        note: None,
    };
    let page = |p: &mut Panel| build(p, &h, &facts, &crate::fixture::LIMITS, |_| None).0;
    let before = page(&mut p);
    assert_eq!((before.pages.at, before.highlight), (1, Some(2)));
    assert_eq!(p.act(ClipAct::Rows(3), &h), Do::Show);
    let after = page(&mut p);
    assert_eq!(
        (after.pages.at, after.highlight),
        (2, Some(1)),
        "place 7 of rows of 3"
    );
    assert_eq!(
        after.rows[1].id,
        id(&h, "5"),
        "the same entry stays highlighted"
    );
}
