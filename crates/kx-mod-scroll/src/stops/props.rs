//! Properties over any run of taps, readings, presses, failures and waits, under any stop settings: each stop has its
//! cause, and every stop leaves the pad still.

use super::*;
use crate::fixture::settings as fixture;
use crate::settings::{LIMIT_S, NO_LIMIT};
use crate::{Change, Pad};
use kx_module_api::US_PER_S;
use kx_module_api::ports::Dir;
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::select;

/// Random runs per property.
const CASES: u32 = 512;
/// Most steps in one run.
const MAX_STEPS: usize = 60;
/// How many of the longest limits the longest wait spans, so some waits pass the limit.
const LONG_WAIT: u64 = 2;

/// One thing the actor meets, as the platform and the user give it.
#[derive(Copy, Clone, Debug)]
enum Step {
    /// A pad tap; the aim's error, if any, and the reach each way.
    Tap {
        dir: Dir,
        aim: Option<ScrollError>,
        ahead: Reach,
        back: Reach,
    },
    /// The Stop button.
    Stop,
    /// A tick's reach reading.
    Read(Reach),
    /// A press, on our windows or not.
    Press(bool),
    /// A failed aim or turn.
    Fail(ScrollError),
    /// Time passes, in µs.
    Wait(u64),
}

fn reach() -> impl Strategy<Value = Reach> {
    select(vec![Reach::More, Reach::End, Reach::Unknown])
}

fn error() -> impl Strategy<Value = ScrollError> {
    select(vec![
        ScrollError::NoTarget,
        ScrollError::Gone,
        ScrollError::Refused,
    ])
}

/// The longest limit, in µs.
fn longest() -> u64 {
    u64::from(*LIMIT_S.end()) * u64::from(US_PER_S)
}

fn step() -> impl Strategy<Value = Step> {
    let dir = select(Dir::ALL.to_vec());
    let tap = (dir, proptest::option::of(error()), reach(), reach()).prop_map(
        |(dir, aim, ahead, back)| Step::Tap {
            dir,
            aim,
            ahead,
            back,
        },
    );
    prop_oneof![
        3 => tap,
        1 => Just(Step::Stop),
        2 => reach().prop_map(Step::Read),
        2 => any::<bool>().prop_map(Step::Press),
        1 => error().prop_map(Step::Fail),
        2 => (0..=LONG_WAIT * longest()).prop_map(Step::Wait),
    ]
}

/// Any stop settings: each switch, and no limit or any limit.
fn settings() -> impl Strategy<Value = ScrollSettings> {
    let limit = prop_oneof![Just(NO_LIMIT), LIMIT_S];
    (limit, any::<bool>(), any::<bool>()).prop_map(|(limit_s, stop_at_end, stop_on_press)| {
        ScrollSettings {
            limit_s,
            stop_at_end,
            stop_on_press,
            ..fixture()
        }
    })
}

/// The actor's rules for one step, taking a running scroll to have a target (`tests/scroll/taps.rs` covers a pending
/// first aim): the halt it meets, if any. Every tap that does not stop aims, and a new way reads the reach of the
/// target found, or of the running scroll's when the OS refused the aim.
fn rule(pad: &mut Pad, stops: &mut Stops, step: Step, now: Mono) -> Option<Halt> {
    let live = pad.scrolling();
    match step {
        Step::Tap {
            dir,
            aim,
            ahead,
            back,
        } => {
            stops.tapped(now);
            let change = pad.tap(dir);
            if change == Change::Stopped {
                return None;
            }
            let reads = change.new_way() && (aim.is_none() || live);
            let blocked = || reads.then(|| stops.blocked(ahead, || back)).flatten();
            aim.and_then(Halt::unaimed).or_else(blocked)
        }
        Step::Stop => {
            pad.stop();
            None
        }
        Step::Read(reach) if live => stops.reached(reach),
        Step::Press(ours) if live => stops.pressed(ours),
        Step::Fail(err) if live => Halt::failed(err),
        Step::Wait(_) if live => stops.timed_out(now),
        _ => None,
    }
}

