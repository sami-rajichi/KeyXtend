//! Any run of asks, faults, quits and waits: one worker at most, no ask id twice, nothing sent or started while the
//! worker rests, and none left running at the end.

use super::*;
use proptest::prelude::*;

/// An id no ask of a short run reaches.
const STRAY: u32 = u32::MAX;
/// Milliseconds in a second.
const MS_PER_S: u32 = 1_000;
/// The longest run.
const STEPS: usize = 60;

/// One thing that happens to the host.
#[derive(Clone, Debug)]
enum Step {
    /// An ask the worker meets with this act.
    Ask(Act),
    /// The worker quits between asks.
    Quit,
    /// The next start is refused.
    Refuse,
    /// The idle check.
    Idle,
    /// The clock moves this many ms.
    Wait(u32),
}

fn step() -> impl Strategy<Value = Step> {
    let stray = Reply::Failed {
        id: STRAY,
        why: Why::Refused,
    };
    let acts = vec![
        Act::Draw(1, 1),
        Act::Silent,
        Act::Dib(1),
        Act::Fail(Why::Internal),
        Act::Broken(IpcError::Closed),
        Act::Broken(IpcError::BadMagic),
        Act::Raw(stray),
    ];
    prop_oneof![
        prop::sample::select(acts).prop_map(Step::Ask),
        Just(Step::Quit),
        Just(Step::Refuse),
        Just(Step::Idle),
        (0..=2 * TIMES.restart_max_ms).prop_map(Step::Wait),
        Just(Step::Wait(TIMES.idle_s * MS_PER_S)),
    ]
}

/// The ids asked of the workers so far, in order.
fn ids(script: &Script<MediaFrames>) -> Vec<u32> {
    let id = |call: &Call| match call {
        Call::Draw { id, .. } => Some(*id),
        _ => None,
    };
    script.calls().iter().filter_map(id).collect()
}

/// How many workers were ended so far.
fn ends(script: &Script<MediaFrames>) -> usize {
    let ended = script.calls().into_iter().filter(|c| *c == Call::End);
    ended.count()
}

/// An ask met with `act`; `rest` is how long the backoff still runs, as the last refusal told.
fn ask(
    host: &mut Media,
    script: &Script<MediaFrames>,
    act: Act,
    rest: &mut Duration,
) -> Result<(), TestCaseError> {
    let (tries, sent) = (script.tries(), ids(script));
    script.act(act);
    let heard = ping(host);
    if !rest.is_zero() {
        prop_assert_eq!(&heard, &down(*rest), "still down, for what is left");
        prop_assert_eq!(script.tries(), tries, "no start within the backoff");
    }
    *rest = Duration::ZERO;
    match heard {
        Err(Miss::Lost(Lost::Down { retry_in })) => {
            prop_assert!(!retry_in.is_zero() && retry_in <= ms(TIMES.restart_max_ms));
            prop_assert_eq!(ids(script), sent, "a resting worker is asked nothing");
            *rest = retry_in;
        }
        Ok(id) => prop_assert_eq!(ids(script).last().copied(), Some(id), "its own answer"),
        Err(_) => {}
    }
    Ok(())
}

/// What holds after every step.
fn check(script: &Script<MediaFrames>) -> Result<(), TestCaseError> {
    let (spawns, ends) = (script.spawns(), ends(script));
    prop_assert!(ends <= spawns && spawns - ends <= 1, "one worker at most");
    prop_assert!(ids(script).is_sorted_by(|a, b| a < b), "no id twice");
    Ok(())
}

proptest! {
    #[test]
    fn any_run_keeps_one_worker_at_most_and_never_asks_or_starts_a_resting_one(
        steps in prop::collection::vec(step(), 0..STEPS),
    ) {
        let (script, clock) = (Script::default(), Hands::default());
        let mut host = host(&script, &clock);
        let mut rest = Duration::ZERO;
        for step in steps {
            match step {
                Step::Ask(act) => ask(&mut host, &script, act, &mut rest)?,
                Step::Quit => script.quit(DIED),
                Step::Refuse => script.refuse(1),
                Step::Idle => host.idle(),
                Step::Wait(n) => {
                    clock.advance(ms(n));
                    rest = rest.saturating_sub(ms(n));
                }
            }
            check(&script)?;
        }
        drop(host);
        prop_assert_eq!(ends(&script), script.spawns(), "none left running");
    }
}
