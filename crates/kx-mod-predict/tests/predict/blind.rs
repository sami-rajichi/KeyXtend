//! Text boxes that cannot confirm a pick: one whose read gives no text, or whose check cannot tell, offers no chip that
//! erases until a read gives text or focus moves, and a selection holds the lane empty until the next piece, click or
//! focus.

use crate::rig::{Rig, plant};
use kx_mod_predict::Msg;
use kx_module_api::Redacted;
use kx_module_api::ports::{FieldError, FieldKind, Lane};

/// A rig in English whose learner knows `words`, in a text box that tells `before`.
fn rigged(words: &'static [&'static str], before: Option<&'static str>) -> Rig {
    let mut rig = Rig::build(|f, _| {
        plant(f, "en", words);
        f.field.set_before(before);
    });
    rig.lang("en");
    rig.focus(FieldKind::Text);
    rig
}

/// A rig whose learner knows `London` and `long`, in a text box that tells `before`.
fn londoned(before: Option<&'static str>) -> Rig {
    rigged(&["London", "long"], before)
}

/// The word chips after `lon`, sorted.
fn after_lon(rig: &mut Rig) -> Vec<String> {
    rig.typed("lon");
    let mut words = rig.words();
    words.sort();
    words
}

#[test]
fn a_text_box_that_gives_no_text_offers_no_chip_that_erases_until_focus_moves() {
    let mut rig = londoned(None);
    assert_eq!(after_lon(&mut rig), ["long"], "London would erase the l");
    rig.pick(0);
    assert_eq!(rig.typed_words(), [(0, "g ".to_owned())], "only the rest");
    assert_eq!(rig.fakes.field.ends_calls(), 1, "its kind is still checked");
    rig.fakes.field.set_before(Some(""));
    rig.focus(FieldKind::Text);
    assert_eq!(after_lon(&mut rig), ["London", "long"], "a new box");
}

#[test]
fn a_text_box_that_cannot_tell_an_end_offers_no_chip_that_erases_until_focus_moves() {
    let mut rig = londoned(Some(""));
    assert_eq!(after_lon(&mut rig), ["London", "long"]);
    rig.fakes.field.set_ends(None);
    let at = rig.words().iter().position(|w| w == "long").unwrap();
    rig.pick(u8::try_from(at).unwrap());
    let asked = rig.fakes.field.last_ends().map(Redacted::into_inner);
    assert_eq!(asked.as_deref(), Some("lon"), "the box was asked");
    assert_eq!(rig.typed_words(), [(0, "g ".to_owned())], "only the rest");
    rig.typed("g ");
    assert_eq!(after_lon(&mut rig), ["long"], "London would erase the l");
    rig.focus(FieldKind::Text);
    assert_eq!(after_lon(&mut rig), ["London", "long"], "a new box");
}

#[test]
fn a_timed_out_read_blinds_the_box_and_a_later_read_with_text_heals_it() {
    let mut rig = londoned(Some(""));
    rig.fakes.field.fail_reads(FieldError::TimedOut);
    rig.send(Msg::Pressed(false));
    assert_eq!(after_lon(&mut rig), ["long"], "the read timed out");
    rig.fakes.field.heal_reads();
    rig.send(Msg::Pressed(false));
    assert_eq!(
        after_lon(&mut rig),
        ["London", "long"],
        "the next read told"
    );
}

#[test]
fn a_pick_while_the_box_holds_a_selection_erases_nothing_and_reads_again() {
    let mut rig = rigged(&["hello", "help"], Some("we said "));
    rig.typed("hel");
    rig.fakes.field.select("hel");
    let reads = rig.fakes.field.before_calls();
    rig.pick(0);
    assert!(rig.typed_words().is_empty(), "nothing over the selection");
    assert_eq!(rig.fakes.field.before_calls(), reads + 1, "read again");
}

/// A way the user acts after a refused pick, by name.
type Way = (&'static str, fn(&mut Rig));

#[test]
fn a_selection_after_part_of_a_word_holds_the_lane_empty_until_the_next_piece_click_or_focus() {
    let ways: [Way; 3] = [
        ("a piece", |rig| rig.typed("i")),
        ("a click", |rig| {
            rig.send(Msg::Pressed(false));
        }),
        ("a focus", |rig| rig.focus(FieldKind::Text)),
    ];
    for (way, act) in ways {
        let mut rig = rigged(&["said", "same"], Some("we sa"));
        assert!(!rig.words().is_empty(), "{way}: chips for sa");
        rig.fakes.field.select("me");
        rig.pick(0);
        assert!(rig.typed_words().is_empty(), "{way}: nothing typed");
        assert_eq!(rig.lane().lane, Lane::Empty, "{way}: no dead chips");
        act(&mut rig);
        assert!(!rig.words().is_empty(), "{way} brings the chips back");
    }
}
