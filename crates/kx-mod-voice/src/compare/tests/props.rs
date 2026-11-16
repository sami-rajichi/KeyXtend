//! Any run of Starts, Stops, Cancels, resets and notes, an older run's included: the lend follows the stage, each run
//! opens once and closes once with no call after, a resting Compare holds no open run, Cancel always rests, the
//! numbers are kept at most once a run and texts show only with no set text.

use super::*;
use proptest::collection::vec;
use proptest::prelude::*;
use std::collections::BTreeSet;

/// The most models one run compares.
const MAX_MODELS: u8 = 3;
/// The most steps in one case.
const MAX_STEPS: usize = 50;
/// The highest piece index a note names.
const MAX_INDEX: u16 = 3;
/// How often a note goes to the older run.
const OLD: f64 = 0.15;

/// The stages a run may tell.
const STAGES: [SpeechStage; 5] = [
    SpeechStage::Loading,
    SpeechStage::Open,
    SpeechStage::Shut,
    SpeechStage::Limit,
    SpeechStage::Silence,
];

/// The faults a run may tell: a card's, a model's, the recording's and the job's.
const FAULTS: [SpeechError; 9] = [
    SpeechError::NoCard,
    SpeechError::CardFailed,
    SpeechError::BadFile,
    SpeechError::Language,
    SpeechError::NoMic,
    SpeechError::Unplugged,
    SpeechError::NoSound,
    SpeechError::Gone,
    SpeechError::Timeout,
];

/// What a run tells.
#[derive(Copy, Clone, Debug)]
enum Said {
    Level(u8),
    Heard,
    Stage(usize),
    /// A piece at its place, the set text when true.
    Piece(u16, bool),
    Done(u32),
    Wiped,
    Fault(usize),
}

impl Said {
    fn note(self) -> SpeechNote {
        match self {
            Self::Level(level) => SpeechNote::Level(level),
            Self::Heard => SpeechNote::Heard,
            Self::Stage(i) => SpeechNote::Stage(STAGES[i]),
            Self::Piece(index, set) => SpeechNote::Piece {
                index,
                text: secret(if set { SET } else { "other words" }),
            },
            Self::Done(wait_ms) => done(wait_ms, wait_ms),
            Self::Wiped => SpeechNote::Wiped,
            Self::Fault(i) => SpeechNote::Fault(FAULTS[i]),
        }
    }
}

/// One thing Compare meets.
#[derive(Clone, Debug)]
enum Step {
    /// Start with this many models, on cards where true, with the set text when true.
    Start(Vec<bool>, bool),
    Stop,
    Cancel,
    Reset,
    /// A note to the last run, or to the one before when true.
    Note(Said, bool),
}

fn said() -> impl Strategy<Value = Said> {
    prop_oneof![
        1 => any::<u8>().prop_map(Said::Level),
        2 => Just(Said::Heard),
        1 => (0..STAGES.len()).prop_map(Said::Stage),
        2 => (0..MAX_INDEX, any::<bool>()).prop_map(|(i, set)| Said::Piece(i, set)),
        3 => any::<u16>().prop_map(|ms| Said::Done(u32::from(ms))),
        1 => Just(Said::Wiped),
        1 => (0..FAULTS.len()).prop_map(Said::Fault),
    ]
}

fn step() -> impl Strategy<Value = Step> {
    let models = vec(any::<bool>(), 0..=usize::from(MAX_MODELS));
    prop_oneof![
        2 => (models, any::<bool>()).prop_map(|(m, set)| Step::Start(m, set)),
        2 => Just(Step::Stop),
        1 => Just(Step::Cancel),
        1 => Just(Step::Reset),
        8 => (said(), prop::bool::weighted(OLD)).prop_map(|(s, old)| Step::Note(s, old)),
    ]
}

/// What the case saw so far.
#[derive(Default)]
struct Seen {
    opened: BTreeSet<u32>,
    closed: BTreeSet<u32>,
    saved: BTreeSet<u32>,
    /// The last run opened.
    last: u32,
    /// The last run's set text was empty.
    bare: bool,
}

