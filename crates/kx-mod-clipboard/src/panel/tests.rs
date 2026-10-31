//! The panel's keys over a small history, with the plan's page of 5 rows; `acts` tests the ticks and the footer.

mod acts;

use kx_module_api::Redacted;
use kx_module_api::ports::{ClipAct, Filter, PanelKey, Preview};

use super::rig::{PANEL, chars, history, id, opened, opened_with};
use super::*;
use crate::fixture::{at, copied, date};

/// Some preview content.
fn content() -> Preview {
    Preview::Text {
        text: Redacted::new("text".to_owned()),
        chars: 4,
    }
}

/// The ids of the entries titled `titles`, in that order.
fn ids(h: &History, titles: &[&str]) -> Vec<u64> {
    titles.iter().map(|t| id(h, t)).collect()
}

#[test]
fn esc_clears_the_search_then_closes_the_panel() {
    let h = history(&["alpha", "beta"]);
    let mut p = opened();
    assert_eq!(p.key(&chars("al"), &h), Do::Show);
    assert_eq!(p.query().expose(), "al");
    assert_eq!(p.key(&PanelKey::Esc, &h), Do::Show);
    assert_eq!(p.query().expose(), "");
    assert!(p.is_open());
    assert_eq!(p.key(&PanelKey::Esc, &h), Do::Close);
    assert!(!p.is_open());
}

#[test]
fn esc_backs_out_of_the_question_then_the_preview_then_the_search() {
    let h = history(&["alpha", "beta"]);
    let mut p = opened();
    p.key(&chars("a"), &h);
    assert_eq!(p.act(ClipAct::Preview, &h), Do::Preview(id(&h, "beta")));
    p.show(id(&h, "beta"), content());
    assert_eq!(p.act(ClipAct::DeleteAll, &h), Do::Show);
    assert!(p.asking());
    assert_eq!(p.key(&PanelKey::Esc, &h), Do::Show);
    assert!(!p.asking() && p.preview().is_some());
    assert_eq!(p.key(&PanelKey::Esc, &h), Do::Show);
    assert!(p.preview().is_none());
    assert_eq!(p.query().expose(), "a");
    assert_eq!(p.key(&PanelKey::Esc, &h), Do::Show);
    assert_eq!(p.key(&PanelKey::Esc, &h), Do::Close);
}

#[test]
fn enter_inserts_the_ticked_rows_else_the_highlighted_one() {
    let h = history(&["a", "b", "c"]);
    let mut p = opened();
    p.key(&PanelKey::Down, &h);
    assert_eq!(p.key(&PanelKey::Enter, &h), Do::Insert(ids(&h, &["b"])));
    p.key(&PanelKey::Tick, &h);
    p.key(&PanelKey::Down, &h);
    p.key(&PanelKey::Tick, &h);
    assert_eq!(
        p.key(&PanelKey::Enter, &h),
        Do::Insert(ids(&h, &["b", "a"]))
    );
    let empty = History::default();
    assert_eq!(opened().key(&PanelKey::Enter, &empty), Do::Stay);
    assert_eq!(opened().act(ClipAct::Insert, &h), Do::Stay, "no tick");
}

#[test]
fn up_and_down_move_the_highlight_and_stop_at_the_ends() {
    let h = history(&["1", "2", "3", "4", "5", "6", "7"]);
    let mut p = opened();
    assert_eq!(p.key(&PanelKey::Up, &h), Do::Stay);
    for _ in 0..5 {
        assert_eq!(p.key(&PanelKey::Down, &h), Do::Show);
    }
    assert_eq!(p.key(&PanelKey::Enter, &h), Do::Insert(ids(&h, &["2"])));
    p.key(&PanelKey::Down, &h);
    assert_eq!(p.key(&PanelKey::Down, &h), Do::Stay);
    assert_eq!(p.highlight(&p.list(&h)), Some(6));
}

