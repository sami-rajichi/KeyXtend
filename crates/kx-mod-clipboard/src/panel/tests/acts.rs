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
fn the_question_takes_only_its_answer_esc_and_close() {
    let h = history(&["a", "b"]);
    let mut p = opened();
    p.act(ClipAct::DeleteAll, &h);
    assert_eq!(p.act(ClipAct::Insert, &h), Do::Stay);
    assert_eq!(p.act(ClipAct::Tick { round: 0, row: 0 }, &h), Do::Stay);
    assert_eq!(p.act(ClipAct::Preview, &h), Do::Stay);
    assert_eq!(p.key(&PanelKey::Down, &h), Do::Stay);
    assert_eq!(p.key(&chars("x"), &h), Do::Stay);
    assert!(p.query().expose().is_empty() && p.ticks().is_empty());
    assert!(matches!(p.key(&PanelKey::Enter, &h), Do::Delete(ids) if ids.len() == 2));
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
