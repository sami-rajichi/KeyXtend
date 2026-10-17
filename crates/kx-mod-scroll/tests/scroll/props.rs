//! Properties over random stop settings, taps, stops, peeks, presses, asks, pointer moves, reaches, turn errors and
//! waits: the actor ticks only while it says it scrolls, turns only where the user pointed outside our windows, and
//! leaves nothing.

use crate::rig::{APP, EPS, Heard, LIST, OTHER, Rig, T0, hidden};
use kx_mod_scroll::Msg;
use kx_module_api::Mono;
use kx_module_api::ports::{Aim, Dir, Reach, ScrollChanged, ScrollError};
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::select;
use std::time::Duration;

/// Random runs per property.
const CASES: u32 = 512;
/// Most steps in one run.
const MAX_STEPS: usize = 80;
/// How many checks the longest short wait spans, so checks happen.
const SHORT_CHECKS: u64 = 2;
/// How many limits the longest long wait spans, so some runs time out.
const LONG_LIMITS: u64 = 2;

/// One thing that happens to the actor.
#[derive(Clone, Debug)]
enum Step {
    /// A message from the bus.
    Send(Msg),
    /// The pointer goes somewhere: a target, or an aim that fails.
    Point(Result<Aim, ScrollError>),
    /// The target can or cannot move a way.
    Reach(Dir, Reach),
    /// Turns fail with this error from now on, or work again.
    Fail(Option<ScrollError>),
    /// Time passes by this many µs, then the actor ticks.
    Wait(u64),
}

fn dir() -> impl Strategy<Value = Dir> {
    select(Dir::ALL.to_vec())
}

fn error() -> impl Strategy<Value = ScrollError> {
    select(vec![
        ScrollError::NoTarget,
        ScrollError::Gone,
        ScrollError::Refused,
    ])
}

fn msg() -> impl Strategy<Value = Msg> {
    prop_oneof![
        4 => dir().prop_map(Msg::Tap),
        1 => Just(Msg::Stop),
        1 => any::<bool>().prop_map(Msg::Peek),
        1 => any::<bool>().prop_map(Msg::Pressed),
        1 => Just(Msg::Ask),
    ]
}

/// The longest short and long waits in µs, from the rig's `check_ms` and `limit_s`.
fn waits() -> (u64, u64) {
    let s = Rig::new().settings;
    let check = Duration::from_millis(s.check_ms.into());
    let limit = Duration::from_secs(s.limit_s.into());
    let us = |d: Duration| u64::try_from(d.as_micros()).unwrap();
    (SHORT_CHECKS * us(check), LONG_LIMITS * us(limit))
}

fn step() -> impl Strategy<Value = Step> {
    let (short, long) = waits();
    let aim = select(vec![APP, OTHER, LIST]).prop_map(Ok);
    let reach = select(vec![Reach::More, Reach::End, Reach::Unknown]);
    prop_oneof![
        6 => msg().prop_map(Step::Send),
        2 => prop_oneof![3 => aim, 1 => error().prop_map(Err)].prop_map(Step::Point),
        1 => (dir(), reach).prop_map(|(d, r)| Step::Reach(d, r)),
        1 => proptest::option::of(error()).prop_map(Step::Fail),
        5 => (0..=short).prop_map(Step::Wait),
        1 => (0..=long).prop_map(Step::Wait),
    ]
}

/// The stops a run draws: `stop_at_end`, `stop_on_press`, and `limit_s` as 0 for none or the defaults' limit.
fn stops() -> impl Strategy<Value = (bool, bool, u32)> {
    let limit = Rig::new().settings.limit_s;
    (any::<bool>(), any::<bool>(), select(vec![0, limit]))
}

/// A rig on the defaults with the drawn stops.
fn rig((at_end, on_press, limit): (bool, bool, u32)) -> Rig {
    Rig::with(|host| {
        host.put("stop_at_end", at_end.into());
        host.put("stop_on_press", on_press.into());
        host.put("limit_s", limit.into());
    })
}

/// What the rig heard so far: the latest state and whether the outline shows.
struct Seen {
    state: ScrollChanged,
    outlined: bool,
}

impl Seen {
    fn take(&mut self, heard: Vec<Heard>) {
        for h in heard {
            match h {
                Heard::Changed(state) => self.state = state,
                Heard::Aimed(aimed) => self.outlined = aimed.area.is_some(),
                Heard::Page(_) | Heard::Notice(_) => {}
            }
        }
    }
}

/// Plays `step` on `rig` at `now`, which a wait moves first.
fn play(rig: &mut Rig, step: &Step, now: &mut Mono) {
    match step {
        Step::Send(msg) => {
            rig.clock.set(*now);
            rig.send(*msg);
        }
        Step::Point(found) => rig.target.push_aim(*found),
        Step::Reach(dir, reach) => rig.target.set_reach(*dir, *reach),
        Step::Fail(error) => rig.target.set_turn_error(*error),
        Step::Wait(gap) => {
            *now = Mono::from_us(now.as_us() + gap);
            rig.tick_at(*now);
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn it_glides_only_while_it_says_so_and_only_where_the_user_pointed(
        drawn in stops(),
        steps in vec(step(), 0..=MAX_STEPS),
    ) {
        let mut rig = rig(drawn);
        rig.begin();
        let mut seen = Seen { state: ScrollChanged::OFF, outlined: false };
        seen.take(rig.take());
        let (mut now, mut peek) = (T0, false);
        let check = Duration::from_millis(rig.settings.check_ms.into()).as_secs_f64();
        let most = f64::from(rig.settings.fast) * check;
        for step in &steps {
            let before = seen.state.dir.is_some();
            if let Step::Send(Msg::Peek(on)) = step {
                peek = *on;
            }
            play(&mut rig, step, &mut now);
            seen.take(rig.take());
            let scrolling = seen.state.dir.is_some();
            prop_assert!(seen.state.live);
            prop_assert_eq!(rig.actor.wait().is_some(), scrolling, "ticks only while scrolling");
            let turns = rig.target.take_turns();
            prop_assert!(turns.is_empty() || before, "a turn while still: {:?}", step);
            prop_assert!(turns.iter().all(|t| t.id == APP.id || t.id == OTHER.id), "{:?}", turns);
            prop_assert!(scrolling || peek || !seen.outlined, "an outline left while still");
            let lines: f64 = turns.iter().map(|t| t.lines).sum();
            prop_assert!(lines <= most + EPS, "{} lines piled up in one step", lines);
        }
        prop_assert!(!rig.send(Msg::Quit));
        let heard = rig.take();
        prop_assert_eq!(&heard[heard.len() - 2..], &[Heard::Changed(ScrollChanged::OFF), hidden()]);
        prop_assert!(!rig.target.running());
    }
}