/// Feeds `step` to `compare`; an older run's note must change nothing.
fn act(compare: &mut Compare, step: &Step, seen: &Seen) -> Result<Vec<Act>, TestCaseError> {
    Ok(match step {
        Step::Start(cards, set) => {
            let models = (1..)
                .zip(cards)
                .map(|(n, &card)| entrant(n, card))
                .collect();
            let set = if *set { SET } else { "" };
            compare.start(plan(set, models), &settings())
        }
        Step::Stop => compare.stop(),
        Step::Cancel => {
            let acts = compare.cancel();
            prop_assert!(!compare.busy(), "Cancel always rests");
            acts
        }
        Step::Reset => {
            compare.reset();
            Vec::new()
        }
        Step::Note(said, old) => {
            let take = seen.last.wrapping_sub(u32::from(*old));
            let before = (compare.lend(), compare.results().len());
            let acts = compare.note(take, said.note());
            if take != seen.last {
                prop_assert!(acts.is_empty(), "an older run's note acted");
                prop_assert_eq!(before, (compare.lend(), compare.results().len()));
            }
            acts
        }
    })
}

impl Seen {
    /// Checks one step's effects: one Open per run, calls only on the open run, Level only while recording.
    fn effects(&mut self, acts: &[Act], compare: &Compare) -> Result<(), TestCaseError> {
        for a in acts {
            match a {
                Act::Open { take, ask } => {
                    prop_assert!(
                        *take != self.last && self.opened.insert(*take),
                        "{take} reopened"
                    );
                    prop_assert!(ask.model.is_none(), "the recording loads no model");
                    let set = compare.plan().map(|p| p.set.expose().is_empty());
                    (self.last, self.bare) = (*take, set.unwrap_or(false));
                }
                Act::Run { take, call } => {
                    let live = *take == self.last && !self.closed.contains(take);
                    prop_assert!(live, "{call:?} on run {take}");
                    if *call == RunCall::Close {
                        self.closed.insert(*take);
                    }
                }
                Act::Level(_) => prop_assert!(compare.recording(), "a level while not recording"),
                Act::Save(board) => {
                    prop_assert!(
                        self.saved.insert(self.last),
                        "run {} saved twice",
                        self.last
                    );
                    prop_assert_eq!(board.scores.len(), compare.results().len());
                }
                Act::Fell { .. } => {}
            }
        }
        Ok(())
    }

    /// Checks the state after a step: the lend follows the stage and a resting Compare holds no open run.
    fn state(&self, compare: &Compare) -> Result<(), TestCaseError> {
        let lend = compare.lend();
        prop_assert_eq!(compare.busy(), lend != Lend::Off);
        prop_assert_eq!(compare.recording(), lend == Lend::Listen);
        prop_assert_eq!(compare.running().is_some(), lend == Lend::Write);
        let open = self.opened.len() - self.closed.len();
        prop_assert_eq!(
            open,
            usize::from(compare.busy()),
            "an open run without a busy Compare"
        );
        let models = compare.plan().map_or(0, |p| p.entrants.len());
        let results = compare.results();
        prop_assert!(results.is_empty() || !compare.busy(), "results while busy");
        prop_assert!(results.len() <= models, "more results than models");
        let texts = results.iter().all(|l| l.text.is_some() == self.bare);
        prop_assert!(texts, "a text shown with a set text, or none without");
        Ok(())
    }
}

proptest! {
    #[test]
    fn any_run_keeps_its_lend_runs_and_numbers_right(steps in vec(step(), 1..MAX_STEPS)) {
        let mut compare = Compare::default();
        let mut seen = Seen::default();
        for step in &steps {
            let acts = act(&mut compare, step, &seen)?;
            seen.effects(&acts, &compare)?;
            seen.state(&compare)?;
        }
    }
}
