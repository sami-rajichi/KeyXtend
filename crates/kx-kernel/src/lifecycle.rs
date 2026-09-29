//! The module lifecycle: the one table of the steps that move a module between states.

use kx_module_api::ModuleState;
use thiserror::Error;

/// What happens to a module.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Step {
    /// Its start begins: first run, Try again, or switched back on.
    Start,
    /// Its start succeeded.
    Started,
    /// Its start failed, it panicked, or a module it requires failed first.
    Fail,
    /// Its stop begins.
    Stop,
    /// Its stop finished.
    Stopped,
    /// It is switched off, or waits for a switched-off module, before it ran or after it failed.
    Skip,
}

/// A step the lifecycle table refuses in a state; the caller keeps the old state.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
#[error("lifecycle step {step:?} is not allowed in state {state:?}")]
pub struct BadStep {
    /// The state the module is in.
    pub state: ModuleState,
    /// The refused step.
    pub step: Step,
}

/// Every allowed move as (from, step, to); ARCHITECTURE.md points here as the one source.
const MOVES: [(ModuleState, Step, ModuleState); 10] = {
    use ModuleState::{Active, Failed, Pending, Starting, Stopped, Stopping};
    [
        (Pending, Step::Start, Starting),
        (Pending, Step::Fail, Failed),
        (Pending, Step::Skip, Stopped),
        (Starting, Step::Started, Active),
        (Starting, Step::Fail, Failed),
        (Active, Step::Stop, Stopping),
        (Stopping, Step::Stopped, Stopped),
        (Failed, Step::Start, Starting),
        (Failed, Step::Skip, Stopped),
        (Stopped, Step::Start, Starting),
    ]
};

/// The state a module in `state` reaches through `step`.
///
/// # Errors
/// `BadStep` when the table has no such move.
pub fn next(state: ModuleState, step: Step) -> Result<ModuleState, BadStep> {
    MOVES
        .iter()
        .find(|&&(from, by, _)| from == state && by == step)
        .map(|&(_, _, to)| to)
        .ok_or(BadStep { state, step })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ModuleState::{Active, Failed, Pending, Starting, Stopped, Stopping};
    use proptest::prelude::*;

    /// The diagram as the test expects it, written apart from `MOVES`.
    const DIAGRAM: [(ModuleState, Step, ModuleState); 10] = [
        (Pending, Step::Start, Starting),
        (Pending, Step::Fail, Failed),
        (Pending, Step::Skip, Stopped),
        (Starting, Step::Started, Active),
        (Starting, Step::Fail, Failed),
        (Active, Step::Stop, Stopping),
        (Stopping, Step::Stopped, Stopped),
        (Failed, Step::Start, Starting),
        (Failed, Step::Skip, Stopped),
        (Stopped, Step::Start, Starting),
    ];

    const STEPS: [Step; 6] = [
        Step::Start,
        Step::Started,
        Step::Fail,
        Step::Stop,
        Step::Stopped,
        Step::Skip,
    ];

    fn in_diagram(state: ModuleState, step: Step) -> bool {
        DIAGRAM
            .iter()
            .any(|&(from, by, _)| from == state && by == step)
    }

    #[test]
    fn each_allowed_move_reaches_its_state() {
        for (from, step, to) in DIAGRAM {
            assert_eq!(next(from, step), Ok(to), "{from:?} by {step:?}");
        }
    }

    #[test]
    fn refused_moves_name_the_state_and_step() {
        let refused = [
            (Active, Step::Start),
            (Stopped, Step::Started),
            (Pending, Step::Stop),
            (Stopping, Step::Fail),
            (Failed, Step::Started),
            (Active, Step::Skip),
        ];
        for (state, step) in refused {
            assert_eq!(next(state, step), Err(BadStep { state, step }));
        }
    }

    #[test]
    fn a_bad_step_message_names_both() {
        let text = BadStep {
            state: Active,
            step: Step::Start,
        }
        .to_string();
        assert!(text.contains("Active") && text.contains("Start"), "{text}");
    }

    proptest! {
        /// Every visited state is reached from Pending by diagram moves only.
        #[test]
        fn random_steps_only_take_diagram_moves(
            steps in prop::collection::vec(prop::sample::select(STEPS.to_vec()), 0..64)
        ) {
            let mut state = Pending;
            for step in steps {
                match next(state, step) {
                    Ok(to) => {
                        prop_assert!(DIAGRAM.contains(&(state, step, to)));
                        state = to;
                    }
                    Err(bad) => {
                        prop_assert!(!in_diagram(state, step));
                        prop_assert_eq!(bad, BadStep { state, step });
                    }
                }
            }
        }
    }
}
