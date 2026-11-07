//! Picks on a search: each says it was offered for a line, its stored text decides whether it may erase, the lane keeps
//! its chips until the new text comes, and an old round sends nothing.

use super::{aimed, in_box, search};
use kx_mod_predict::Msg;
use kx_module_api::ports::{ChipPicked, Lane, PanelLine};

#[test]
fn a_pick_names_the_line_and_sends_only_the_rest_of_its_chip() {
    let mut rig = aimed("hel");
    rig.pick(1);
    assert_eq!(rig.typed_words(), [(0, "p ".to_owned())]);
    assert_eq!(rig.word_targets(), [true]);
}

#[test]
fn a_pick_that_changes_the_word_erases_what_differs_as_the_text_confirms_it() {
    let mut rig = in_box("fr", &["école"], "");
    rig.line(search("ecol"));
    assert_eq!(rig.words(), ["école"]);
    rig.pick(0);
    assert_eq!(rig.typed_words(), [(4, "école ".to_owned())]);
    assert_eq!(rig.word_targets(), [true]);
}

#[test]
fn a_pick_refused_or_dropped_later_leaves_the_chips_on_the_lane() {
    let mut rig = aimed("hel");
    let round = rig.lane().round;
    rig.pick(0);
    assert_eq!(rig.typed_words().len(), 1);
    assert_ne!(rig.lane().round, round, "an older tap is stale");
    assert_eq!(rig.words(), ["hello", "help"], "no new text, same chips");
}

#[test]
fn the_new_text_moves_the_lane_on_after_a_pick() {
    let mut rig = aimed("hel");
    rig.pick(1);
    rig.line(search("help "));
    assert_eq!(rig.words(), ["hello", "help"], "next words now");
    rig.line(search("hel"));
    assert_eq!(rig.words(), ["hello", "help"], "and back to its open word");
}

#[test]
fn a_stale_round_sends_nothing() {
    let mut rig = aimed("hel");
    let old = rig.lane().round;
    rig.line(search("hell"));
    rig.send(Msg::Picked(ChipPicked {
        round: old,
        index: 0,
    }));
    assert!(rig.typed_words().is_empty());
}

#[test]
fn a_pick_for_the_lane_of_the_app_sends_nothing_once_the_line_shows() {
    let mut rig = in_box("en", &["hello", "help"], "hel");
    let old = rig.lane().round;
    rig.line(search("x"));
    rig.send(Msg::Picked(ChipPicked {
        round: old,
        index: 0,
    }));
    assert!(rig.typed_words().is_empty(), "the old lane types nothing");
}

#[test]
fn a_pick_after_the_aim_left_names_the_app() {
    let mut rig = aimed("hel");
    rig.fakes.field.set_before(Some("hel"));
    rig.line(PanelLine::None);
    rig.pick(0);
    assert_eq!(rig.word_targets(), [false]);
}

#[test]
fn a_chip_past_the_keyboards_cap_is_not_offered_on_a_search_either() {
    let mut rig = aimed("hel");
    let capped = kx_module_api::ports::TypingMode {
        cap: Some(1),
        ..Default::default()
    };
    rig.send(Msg::Mode(capped));
    rig.line(search("he"));
    assert!(rig.words().is_empty());
}

#[test]
fn a_latch_that_holds_the_lane_holds_it_on_a_search_too() {
    let mut rig = aimed("hel");
    let ctrl = kx_module_api::ports::TypingMode {
        shortcut: true,
        ..Default::default()
    };
    rig.send(Msg::Mode(ctrl));
    rig.line(search("he"));
    assert_eq!(rig.lane().lane, Lane::Empty);
}
