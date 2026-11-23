//! A run under any script and any later answers, cuts and drop: texts in order from piece 0, then at most one end and
//! nothing after it, and a log that matches what was told.

use super::*;
use proptest::prelude::*;

/// What a property does to the run after it is sent.
#[derive(Clone, Debug)]
enum Step {
    /// Answers its next pieces.
    Answer(Vec<CloudAnswer>),
    /// Cuts it with the fault at this place in `FAULTS`.
    Cut(usize),
    /// Drops it.
    Drop,
}

/// A text, or the fault at a place in `FAULTS`.
fn answer() -> impl Strategy<Value = CloudAnswer> {
    let fault = (0..FAULTS.len()).prop_map(|at| CloudAnswer::Fault(FAULTS[at]));
    prop_oneof![3 => Just(text(SAID)), 1 => fault]
}

fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        4 => prop::collection::vec(answer(), 0..3).prop_map(Step::Answer),
        1 => (0..FAULTS.len()).prop_map(Step::Cut),
        1 => Just(Step::Drop),
    ]
}

/// The state a run's notes and its drop make it.
fn state_of(end: Option<&CloudNote>, dropped: bool) -> CloudState {
    match end {
        Some(CloudNote::Done) => CloudState::Done,
        Some(CloudNote::Fault(fault)) => CloudState::Fault(*fault),
        _ if dropped => CloudState::Dropped,
        _ => CloudState::Sending,
    }
}

proptest! {
    #[test]
    fn a_run_tells_texts_in_order_then_at_most_one_end(
        pieces in 1..6usize,
        script in prop::collection::vec(answer(), 0..6),
        steps in prop::collection::vec(step(), 0..6),
    ) {
        let fake = FakeCloud::new();
        fake.script(script);
        let (run, heard) = sent(&fake, &vec![SAMPLE.len(); pieces]);
        let mut run = Some(run);
        for step in steps {
            let before = notes(&heard).len();
            let told = match step {
                Step::Answer(more) => fake.answer(0, more),
                Step::Cut(at) => fake.cut(0, FAULTS[at]),
                Step::Drop => { run = None; false }
            };
            prop_assert_eq!(told, notes(&heard).len() > before);
        }
        let told = notes(&heard);
        let texts = told.iter().take_while(|n| matches!(n, CloudNote::Piece { .. })).count();
        for (at, note) in told[..texts].iter().enumerate() {
            let in_order = matches!(note, CloudNote::Piece { index, .. } if usize::from(*index) == at);
            prop_assert!(in_order, "{:?} at {}", note, at);
        }
        prop_assert!(told.len() <= texts + 1, "at most one end: {:?}", told);
        let log = fake.calls().runs.remove(0);
        let state = state_of(told.get(texts), run.is_none());
        prop_assert_eq!((log.told, log.state), (texts, state));
        prop_assert!(state != CloudState::Done || texts == pieces);
        let cancelled = if state == CloudState::Dropped { pieces - texts } else { 0 };
        prop_assert_eq!(log.cancelled(), cancelled);
    }
}
