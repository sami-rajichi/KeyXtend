//! The lane and picks: typing shows chips, a pick sends one `TypeWord` with the right erase, a stale round sends
//! nothing, and a pick whose word no longer ends the box sends nothing and reads again.

use crate::rig::{Rig, plant};
use kx_mod_predict::Msg;
use kx_module_api::Redacted;
use kx_module_api::ports::{ChipPicked, FieldKind, Lane, Piece};

/// A rig in English whose learner knows `hello` and `help`, in a text box holding `before`.
fn texted(before: Option<&str>) -> Rig {
    let mut rig = Rig::build(|f, _| {
        plant(f, "en", &["hello", "help"]);
        f.field.set_before(before);
    });
    rig.lang("en");
    rig.focus(FieldKind::Text);
    rig
}

#[test]
fn typing_hel_shows_chips_and_a_pick_sends_one_word_erasing_what_was_typed() {
    let mut rig = texted(None);
    rig.typed("h");
    rig.typed("el");
    assert_eq!(rig.words(), ["hello", "help"]);
    let round = rig.lane().round;
    rig.pick(1);
    assert_eq!(rig.typed_words(), [(3, "help ".to_owned())]);
    let after = rig.lane();
    assert!(after.round != round, "a new round");
    assert_eq!(
        after.lane,
        Lane::Empty,
        "until the keyboard tells what it typed"
    );
    rig.pick(0);
    assert_eq!(rig.typed_words().len(), 1, "a second tap types nothing");
}

#[test]
fn each_change_brings_a_new_round_and_a_stale_round_sends_nothing() {
    let mut rig = texted(None);
    rig.typed("he");
    let old = rig.lane().round;
    rig.typed("l");
    assert_ne!(rig.lane().round, old);
    rig.send(Msg::Picked(ChipPicked {
        round: old,
        index: 0,
    }));
    assert!(rig.typed_words().is_empty(), "a stale lane");
    rig.send(Msg::Picked(ChipPicked {
        round: rig.lane().round,
        index: 9,
    }));
    assert!(rig.typed_words().is_empty(), "no chip there");
}

#[test]
fn a_read_word_is_checked_and_a_box_that_moved_on_or_cannot_tell_gets_nothing_and_a_new_read() {
    for ends in [Some(false), None] {
        let mut rig = texted(Some("we said hel"));
        assert_eq!(rig.words(), ["hello", "help"]);
        rig.fakes.field.set_ends(ends);
        let reads = rig.fakes.field.before_calls();
        rig.pick(0);
        assert!(rig.typed_words().is_empty(), "{ends:?}: nothing erased");
        let asked = rig.fakes.field.last_ends().map(Redacted::into_inner);
        assert_eq!(
            asked.as_deref(),
            Some("hel"),
            "{ends:?}: it checked the read word"
        );
        assert_eq!(
            rig.fakes.field.before_calls(),
            reads + 1,
            "{ends:?}: read again"
        );
    }
}

#[test]
fn a_read_word_the_box_still_ends_with_is_replaced() {
    let mut rig = texted(Some("we said hel"));
    rig.fakes.field.set_ends(Some(true));
    rig.pick(0);
    assert_eq!(rig.typed_words(), [(3, "hello ".to_owned())]);
}

#[test]
fn a_word_typed_here_needs_no_check() {
    let mut rig = texted(None);
    rig.typed("hel");
    rig.pick(0);
    assert_eq!(rig.fakes.field.ends_calls(), 0);
    assert_eq!(rig.typed_words(), [(3, "hello ".to_owned())]);
}

#[test]
fn after_a_pick_the_lane_is_empty_until_the_keyboard_types_it() {
    let mut rig = texted(None);
    rig.typed("hel");
    rig.pick(0);
    assert_eq!(rig.lane().lane, Lane::Empty, "no chips until the echo");
    rig.pick(0);
    assert_eq!(rig.typed_words().len(), 1, "a second tap types nothing");
}

#[test]
fn the_echo_of_a_pick_brings_the_chips_back() {
    let mut rig = texted(None);
    rig.typed("hel");
    rig.pick(0);
    for _ in 0..3 {
        rig.send(Msg::Typed(Piece::Erase));
    }
    rig.typed("hel");
    assert_eq!(rig.words(), ["hello", "help"]);
}

#[test]
fn presses_are_wanted_while_a_word_or_context_is_held_and_not_after_a_reset_or_stop() {
    let mut rig = texted(None);
    rig.typed("hel");
    assert_eq!(rig.presses().last(), Some(&true));
    rig.send(Msg::Pressed(true));
    assert_eq!(rig.presses().last(), Some(&true), "a press on our window");
    rig.send(Msg::Pressed(false));
    assert_eq!(
        rig.presses().last(),
        Some(&false),
        "a click elsewhere resets"
    );
    rig.typed("hel");
    rig.send(Msg::Typed(Piece::Break));
    assert_eq!(rig.presses().last(), Some(&false), "a break resets");
    rig.typed("hel");
    rig.stop();
    assert_eq!(rig.presses().last(), Some(&false), "at stop");
}