#[test]
fn page_clicks_turn_to_the_first_row_of_a_page_and_stop_at_the_ends() {
    let names: Vec<String> = (1..=12).map(|n| n.to_string()).collect();
    let h = history(&names.iter().map(String::as_str).collect::<Vec<_>>());
    let mut p = opened();
    assert_eq!(p.act(ClipAct::Page { back: true }, &h), Do::Stay);
    assert_eq!(p.act(ClipAct::Page { back: false }, &h), Do::Show);
    assert_eq!(p.key(&PanelKey::Enter, &h), Do::Insert(ids(&h, &["7"])));
    p.key(&PanelKey::Down, &h);
    assert_eq!(p.act(ClipAct::Page { back: false }, &h), Do::Show);
    assert_eq!(p.key(&PanelKey::Enter, &h), Do::Insert(ids(&h, &["2"])));
    assert_eq!(p.act(ClipAct::Page { back: false }, &h), Do::Stay);
    assert_eq!(p.act(ClipAct::Page { back: true }, &h), Do::Show);
    assert_eq!(p.highlight(&p.list(&h)), Some(5));
}

#[test]
fn left_and_right_step_through_the_filters_and_stop_at_the_ends() {
    let h = history(&["a"]);
    let mut p = opened();
    assert_eq!(p.key(&PanelKey::Left, &h), Do::Stay);
    for f in &Filter::ORDER[1..] {
        assert_eq!(p.key(&PanelKey::Right, &h), Do::Show);
        assert_eq!(p.filter(), *f);
    }
    assert_eq!(p.key(&PanelKey::Right, &h), Do::Stay);
    assert_eq!(p.key(&PanelKey::Left, &h), Do::Show);
    assert_eq!(p.filter(), Filter::Files);
    assert_eq!(p.act(ClipAct::Filter(Filter::Files), &h), Do::Stay);
}

#[test]
fn the_search_types_erases_and_brings_the_highlight_to_the_top() {
    let h = history(&["a b", "a bc", "z"]);
    let mut p = opened();
    p.key(&PanelKey::Down, &h);
    assert_eq!(p.key(&chars("a\u{7}\r\nb"), &h), Do::Show);
    assert_eq!(p.query().expose(), "a b", "one line");
    assert_eq!(p.key(&PanelKey::Enter, &h), Do::Insert(ids(&h, &["a bc"])));
    assert_eq!(p.key(&PanelKey::Erase, &h), Do::Show);
    assert_eq!(p.query().expose(), "a ");
    p.key(&PanelKey::Erase, &h);
    p.key(&PanelKey::Erase, &h);
    assert_eq!(p.key(&PanelKey::Erase, &h), Do::Stay);
    assert_eq!(p.key(&chars(""), &h), Do::Stay);
}

#[test]
fn the_search_keeps_at_most_query_chars_characters() {
    let h = history(&["a"]);
    let mut p = opened_with(PanelConfig {
        query_chars: 3,
        ..PANEL
    });
    assert_eq!(p.key(&chars("éàüö"), &h), Do::Show);
    assert_eq!(p.query().expose(), "éàü");
    assert_eq!(p.key(&chars("e"), &h), Do::Stay);
}

#[test]
fn the_highlight_stays_on_its_entry_when_a_copy_arrives() {
    let mut h = history(&["a", "b", "c"]);
    let mut p = opened();
    p.key(&PanelKey::Down, &h);
    h.add(copied("d", &at(date(2026, 10, 31), 16, 40)));
    assert_eq!(p.key(&PanelKey::Enter, &h), Do::Insert(ids(&h, &["b"])));
    h.remove(id(&h, "b"));
    assert_eq!(p.key(&PanelKey::Enter, &h), Do::Insert(ids(&h, &["a"])));
}

#[test]
fn a_closed_panel_takes_nothing_and_opens_fresh() {
    let h = history(&["a", "b"]);
    let mut p = Panel::new(PANEL);
    assert!(!p.is_open());
    assert_eq!(p.key(&PanelKey::Down, &h), Do::Stay);
    assert_eq!(p.act(ClipAct::Insert, &h), Do::Stay);
    p.show(id(&h, "a"), content());
    assert!(p.preview().is_none());
    p.open();
    p.key(&chars("a"), &h);
    p.act(ClipAct::Filter(Filter::Text), &h);
    p.key(&PanelKey::Tick, &h);
    let round = p.shown(ids(&h, &["a"]));
    assert_eq!(p.act(ClipAct::Close, &h), Do::Close);
    p.open();
    assert!(p.query().expose().is_empty() && p.ticks().is_empty());
    assert_eq!(p.filter(), Filter::All);
    assert_eq!(p.act(ClipAct::Tick { round, row: 0 }, &h), Do::Stay);
}
