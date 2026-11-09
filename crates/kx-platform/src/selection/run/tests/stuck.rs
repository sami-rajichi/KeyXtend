//! The hung-skip state over any run of jobs started, given up and ended: the thread is skipped exactly while the job
//! given up last still runs, and used again once that job ends.

use super::*;
use proptest::collection::vec;
use proptest::prelude::*;

/// Random runs per property.
const CASES: u32 = 256;
/// Most steps in one run.
const MAX_STEPS: usize = 60;

/// One thing the run or the thread does.
#[derive(Copy, Clone, Debug)]
enum Step {
    /// The run hands a new job to the thread.
    Start,
    /// The run gives up on the newest job at its cap.
    GiveUp,
    /// The thread ends one of the jobs it holds, by its place among them.
    End(usize),
    /// The run asks whether the thread is busy.
    Ask,
}

fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        Just(Step::Start),
        Just(Step::GiveUp),
        any::<usize>().prop_map(Step::End),
        Just(Step::Ask),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    /// The thread reads busy exactly while the job given up last runs, and never again once that job ended.
    #[test]
    fn the_thread_is_skipped_only_while_the_job_given_up_last_runs(steps in vec(step(), 1..=MAX_STEPS)) {
        let (mut stuck, mut jobs) = (Stuck::default(), Vec::<Arc<()>>::new());
        let mut given: Option<Weak<()>> = None;
        for step in steps {
            match step {
                Step::Start => jobs.push(Arc::new(())),
                Step::GiveUp => {
                    if let Some(job) = jobs.last() {
                        stuck.gave_up(Arc::downgrade(job));
                        given = Some(Arc::downgrade(job));
                    }
                }
                Step::End(at) if !jobs.is_empty() => drop(jobs.remove(at % jobs.len())),
                Step::End(_) => {}
                Step::Ask => {
                    let runs = given.as_ref().is_some_and(|g| g.strong_count() > 0);
                    prop_assert_eq!(stuck.busy(), runs);
                    if !runs {
                        given = None;
                    }
                }
            }
        }
    }
}
