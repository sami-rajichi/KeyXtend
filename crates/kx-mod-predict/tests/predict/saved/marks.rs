//! The mark a saved pick leaves on its token: nothing in that token is stored, whatever is glued before the pick or
//! typed after it. The wait for its text holds through a focus change, a new set or a racing key, until that text, a
//! new pick or turn-off.

use super::{SAVED, knowing, landed, notebook, stored};
use kx_mod_predict::Msg;
use kx_module_api::Redacted;
use kx_module_api::ports::{ChipKind, FieldKind, Piece, WordsAsk};

/// The saved email the second chip after `@@` types.
const LEA: &str = SAVED[1];
/// Its start, which an edit after the pick keeps.
const START: &str = "lea@carnet";
/// An email typed by hand.
const HAND: &str = "nour@y.com";

#[test]
fn a_saved_pick_after_an_at_at_glued_to_a_word_is_never_stored() {
    for word in ["alex", "أليكس"] {
        let mut rig = knowing(&[], &SAVED);
        rig.typed(word);
        rig.typed("@@");
        rig.pick(1);
        landed(&mut rig);
        rig.typed(" ");
        rig.typed(&format!("{HAND} "));
        let [saved, typed] = stored(&mut rig, [LEA, HAND]);
        assert!(!saved, "{word}: the saved email was stored inside another");
        assert!(typed, "{word}: the mark went with its token");
    }
}

#[test]
fn text_added_or_changed_after_a_saved_pick_is_never_stored() {
    let edits: [(usize, &str); 2] = [(0, ".tn"), (2, "com")];
    for (erase, add) in edits {
        let mut rig = knowing(&[], &SAVED);
        rig.typed("@@");
        rig.pick(1);
        landed(&mut rig);
        for _ in 0..erase {
            rig.send(Msg::Typed(Piece::Erase));
        }
        rig.typed(add);
        rig.send(Msg::Pressed(false));
        assert_eq!(stored(&mut rig, [START]), [false], "{erase} {add}");
    }
}

#[test]
fn a_saved_pick_told_after_a_focus_change_is_never_stored() {
    let mut rig = knowing(&[], &SAVED);
    rig.typed("@@");
    rig.pick(1);
    rig.focus(FieldKind::Text);
    landed(&mut rig);
    rig.send(Msg::Pressed(false));
    assert_eq!(stored(&mut rig, [LEA]), [false]);
}

#[test]
fn a_new_set_between_a_saved_pick_and_its_echo_keeps_the_mark() {
    let mut rig = knowing(&[], &SAVED);
    rig.typed("@@");
    rig.pick(1);
    rig.send(notebook(&[]));
    landed(&mut rig);
    rig.typed(" ");
    assert_eq!(stored(&mut rig, [LEA]), [false]);
}

#[test]
fn a_key_racing_a_saved_picks_echo_keeps_the_mark() {
    let space = Piece::Text(Redacted::new(" ".to_owned()));
    for race in [space, Piece::Advance, Piece::Break, Piece::Erase] {
        let mut rig = knowing(&[], &SAVED);
        rig.typed("@@");
        rig.pick(1);
        rig.send(Msg::Typed(race.clone()));
        landed(&mut rig);
        rig.typed(" ");
        assert_eq!(stored(&mut rig, [LEA]), [false], "{race:?}");
    }
}

#[test]
fn the_echo_ends_the_wait_so_the_same_text_later_is_learned() {
    let mut rig = knowing(&[], &SAVED);
    rig.typed("@@");
    rig.pick(1);
    landed(&mut rig);
    rig.typed(" ");
    rig.typed(LEA);
    rig.typed(" ");
    assert_eq!(stored(&mut rig, [LEA]), [true]);
}

#[test]
fn a_new_pick_ends_the_wait_of_a_saved_pick_never_told() {
    let mut rig = knowing(&[HAND], &SAVED);
    rig.typed("@@");
    let lea = rig.chips().iter().position(|(e, _)| e == LEA).unwrap();
    rig.pick(u8::try_from(lea).unwrap());
    assert_eq!(rig.typed_words().pop().unwrap().1, LEA);
    rig.typed(" ");
    rig.typed("nou");
    assert_eq!(rig.chips()[0], (HAND.to_owned(), ChipKind::Email));
    rig.pick(0);
    landed(&mut rig);
    rig.typed(" ");
    rig.typed(LEA);
    rig.typed(" ");
    assert_eq!(stored(&mut rig, [LEA]), [true]);
}

#[test]
fn turning_prediction_off_ends_the_wait_of_a_saved_pick_never_told() {
    let mut rig = knowing(&[], &SAVED);
    rig.typed("@@");
    rig.pick(1);
    rig.ask(WordsAsk::Power(false));
    rig.ask(WordsAsk::Power(true));
    rig.focus(FieldKind::Text);
    rig.typed(LEA);
    rig.typed(" ");
    assert_eq!(stored(&mut rig, [LEA]), [true]);
}
