//! Properties over random line words, letters, clicks and picks: a word for a line never reaches the app, a word for the
//! app never does while aimed at, panel keys never tell a piece, and the last line told is the one in effect.

use super::forms::search;
use super::picks::chip;
use super::told::{Ears, Said, quiet};
use super::{Heard, LETTER, Quick};
use kx_module_api::ports::{FaceAction, PanelLine};
use proptest::collection::vec;
use proptest::prelude::*;

/// Random runs per property.
const CASES: u32 = 256;
/// Most steps in one run.
const MAX_STEPS: usize = 40;
/// What a pick erases.
const ERASE: u16 = 1;
/// What a pick types.
const WORD: &str = "ab ";

/// One thing that happens to the keyboard.
#[derive(Clone, Debug)]
enum Step {
    /// The panel opens or closes.
    Open(bool),
    /// Its module says this line shows.
    Line(PanelLine),
    /// A letter is clicked.
    Letter,
    /// A pick for a line, or for the app.
    Pick(bool),
    /// A click outside our windows.
    Away,
    /// A click on the panel's line.
    Aim,
    /// The relabel timer runs.
    Relabel,
}

/// A random step.
fn step() -> impl Strategy<Value = Step> {
    let lines = prop_oneof![
        Just(search("")),
        Just(search("a")),
        Just(search("ab ")),
        Just(PanelLine::Private),
        Just(PanelLine::None),
    ];
    prop_oneof![
        1 => any::<bool>().prop_map(Step::Open),
        3 => lines.prop_map(Step::Line),
        3 => Just(Step::Letter),
        3 => any::<bool>().prop_map(Step::Pick),
        1 => Just(Step::Away),
        1 => Just(Step::Aim),
        1 => Just(Step::Relabel),
    ]
}

/// Does `step`.
fn apply(q: &Quick, step: &Step) {
    match step {
        Step::Open(open) => q.open(*open),
        Step::Line(line) => q.line(line.clone()),
        Step::Letter => {
            q.b.rig.tap(LETTER).unwrap();
        }
        Step::Pick(line) => q.pick(ERASE, WORD, *line),
        Step::Away => q.away(),
        Step::Aim => {
            q.aim();
        }
        Step::Relabel => {
            q.b.rig.act(FaceAction::Relabel).unwrap();
        }
    }
}

/// True when `said` holds a piece.
fn told_a_piece(said: &[Said]) -> bool {
    said.iter().any(|s| matches!(s, Said::Piece(_)))
}

/// The last line told in `said`, or `last` when none was.
fn last_line(said: &[Said], last: PanelLine) -> PanelLine {
    let told = said.iter().rev().find_map(|s| match s {
        Said::Line(line) => Some(line.clone()),
        Said::Piece(_) => None,
    });
    told.unwrap_or(last)
}

/// The panel's state as its module said it.
#[derive(Default)]
struct Model {
    open: bool,
    line: PanelLine,
}

impl Model {
    /// Follows what `step` says: a new panel shows no line yet, and a word for a shut panel changes nothing.
    fn follow(&mut self, step: &Step) {
        match step {
            Step::Open(open) => {
                if *open != self.open {
                    self.line = PanelLine::None;
                }
                self.open = *open;
            }
            Step::Line(line) if self.open => self.line = line.clone(),
            _ => {}
        }
    }
}

/// The checks of one step: what reached the app and prediction after `step`, with `aimed` before it and `lined` when
/// the aim held a line then.
fn check(
    q: &Quick,
    ears: &Ears,
    step: &Step,
    (aimed, lined): (bool, bool),
    last: &mut PanelLine,
) -> Result<(), TestCaseError> {
    let strokes = q.strokes();
    let said = ears.take();
    let heard = q.heard();
    match step {
        Step::Pick(true) => {
            prop_assert!(strokes.is_empty(), "a word for a line reached the app");
            prop_assert!(!told_a_piece(&said), "a word for a line told a piece");
            let one = [Heard::Key(chip(ERASE, WORD))];
            prop_assert!(
                heard.is_empty() || heard == one,
                "a word for a line is one key or none"
            );
        }
        Step::Pick(false) if lined => {
            prop_assert!(
                strokes.is_empty(),
                "a word for the app went while a line was aimed"
            );
            prop_assert!(!told_a_piece(&said));
        }
        Step::Letter if aimed => {
            prop_assert!(strokes.is_empty(), "a panel key reached the app");
            prop_assert!(!told_a_piece(&said), "a panel key told a piece");
        }
        _ => {}
    }
    *last = last_line(&said, last.clone());
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn a_pick_goes_only_where_it_was_offered_whatever_happens(steps in vec(step(), 1..MAX_STEPS)) {
        let (q, ears) = quiet();
        let (mut last, mut model) = (PanelLine::None, Model::default());
        for step in &steps {
            // The state read settles the aim, and may tell that it left.
            let aimed = q.aimed();
            let lined = aimed && !matches!(model.line, PanelLine::None);
            last = last_line(&ears.take(), last);
            apply(&q, step);
            model.follow(step);
            check(&q, &ears, step, (aimed, lined), &mut last)?;
        }
        let aimed = q.aimed();
        last = last_line(&ears.take(), last);
        let want = if aimed { model.line } else { PanelLine::None };
        prop_assert_eq!(last, want, "the last line told is the one in effect");
    }
}
