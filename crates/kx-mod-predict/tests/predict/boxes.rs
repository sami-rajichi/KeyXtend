//! Box kinds: a password box locks the lane and is neither read nor learned from; an unknown box or none gets chips
//! but is neither read nor learned from; a text box is read and learned from. The watch's kind wins over a late event.

use crate::rig::{Rig, plant, saved};
use kx_mod_predict::Msg;
use kx_module_api::Redacted;
use kx_module_api::ports::{ChipKind, ChipPicked, FieldKind, Lane, Piece};

/// A rig in English whose learner knows `hello` and `help`, focus on a new box of `kind`.
fn in_box(kind: FieldKind) -> Rig {
    let mut rig = Rig::build(|f, _| plant(f, "en", &["hello", "help"]));
    rig.lang("en");
    rig.focus(kind);
    rig
}

/// How many times `word` is learned in the English learner saved now.
fn uses(rig: &Rig, word: &str) -> u32 {
    use kx_mod_predict::score::Learned;
    saved(&rig.fakes, "en").map_or(0, |l| l.view(None).count(word))
}

#[test]
fn a_password_box_locks_the_lane_learns_nothing_and_is_never_read() {
    let mut rig = in_box(FieldKind::Password);
    rig.typed("hel");
    assert_eq!(rig.lane().lane, Lane::Locked);
    rig.typed("lo ");
    rig.typed("sami@x.org ");
    rig.stop();
    assert_eq!(uses(&rig, "hello"), 2, "nothing learned");
    assert_eq!(rig.fakes.store.saves(), 0, "nothing saved");
    let field = &rig.fakes.field;
    assert_eq!((field.before_calls(), field.ends_calls()), (0, 0));
    assert!(
        rig.presses().iter().all(|on| !on),
        "a password is never followed"
    );
}

#[test]
fn an_unknown_box_or_none_suggests_but_learns_nothing_and_is_never_read() {
    for kind in [FieldKind::Unknown, FieldKind::None] {
        let mut rig = in_box(kind);
        rig.typed("hel");
        assert_eq!(rig.words(), ["hello", "help"], "{kind:?}");
        rig.typed("lo ");
        rig.typed("sami@x.org ");
        rig.stop();
        assert_eq!(uses(&rig, "hello"), 2, "{kind:?} learns nothing");
        assert_eq!(rig.fakes.store.saves(), 0, "{kind:?}");
        let field = &rig.fakes.field;
        assert_eq!(
            (field.before_calls(), field.ends_calls()),
            (0, 0),
            "{kind:?}"
        );
    }
}

#[test]
fn a_text_box_is_read_for_its_last_words_and_learned_from() {
    let mut rig = Rig::build(|f, _| {
        plant(f, "en", &["hello", "help"]);
        f.field.set_before(Some("we said hel"));
    });
    rig.lang("en");
    let reads = rig.fakes.field.before_calls();
    rig.focus(FieldKind::Text);
    assert_eq!(rig.fakes.field.before_calls(), reads + 1, "one read");
    assert_eq!(rig.words(), ["hello", "help"], "the read word gets chips");
    rig.typed("lo ");
    rig.typed("sami@x.org ");
    rig.stop();
    assert_eq!(uses(&rig, "hello"), 3, "the finished word is learned");
    assert_eq!(rig.fakes.store.saves(), 2, "the words and the emails");
}

/// The text `text` as the keyboard types it.
fn text(text: &str) -> Msg {
    Msg::Typed(Piece::Text(Redacted::new(text.to_owned())))
}

#[test]
fn a_read_that_finds_a_password_box_wins_over_its_late_focus_event() {
    let mut rig = in_box(FieldKind::Text);
    rig.typed("hel");
    rig.fakes.field.focus(FieldKind::Password);
    rig.late(Msg::Pressed(false));
    assert_eq!(rig.lane().lane, Lane::Locked, "locked at once");
    rig.late(text("secret "));
    assert_eq!(rig.lane().lane, Lane::Locked);
    rig.told();
    rig.stop();
    assert_eq!(uses(&rig, "secret"), 0, "nothing learned");
    assert_eq!(rig.fakes.store.saves(), 0);
}

#[test]
fn a_read_that_finds_an_unknown_box_stops_learning_before_its_late_focus_event() {
    let mut rig = in_box(FieldKind::Text);
    rig.typed("hel");
    rig.fakes.field.focus(FieldKind::Unknown);
    rig.late(Msg::Pressed(false));
    rig.late(text("hello "));
    rig.told();
    rig.stop();
    assert_eq!(uses(&rig, "hello"), 2, "nothing learned");
    assert_eq!(rig.fakes.store.saves(), 0);
}

#[test]
fn a_pick_after_an_unheard_move_to_a_password_box_types_nothing() {
    for (kind, typed) in [
        (FieldKind::Text, "hel"),
        (FieldKind::Unknown, "hel"),
        (FieldKind::Text, "hello "),
    ] {
        let mut rig = in_box(kind);
        rig.typed(typed);
        let round = rig.lane().round;
        rig.fakes.field.focus(FieldKind::Password);
        rig.late(Msg::Picked(ChipPicked { round, index: 0 }));
        assert!(rig.typed_words().is_empty(), "{kind:?} {typed:?}");
        assert_eq!(rig.lane().lane, Lane::Locked, "{kind:?} {typed:?}");
    }
}

#[test]
fn a_word_ended_after_an_unheard_move_to_a_password_box_is_not_learned() {
    let mut rig = in_box(FieldKind::Text);
    rig.fakes.field.focus(FieldKind::Password);
    rig.late(text("secret "));
    assert_eq!(rig.lane().lane, Lane::Locked);
    rig.told();
    rig.stop();
    assert_eq!(uses(&rig, "secret"), 0);
    assert_eq!(rig.fakes.store.saves(), 0);
}

#[test]
fn a_shortcut_latch_empties_the_lane_until_it_lets_go() {
    use kx_module_api::ports::TypingMode;
    let mut rig = in_box(FieldKind::Text);
    rig.typed("hel");
    let ctrl = TypingMode {
        shortcut: true,
        ..TypingMode::default()
    };
    rig.send(Msg::Mode(ctrl));
    assert_eq!(rig.lane().lane, Lane::Empty);
    rig.send(Msg::Mode(TypingMode::default()));
    assert!(rig.chips().iter().all(|c| c.1 == ChipKind::Word));
    assert_eq!(rig.words(), ["hello", "help"]);
}

#[test]
fn a_waiting_dead_key_empties_the_lane_until_it_ends() {
    use kx_module_api::ports::TypingMode;
    let mut rig = in_box(FieldKind::Text);
    rig.typed("hel");
    let dead = TypingMode {
        dead: true,
        ..TypingMode::default()
    };
    rig.send(Msg::Mode(dead));
    assert_eq!(rig.lane().lane, Lane::Empty);
    rig.send(Msg::Mode(TypingMode::default()));
    assert_eq!(rig.words(), ["hello", "help"]);
}
