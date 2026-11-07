//! The search's caret: keys and chips act at it, Left and Right never switch filters while the line holds text, a click
//! places it, and only a change of text moves the list; `form` tests the caret of New and Edit.

mod form;

use kx_module_api::Redacted;
use kx_module_api::ports::{FillPage, PanelLine};

use super::*;

/// Beh with a fatha, then teh, which reads right to left.
const BEH_FATHA_TEH: &str = "\u{628}\u{64e}\u{62a}";

/// The search's caret on the last view.
fn at(r: &Rig) -> u32 {
    *r.page.caret.expose()
}

/// A click that places the caret at `at`.
const fn place(at: u32) -> FillClick {
    FillClick::Caret {
        at: Redacted::new(at),
    }
}

/// The key of a chip that erases `erase` characters, then types `text`.
fn word(erase: u16, text: &str) -> PanelKey {
    PanelKey::Word {
        erase,
        text: Redacted::new(text.to_owned()),
    }
}

/// The search line holding `text`.
fn search(text: &str) -> PanelLine {
    PanelLine::Search(Redacted::new(text.to_owned()))
}

/// The search text.
fn searched(page: &FillPage) -> &str {
    page.search.expose()
}

#[test]
fn the_search_starts_empty_and_follows_what_is_typed() {
    let mut r = Rig::sample();
    assert_eq!(at(&r), 0);
    r.text("ban");
    assert_eq!(at(&r), 3);
}

#[test]
fn left_and_right_move_the_search_caret_and_never_the_filter_while_it_holds_text() {
    let mut r = Rig::sample();
    r.click(FillClick::Filter(FillFilter::Logins));
    r.text("ab");
    assert_eq!(r.key(&PanelKey::Right), Do::Stay, "at the end");
    assert_eq!(r.page.filter, FillFilter::Logins);
    assert_eq!(r.key(&PanelKey::Left), Do::Show);
    assert_eq!(at(&r), 1);
    r.key(&PanelKey::Left);
    assert_eq!(r.key(&PanelKey::Left), Do::Stay, "at the start");
    assert_eq!(at(&r), 0);
    assert_eq!(r.page.filter, FillFilter::Logins);
}

#[test]
fn an_empty_search_still_switches_filters() {
    let mut r = Rig::sample();
    assert_eq!(r.key(&PanelKey::Right), Do::Show);
    assert_eq!(r.page.filter, FillFilter::People);
    assert_eq!(r.key(&PanelKey::Left), Do::Show);
    assert_eq!(r.page.filter, FillFilter::All);
    r.text("a");
    r.key(&PanelKey::Erase);
    r.key(&PanelKey::Right);
    assert_eq!(r.page.filter, FillFilter::People, "erased empty again");
}

#[test]
fn a_right_to_left_search_takes_left_as_on_and_steps_over_its_harakat() {
    let mut r = Rig::sample();
    r.text(BEH_FATHA_TEH);
    assert_eq!(at(&r), 3);
    assert_eq!(r.key(&PanelKey::Left), Do::Stay, "the end is on the left");
    assert_eq!(r.key(&PanelKey::Right), Do::Show);
    assert_eq!(at(&r), 2, "before the teh");
    r.key(&PanelKey::Right);
    assert_eq!(at(&r), 0, "over the beh and its fatha as one");
    r.key(&PanelKey::Left);
    assert_eq!(at(&r), 2);
}

#[test]
fn moving_the_caret_keeps_the_results_and_the_highlight() {
    let mut r = Rig::sample();
    r.text("a");
    r.text("l");
    let names = r.names();
    assert!(names.len() > 1, "{names:?}");
    r.key(&PanelKey::Down);
    let lit = r.page.highlight;
    assert_eq!(r.key(&PanelKey::Left), Do::Show);
    assert_eq!((r.names(), r.page.highlight), (names.clone(), lit));
    assert_eq!(r.click(place(2)), Do::Show);
    assert_eq!((r.names(), r.page.highlight), (names, lit));
}

