//! Box kinds: a password box locks the lane and is neither read nor learned from; an unknown box or none gets chips
//! but is neither read nor learned from; a text box is read and learned from.

use crate::rig::{Rig, plant, saved};
use kx_module_api::ports::{ChipKind, FieldKind, Lane};

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

#[test]
fn a_shortcut_latch_empties_the_lane_until_it_lets_go() {
    use kx_mod_predict::Msg;
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
    use kx_mod_predict::Msg;
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
