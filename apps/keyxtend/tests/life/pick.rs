//! The whole word loop over the app: keys typed in a text box bring a chip from the shipped pack, its pick erases what
//! was typed and types the word through the keyboard, and the word is learned and sealed at Quit.

use crate::rig::{Rig, WAIT, char_key, hear, words_out};
use keyxtend::boot::Boot;
use keyxtend::drain;
use kx_mod_keyboard::BoardSpec;
use kx_mod_predict::learn::Learner;
use kx_mod_predict::score::Learned;
use kx_mod_predict::script::{self, Script};
use kx_mod_predict::{PredictSettings, learned_blob};
use kx_module_api::ports::{
    FaceAction, FieldKind, KeyId, Lane, Piece, Suggested, Typed, pack_file,
};
use kx_platform_fake::Stroke;
use std::sync::Arc;
use std::sync::mpsc::Receiver;

/// The language of the layout in front, English, whose pack the build writes.
const ISO: &str = "en";
/// The scan codes of H, E and L, typed as the start of the word.
const TYPED: [u16; 3] = [0x23, 0x12, 0x26];
/// The word the chip completes them to.
const WORD: &str = "help";
/// Its chip's place on the shipped pack's lane for them: first.
const PLACE: usize = 0;

/// The round and place of `word` on the first lane for `typed`: every chip completes it, so earlier letters' lanes
/// are passed over.
fn offered(lanes: &Receiver<Suggested>, typed: &str, word: &str) -> Option<(u32, usize)> {
    while let Ok(lane) = lanes.recv_timeout(WAIT) {
        let Lane::Chips(chips) = &lane.lane else {
            continue;
        };
        let texts: Vec<&str> = chips.iter().map(|c| c.text.expose().as_str()).collect();
        let at = texts.iter().position(|t| *t == word);
        if let Some(at) = at.filter(|_| texts.iter().all(|t| t.starts_with(typed))) {
            return Some((lane.round, at));
        }
    }
    None
}

/// Waits for the typed piece that carries text, and gives it.
fn typed_text(pieces: &Receiver<Typed>) -> Option<String> {
    while let Ok(typed) = pieces.recv_timeout(WAIT) {
        if let Piece::Text(text) = typed.piece {
            return Some(text.into_inner());
        }
    }
    None
}

/// A rig whose packs serve the shipped `ISO` pack, its box saying it ends with what a pick erases.
fn shipped() -> Rig {
    let rig = Rig::new();
    let pack = std::fs::read(words_out().join(pack_file(ISO).unwrap())).unwrap();
    rig.packs.add(ISO, pack);
    rig.packs.ship(ISO);
    rig.field.set_ends(Some(true));
    rig
}

/// Quits `boot` as the tray does and waits for every module to stop, so each blob is saved.
fn quit(boot: Boot) {
    let Boot {
        kernel,
        hub,
        requests,
        heard,
    } = boot;
    let thread = drain::spawn(kernel, Arc::clone(&hub), requests).unwrap();
    assert!(hub.quit());
    thread.join().unwrap();
    drop(heard);
}

/// Taps the keys of `TYPED` on the keyboard.
fn type_start(boot: &Boot) {
    for sc in TYPED {
        let id = KeyId::new(char_key(sc));
        let _ = boot.hub.act(FaceAction::Tap { id, back: false });
    }
}

/// Checks a pick's strokes: one tap of the board's erase key per typed letter, then `text`, and nothing else.
fn check_strokes(picked: &[Stroke], text: &str) {
    let erase = Stroke::Tap(BoardSpec::shipped().unwrap().erase_key().unwrap());
    let erased = picked.iter().filter(|s| **s == erase).count();
    assert_eq!(erased, TYPED.len(), "one Backspace per typed letter");
    let strokes = TYPED.len() + 1;
    assert_eq!(picked.len(), strokes, "then the word, and nothing else");
    assert!(matches!(picked.last(), Some(Stroke::Text(t, _)) if t.expose() == text));
}

/// The learner `rig`'s store sealed for `ISO`; `None` when none was saved or it does not read.
fn sealed(rig: &Rig) -> Option<Learner> {
    let cfg = PredictSettings::from_defaults().unwrap().learn_config();
    let blob = rig.sealed.blob(&learned_blob(ISO)?)?;
    Learner::from_bytes(&blob, cfg).ok()
}

#[test]
fn a_typed_start_brings_a_chip_whose_pick_types_the_word_which_is_learned_and_sealed() {
    let rig = shipped();
    let mut boot = rig.load();
    let bus = boot.kernel.bus();
    let (_lanes, lanes) = hear::<Suggested>(&bus);
    boot.start().unwrap();
    rig.field.focus(FieldKind::Text);
    type_start(&boot);
    let typed = WORD.get(..TYPED.len()).unwrap();
    let (round, at) = offered(&lanes, typed, WORD).expect("a chip completes the typed start");
    assert_eq!(at, PLACE, "the word's place on the lane");
    let before = rig.injector.strokes().len();
    let (_pieces, pieces) = hear::<Typed>(&bus);
    boot.hub
        .pick_chip(i64::from(round), i32::try_from(at).unwrap());
    let text = typed_text(&pieces).expect("the pick typed");
    let gap = script::of(WORD).map_or("", Script::gap);
    assert_eq!(text, format!("{WORD}{gap}"), "the word typed whole");
    check_strokes(&rig.injector.strokes().split_off(before), &text);
    quit(boot);
    let learner = sealed(&rig).expect("saved at Quit");
    assert_eq!(learner.iso(), ISO);
    let uses = learner.view(None).count(WORD);
    assert_eq!(uses, 1, "the picked word was learned once");
}