#[test]
fn letters_erase_delete_and_a_chip_act_in_the_middle_of_the_search_and_a_change_goes_to_the_top() {
    let mut r = Rig::sample();
    r.text("ak");
    r.click(place(1));
    r.key(&PanelKey::Down);
    r.text("n");
    assert_eq!((searched(&r.page), at(&r)), ("ank", 2));
    assert_eq!(
        r.page.highlight,
        Some(0),
        "a change of text starts at the top"
    );
    r.key(&PanelKey::Erase);
    assert_eq!((searched(&r.page), at(&r)), ("ak", 1));
    assert_eq!(r.key(&PanelKey::Delete), Do::Show);
    assert_eq!((searched(&r.page), at(&r)), ("a", 1));
    assert_eq!(
        r.key(&PanelKey::Delete),
        Do::Stay,
        "nothing after the caret"
    );
    r.key(&PanelKey::Left);
    r.key(&word(0, "b"));
    assert_eq!((searched(&r.page), at(&r)), ("ba", 1));
    r.key(&word(1, "ban"));
    assert_eq!((searched(&r.page), at(&r)), ("bana", 3));
}

#[test]
fn delete_in_the_search_goes_to_the_top_of_the_list() {
    let mut r = Rig::sample();
    r.text("xa");
    r.click(place(0));
    r.key(&PanelKey::Delete);
    assert_eq!(searched(&r.page), "a");
    r.key(&PanelKey::Down);
    assert_eq!(r.page.highlight, Some(1));
    assert_eq!(r.key(&PanelKey::Delete), Do::Show);
    assert_eq!(searched(&r.page), "");
    assert_eq!(r.page.highlight, Some(0), "a change of text");
}

#[test]
fn a_click_on_the_search_places_its_caret_clamped_and_inside_no_grapheme() {
    let mut r = Rig::sample();
    r.text(BEH_FATHA_TEH);
    assert_eq!(r.click(place(0)), Do::Show);
    assert_eq!(at(&r), 0);
    assert_eq!(r.click(place(1)), Do::Stay, "inside the beh's grapheme");
    assert_eq!(at(&r), 0);
    assert_eq!(r.click(place(2)), Do::Show);
    assert_eq!(at(&r), 2);
    assert_eq!(r.click(place(u32::MAX)), Do::Show);
    assert_eq!(at(&r), 3);
}

#[test]
fn prediction_hears_only_the_search_before_the_caret() {
    let mut r = Rig::sample();
    r.text("bank");
    r.click(place(2));
    assert_eq!(r.panel.line(false), search("ba"));
    r.key(&PanelKey::Right);
    assert_eq!(r.panel.line(false), search("ban"));
    r.key(&PanelKey::Delete);
    assert_eq!(r.page.search.expose(), "ban");
    assert_eq!(
        r.panel.line(false),
        search("ban"),
        "nothing before it changed"
    );
}

#[test]
fn a_page_with_no_line_takes_neither_the_keys_nor_the_click() {
    let mut r = Rig::sample();
    r.text("bank");
    r.click(open(0));
    for key in [PanelKey::Left, PanelKey::Right, PanelKey::Delete] {
        assert_eq!(r.key(&key), Do::Stay, "an entry page: {key:?}");
    }
    assert_eq!(r.click(place(0)), Do::Stay, "an entry page");
    assert_eq!(r.page.search.expose(), "bank");
}

#[test]
fn a_labels_values_draws_no_search_so_its_arrows_switch_filters_with_text_held() {
    let mut r = Rig::sample();
    r.text("e");
    r.click(FillClick::Filter(FillFilter::Labels));
    r.click(open(0));
    assert_eq!(r.key(&PanelKey::Delete), Do::Stay);
    assert_eq!(r.click(place(0)), Do::Stay);
    assert_eq!(r.page.search.expose(), "e");
    assert_eq!(r.key(&PanelKey::Left), Do::Show);
    assert_eq!(r.page.filter, FillFilter::Notes);
}
