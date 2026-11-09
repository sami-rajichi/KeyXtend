//! The inbox over any run of changes, ours and another app's, jobs, waits and takes on a clock the test moves: jobs
//! leave in the order they came, a change is told once before the next job, only once its newest change settled and
//! as the pacing says, ours is never told, no take waits, and an ended inbox takes and tells nothing.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::time::Instant;

use kx_module_api::Mono;
use kx_module_api::ports::SelKind;
use proptest::collection::vec;
use proptest::prelude::*;

use super::super::rig::{Script, own, theirs};
use super::super::run::Ask;
use super::*;
use crate::worker::fake::Hands;

/// Random runs per property.
const CASES: u32 = 256;
/// Most steps in one run.
const MAX_STEPS: usize = 60;
/// The longest settle and wait tried, in ms.
const MAX_MS: u32 = 300;
/// The most looks a second tried.
const MAX_LOOKS: u16 = 4;

/// One thing the inbox meets.
#[derive(Copy, Clone, Debug)]
enum Step {
    /// A change in our own process (true) or in another app.
    Change(bool),
    /// The run queues a job.
    Push,
    /// The thread takes the next job, when one is queued.
    Take,
    /// Time passes, in ms.
    Wait(u32),
}

fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        any::<bool>().prop_map(Step::Change),
        Just(Step::Push),
        Just(Step::Take),
        (0..=MAX_MS).prop_map(Step::Wait),
    ]
}

/// The inbox and what the test expects of it: the pacing it should follow, the jobs queued by number, and the newest
/// change still owed a tell.
struct Rig {
    shared: Shared,
    clock: Hands,
    tells: Arc<AtomicUsize>,
    pace: Pace,
    queue: VecDeque<u64>,
    pushed: u64,
    newest: Option<Mono>,
    settle: Duration,
    base: Instant,
}

impl Rig {
    /// An inbox paced as `cfg` says on a clock the test moves, counting its tells.
    fn new(cfg: SelConfig) -> Self {
        let (clock, tells) = (Hands::default(), Arc::new(AtomicUsize::new(0)));
        let count = Arc::clone(&tells);
        let tell = Box::new(move || {
            count.fetch_add(1, Ordering::SeqCst);
        });
        Self {
            shared: Shared::new(cfg, Arc::new(clock.clone()), tell),
            clock,
            tells,
            pace: Pace::new(cfg),
            queue: VecDeque::new(),
            pushed: 0,
            newest: None,
            settle: Duration::from_millis(u64::from(cfg.settle_ms)),
            base: Instant::now(),
        }
    }

    /// Queues the next job, named by its deadline, which no test waits for.
    fn push(&mut self) -> Result<(), SelError> {
        let (reply, _) = mpsc::channel();
        let by = self.base + Duration::from_millis(self.pushed);
        self.shared.push(Job::new(Ask::Clip { by, reply }))?;
        self.queue.push_back(self.pushed);
        self.pushed += 1;
        Ok(())
    }

    /// Takes the next job, checking its number and the tell owed before it.
    fn take(&mut self) -> Result<(), TestCaseError> {
        let Some(want) = self.queue.pop_front() else {
            return Ok(());
        };
        let (now, before) = (self.clock.now(), self.tells.load(Ordering::SeqCst));
        let told = self.pace.tell(now);
        let Some(Ask::Clip { by, .. }) = self.shared.next().map(|j| j.ask) else {
            return Err(TestCaseError::fail("no job, or another kind"));
        };
        prop_assert_eq!(
            by.duration_since(self.base),
            Duration::from_millis(want),
            "out of order"
        );
        prop_assert_eq!(
            self.tells.load(Ordering::SeqCst) - before,
            usize::from(told)
        );
        if told {
            let settled = self
                .newest
                .is_some_and(|at| now >= at.saturating_add(self.settle));
            prop_assert!(settled, "told before the newest change settled");
            self.newest = None;
        }
        Ok(())
    }
}

/// Runs `steps` on an inbox paced as `cfg` says, then ends it.
fn run(cfg: SelConfig, steps: &[Step]) -> Result<(), TestCaseError> {
    let mut rig = Rig::new(cfg);
    let probe = Script::on(Ok(theirs(SelKind::Editable, 1)));
    for &step in steps {
        match step {
            Step::Change(ours) => {
                let el = if ours {
                    own(1)
                } else {
                    theirs(SelKind::Editable, 1)
                };
                rig.shared.moved(&probe, &el);
                if !ours {
                    rig.pace.change(rig.clock.now());
                    rig.newest = Some(rig.clock.now());
                }
            }
            Step::Push => prop_assert!(rig.push().is_ok()),
            Step::Take => rig.take()?,
            Step::Wait(ms) => rig.clock.advance(Duration::from_millis(u64::from(ms))),
        }
    }
    prop_assert_eq!(rig.shared.timed(), 0, "a take waited");
    let told = rig.tells.load(Ordering::SeqCst);
    rig.shared.stop();
    rig.shared.moved(&probe, &theirs(SelKind::Editable, 1));
    prop_assert_eq!(rig.push(), Err(SelError::Unavailable));
    prop_assert!(
        rig.shared.next().is_none(),
        "an ended inbox hands out nothing"
    );
    prop_assert_eq!(
        rig.tells.load(Ordering::SeqCst),
        told,
        "an ended inbox tells nothing"
    );
    Ok(())
}

fn config() -> impl Strategy<Value = SelConfig> {
    (1..=MAX_MS, 0..=MAX_LOOKS).prop_map(|(settle_ms, looks_per_s)| SelConfig {
        settle_ms,
        read_ms: MAX_MS,
        looks_per_s,
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    /// Any run hands jobs out in order, tells each settled change of another app once before the next job, as the
    /// pacing says, never tells ours, never waits on a take, and stops for good.
    #[test]
    fn jobs_leave_in_order_and_each_settled_change_is_told_once_before_the_next(
        cfg in config(),
        steps in vec(step(), 1..=MAX_STEPS),
    ) {
        run(cfg, &steps)?;
    }
}
