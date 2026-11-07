//! Properties over random lines, app pieces, clicks, picks and focus moves: what a line holds is never learned or stored,
//! each pick names what the lane followed when it was sent, and a private line never offers a chip.

use super::search;
use crate::rig::{Heard, Rig, plant, saved};
use kx_mod_predict::Msg;
use kx_mod_predict::score::Learned;
use kx_module_api::ports::{ChipPicked, FieldKind, Lane, PanelLine};
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::select;

/// Random runs per property.
const CASES: u32 = 256;
/// Most steps in one run.
const MAX_STEPS: usize = 40;
/// A word only the lines hold.
const LINE_WORD: &str = "zebra";
/// An email only the lines hold.
const LINE_EMAIL: &str = "lea@zebra.org";

/// One thing that happens to the actor.
#[derive(Clone, Debug)]
enum Step {
    /// The keyboard tells this search.
    Search(&'static str),
    /// The keyboard tells a private line.
    Private,
    /// The aim leaves.
    App,
    /// The app gets this text, which a panel's key never sends.
    Type(&'static str),
    /// A tap on this chip of the last lane.
    Pick(u8),
    /// A click outside our windows.
    Click,
    /// Focus moves to a box of this kind.
    Focus(FieldKind),
}

/// A random step.
fn step() -> impl Strategy<Value = Step> {
    let searches = select(vec![
        "",
        " ",
        "he",
        "zebra",
        "zebra ",
        "zebra he",
        "@@",
        "lea@zebra.org",
        "lea@zebra.org ",
        "lea@",
    ]);
    let kinds = select(vec![
        FieldKind::Text,
        FieldKind::Password,
        FieldKind::Unknown,
    ]);
    prop_oneof![
        4 => searches.prop_map(Step::Search),
        1 => Just(Step::Private),
        2 => Just(Step::App),
        2 => select(vec!["he", "l", "lo ", " "]).prop_map(Step::Type),
        3 => (0..5_u8).prop_map(Step::Pick),
        1 => Just(Step::Click),
        1 => kinds.prop_map(Step::Focus),
    ]
}

/// Does `step` to the actor.
fn apply(rig: &mut Rig, step: &Step) {
    match step {
        Step::Search(text) => rig.line(search(text)),
        Step::Private => rig.line(PanelLine::Private),
        Step::App => rig.line(PanelLine::None),
        Step::Type(text) => rig.typed(text),
        Step::Pick(index) => {
            let round = rig.lane().round;
            rig.send(Msg::Picked(ChipPicked {
                round,
                index: *index,
            }));
        }
        Step::Click => _ = rig.send(Msg::Pressed(false)),
        Step::Focus(kind) => rig.focus(*kind),
    }
}

/// The targets of the words `rig` asked for after `seen` events.
fn targets_since(rig: &Rig, seen: usize) -> Vec<bool> {
    let heard = rig.all();
    let words = heard[seen..].iter().filter_map(|h| match h {
        Heard::Word(w) => Some(w.line),
        _ => None,
    });
    words.collect()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn a_line_is_never_learned_whatever_happens(steps in vec(step(), 1..MAX_STEPS)) {
        let mut rig = Rig::build(|f, _| plant(f, "en", &["hello", "help"]));
        rig.lang("en");
        rig.focus(FieldKind::Text);
        let (mut on_search, mut on_private) = (false, false);
        for step in &steps {
            let seen = rig.all().len();
            apply(&mut rig, step);
            if matches!(step, Step::Search(_) | Step::Private | Step::App) {
                on_search = matches!(step, Step::Search(_));
                on_private = matches!(step, Step::Private);
            }
            for target in targets_since(&rig, seen) {
                prop_assert_eq!(target, on_search, "{:?}", step);
            }
            if on_private {
                prop_assert_eq!(rig.lane().lane, Lane::Empty, "{:?}", step);
            }
        }
        rig.stop();
        if let Some(learner) = saved(&rig.fakes, "en") {
            prop_assert_eq!(learner.view(None).count(LINE_WORD), 0);
        }
        let emails = rig.fakes.store.blob(&kx_mod_predict::emails_blob().unwrap());
        if let Some(bytes) = emails {
            let list = kx_mod_predict::email::Emails::from_bytes(&bytes, usize::MAX).unwrap();
            prop_assert!(!list.list().iter().any(|e| e == LINE_EMAIL));
        }
    }
}