/// The halt a step from `before` may meet under `s`; `since` is the time since the last tap.
fn want(s: &ScrollSettings, before: Pad, step: Step, since: Duration) -> Option<Halt> {
    let mut after = before;
    match step {
        Step::Tap {
            dir,
            aim,
            ahead,
            back,
        } => {
            let change = after.tap(dir);
            // A refused aim keeps a running scroll's target; from still there is none to read yet.
            let reads = change.new_way() && (aim.is_none() || before.scrolling());
            if change == Change::Stopped {
                None
            } else {
                tap_cause(s, aim, reads, ahead, back)
            }
        }
        _ if before.scrolling() => cause(s, step, since),
        _ => None,
    }
}

/// The halt a tap that does not stop allows under `s`: the aim's error, then the reach each way if the tap `reads` it;
/// with `stop_at_end` off, an end only that way starts, as a feed may load more there.
fn tap_cause(
    s: &ScrollSettings,
    aim: Option<ScrollError>,
    reads: bool,
    ahead: Reach,
    back: Reach,
) -> Option<Halt> {
    match (aim, ahead, back) {
        (Some(ScrollError::NoTarget), ..) => Some(Halt::NoPoint),
        (Some(ScrollError::Gone), ..) => Some(Halt::Nothing),
        _ if !reads => None,
        (_, Reach::End, Reach::End) => Some(Halt::Nothing),
        (_, Reach::End, _) if s.stop_at_end => Some(Halt::AtEnd),
        _ => None,
    }
}

/// The halt a running scroll's step allows under `s`; `since` is the time since the last tap.
fn cause(s: &ScrollSettings, step: Step, since: Duration) -> Option<Halt> {
    match step {
        Step::Read(Reach::End) if s.stop_at_end => Some(Halt::End),
        Step::Press(false) if s.stop_on_press => Some(Halt::Press),
        Step::Fail(ScrollError::NoTarget | ScrollError::Gone) => Some(Halt::Gone),
        Step::Wait(_) if due(s, since) => Some(Halt::Limit),
        _ => None,
    }
}

/// Whether `since` the last tap has reached the limit in `s`.
fn due(s: &ScrollSettings, since: Duration) -> bool {
    s.limit_s != NO_LIMIT && since.as_secs() >= u64::from(s.limit_s)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    /// Each halt is the one its cause allows under the settings, only a tap's halt is told, and after any stop the
    /// pad is still.
    #[test]
    fn each_stop_has_its_cause_and_leaves_the_pad_still(
        s in settings(),
        steps in vec(step(), 0..=MAX_STEPS),
    ) {
        let (mut pad, mut stops) = (Pad::default(), Stops::from(&s));
        let (mut now, mut last_tap) = (0, 0);
        for step in steps {
            let before = pad;
            if let Step::Wait(gap) = step {
                now += gap;
            }
            let halt = rule(&mut pad, &mut stops, step, Mono::from_us(now));
            if let Step::Tap { .. } = step {
                last_tap = now;
            }
            let since = Duration::from_micros(now - last_tap);
            prop_assert_eq!(halt, want(&s, before, step, since), "{:?} from {:?}", step, before);
            if let Some(halt) = halt {
                prop_assert_eq!(halt.told(), matches!(step, Step::Tap { .. }));
                pad.stop();
            }
            if halt.is_some() || matches!(step, Step::Stop) {
                prop_assert_eq!(pad, Pad::Idle);
            }
        }
    }

    /// With a limit, scrolling times out exactly when the time since the last tap reaches it; with none, never.
    #[test]
    fn the_limit_counts_from_the_last_tap(
        s in settings(),
        taps in vec(0..=longest(), 1..=MAX_STEPS),
        after in 0..=LONG_WAIT * longest(),
    ) {
        let mut stops = Stops::from(&s);
        let mut now = 0;
        for gap in taps {
            now += gap;
            stops.tapped(Mono::from_us(now));
        }
        let want = due(&s, Duration::from_micros(after)).then_some(Halt::Limit);
        prop_assert_eq!(stops.timed_out(Mono::from_us(now + after)), want);
    }
}
