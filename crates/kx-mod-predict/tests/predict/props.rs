//! Properties over random pieces, boxes, picks, menu taps, switches and waits: a password box always shows the lock,
//! a pick types at most one word, presses are told only when they flip, and a stop leaves the lane empty and presses
//! let go.

use crate::rig::{Heard, Rig, plant};
use kx_mod_predict::Msg;
use kx_module_api::ports::{ChipPicked, FieldKind, Lane, Piece, WordsAsk};
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::select;
use std::time::Duration;

/// Random runs per property.
const CASES: u32 = 512;
/// Most steps in one run.
const MAX_STEPS: usize = 40;
/// The longest wait, in seconds, past the default `save_s` so saves happen.
const MAX_WAIT_S: u64 = 40;

/// One thing that happens to the actor.
#[derive(Clone, Debug)]
enum Step {
    /// The keyboard typed this text.
    Type(&'static str),
    /// The keyboard erased one character.
    Erase,
    /// Focus moved to a new box of this kind.
    Focus(FieldKind),
    /// A tap on this chip of the last lane, or of the lane before it.
    Pick(u8, bool),
    /// A tap on the menu or the list.
    Ask(WordsAsk),
    /// The keyboard shows this language.
    Lang(&'static str),
    /// Time passes by this many seconds, then the actor ticks.
    Wait(u64),
}

fn step() -> impl Strategy<Value = Step> {
    let texts = select(vec![
        "he", "l", "lo ", " ", "@", "@@", "x.org ", "sami", "مر", "حبا ",
    ]);
    let kinds = select(vec![
        FieldKind::Text,
        FieldKind::Password,
        FieldKind::Unknown,
        FieldKind::None,
    ]);
    let asks = select(vec![
        WordsAsk::Forget,
        WordsAsk::Cancel,
        WordsAsk::Pause(true),
        WordsAsk::Pause(false),
        WordsAsk::Power(false),
        WordsAsk::Power(true),
        WordsAsk::Undo,
        WordsAsk::More { rows: 5 },
        WordsAsk::Page { page: 1, rows: 5 },
        WordsAsk::ListPick(0),
        WordsAsk::Again,
    ]);
    prop_oneof![
        3 => texts.prop_map(Step::Type),
        1 => Just(Step::Erase),
        1 => kinds.prop_map(Step::Focus),
        2 => (0..6_u8, any::<bool>()).prop_map(|(i, old)| Step::Pick(i, old)),
        1 => asks.prop_map(Step::Ask),
        1 => select(vec!["en", "ar"]).prop_map(Step::Lang),
        1 => (0..=MAX_WAIT_S).prop_map(Step::Wait),
    ]
}

/// The rounds of the last two lanes, the older first.
fn rounds(rig: &Rig) -> (u32, u32) {
    let last = rig.lane().round;
    (last.wrapping_sub(1), last)
}

/// Does `step` to the actor; true when it may type a word: a chip pick or a list pick.
fn apply(rig: &mut Rig, step: &Step) -> bool {
    match step {
        Step::Type(text) => rig.typed(text),
        Step::Erase => _ = rig.send(Msg::Typed(Piece::Erase)),
        Step::Focus(kind) => rig.focus(*kind),
        Step::Pick(index, old) => {
            let (before, last) = rounds(rig);
            let round = if *old { before } else { last };
            rig.send(Msg::Picked(ChipPicked {
                round,
                index: *index,
            }));
            return true;
        }
        Step::Ask(ask) => {
            rig.ask(*ask);
            return matches!(ask, WordsAsk::ListPick(_));
        }
        Step::Lang(iso) => _ = rig.lang(iso),
        Step::Wait(s) => {
            rig.advance(Duration::from_secs(*s));
            rig.tick();
        }
    }
    false
}

/// A begun rig in English, in a text box, whose learner knows a few words.
fn rig() -> Rig {
    let mut rig = Rig::build(|f, _| plant(f, "en", &["hello", "help", "world"]));
    rig.lang("en");
    rig.focus(FieldKind::Text);
    rig
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn the_actor_keeps_its_promises_whatever_happens(steps in vec(step(), 1..MAX_STEPS)) {
        let mut rig = rig();
        let (mut kind, mut seen) = (FieldKind::Text, rig.all().len());
        for step in &steps {
            if let Step::Focus(k) = step {
                kind = *k;
            }
            let picked = apply(&mut rig, step);
            let heard = rig.all();
            let words = heard[seen..].iter().filter(|h| matches!(h, Heard::Word(_))).count();
            seen = heard.len();
            prop_assert!(words <= usize::from(picked), "{step:?} typed {words} words");
            if kind == FieldKind::Password && rig.lane().on {
                prop_assert_eq!(rig.lane().lane, Lane::Locked, "{:?}", step);
                prop_assert_eq!(words, 0);
            }
        }
        let told = rig.presses();
        prop_assert!(told.windows(2).all(|w| w[0] != w[1]), "presses told twice: {told:?}");
        rig.stop();
        prop_assert_eq!(rig.lane().lane, Lane::Empty);
        prop_assert_eq!(rig.presses().last().copied(), Some(false), "a stop lets presses go");
    }
}
