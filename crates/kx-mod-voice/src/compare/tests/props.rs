//! Any run of Starts, Stops, Cancels, resets, notes and cloud notes, an older run's included: the lend follows the
//! stage, each run opens once and closes once with no call after, a resting Compare holds no open run, Cancel always
//! rests, the numbers are kept at most once a run and texts show only with no set text. Only the ticked cloud models
//! get the recording, once, after Start and the recording's end, never after a Halt; Cancel halts any still writing.

use super::*;
use kx_module_api::ports::{CloudFault, CloudNote};
use proptest::collection::vec;
use proptest::prelude::*;
use seen::Seen;

mod seen;

/// The most models one run compares.
const MAX_MODELS: u8 = 3;
/// The most cloud models one run sends to.
const MAX_CLOUDS: u8 = 2;
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
                text: words(set),
            },
            Self::Done(wait_ms) => done(wait_ms, wait_ms),
            Self::Wiped => SpeechNote::Wiped,
            Self::Fault(i) => SpeechNote::Fault(FAULTS[i]),
        }
    }
}

/// What a cloud model tells: a piece at its place (the set text when true), its end after a wait, or its fault.
#[derive(Copy, Clone, Debug)]
enum Net {
    Piece(u16, bool),
    Done(u32),
    Fault,
}

impl Net {
    /// The note and its wait.
    fn note(self) -> (CloudNote, u32) {
        match self {
            Self::Piece(index, set) => (
                CloudNote::Piece {
                    index,
                    text: words(set),
                },
                0,
            ),
            Self::Done(wait_ms) => (CloudNote::Done, wait_ms),
            Self::Fault => (CloudNote::Fault(CloudFault::Credit), 0),
        }
    }
}

/// The set text when `set`, else other words.
fn words(set: bool) -> Secret {
    secret(if set { SET } else { "other words" })
}

/// One thing Compare meets.
#[derive(Clone, Debug)]
enum Step {
    /// Start with this many models, on cards where true, with the set text when true, and this many cloud models.
    Start(Vec<bool>, bool, u8),
    Stop,
    Cancel,
    Reset,
    /// A note to the last run, or to the one before when true.
    Note(Said, bool),
    /// A note from cloud model `at` to the last run, or to the one before when true.
    Net(usize, Net, bool),
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

fn net() -> impl Strategy<Value = Net> {
    prop_oneof![
        2 => (0..MAX_INDEX, any::<bool>()).prop_map(|(i, set)| Net::Piece(i, set)),
        2 => any::<u16>().prop_map(|ms| Net::Done(u32::from(ms))),
        1 => Just(Net::Fault),
    ]
}

fn step() -> impl Strategy<Value = Step> {
    let models = vec(any::<bool>(), 0..=usize::from(MAX_MODELS));
    let at = 0..usize::from(MAX_CLOUDS);
    prop_oneof![
        2 => (models, any::<bool>(), 0..=MAX_CLOUDS).prop_map(|(m, set, c)| Step::Start(m, set, c)),
        2 => Just(Step::Stop),
        1 => Just(Step::Cancel),
        1 => Just(Step::Reset),
        8 => (said(), prop::bool::weighted(OLD)).prop_map(|(s, old)| Step::Note(s, old)),
        4 => (at, net(), prop::bool::weighted(OLD)).prop_map(|(at, n, old)| Step::Net(at, n, old)),
    ]
}

/// Feeds `step` to `compare`; an older run's note must change nothing.
fn act(compare: &mut Compare, step: &Step, seen: &Seen) -> Result<Vec<Act>, TestCaseError> {
    let take = |old: bool| seen.last.wrapping_sub(u32::from(old));
    Ok(match step {
        Step::Start(cards, set, clouds) => {
            let models = (1..).zip(cards).map(|(n, &c)| entrant(n, c)).collect();
            let set = if *set { SET } else { "" };
            let clouds = (1..=*clouds).map(remote).collect();
            compare.start(
                Plan {
                    clouds,
                    ..plan(set, models)
                },
                &settings(),
            )
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
        Step::Note(said, old) => told(compare, take(*old), seen, |c, t| c.note(t, said.note()))?,
        Step::Net(at, net, old) => {
            let (note, wait_ms) = net.note();
            told(compare, take(*old), seen, |c, t| {
                c.net(t, *at, note, wait_ms, wall())
            })?
        }
    })
}

/// Tells run `take` with `tell`; an older run's note must act on nothing and change nothing.
fn told(
    compare: &mut Compare,
    take: u32,
    seen: &Seen,
    tell: impl FnOnce(&mut Compare, u32) -> Vec<Act>,
) -> Result<Vec<Act>, TestCaseError> {
    let before = (compare.lend(), compare.results().len());
    let acts = tell(compare, take);
    if take != seen.last {
        prop_assert!(acts.is_empty(), "an older run's note acted");
        prop_assert_eq!(before, (compare.lend(), compare.results().len()));
    }
    Ok(acts)
}

/// True while one of the run's cloud models still writes.
fn cloud_writing(compare: &Compare) -> bool {
    let clouds = compare.plan().map_or(0, |p| p.clouds.len());
    let running = compare.running();
    running.is_some_and(|(_, runs)| {
        runs.iter()
            .rev()
            .take(clouds)
            .any(|(_, s)| *s == RunStep::Writing)
    })
}

proptest! {
    #[test]
    fn any_run_keeps_its_lend_runs_numbers_and_sends_right(steps in vec(step(), 1..MAX_STEPS)) {
        let mut compare = Compare::default();
        let mut seen = Seen::default();
        for step in &steps {
            let writing = cloud_writing(&compare);
            let acts = act(&mut compare, step, &seen)?;
            seen.effects(&acts, &compare)?;
            if writing && matches!(step, Step::Cancel) {
                prop_assert!(acts.contains(&Act::Halt(seen.last)), "Cancel left a cloud model writing");
            }
            if let Step::Note(Said::Stage(i), false) = step {
                seen.mic |= STAGES[*i] == SpeechStage::Open;
            }
            seen.state(&compare)?;
        }
    }
}
