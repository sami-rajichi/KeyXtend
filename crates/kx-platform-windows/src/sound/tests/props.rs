//! Properties of the mixer over random runs of plays, prepares, faults and silences at test-clock times; a model of
//! the fake system checks every call and every step.

mod model;
mod sim;

use std::collections::VecDeque;
use std::ops::RangeInclusive;
use std::time::Duration;

use kx_module_api::Mono;
use kx_platform::ui::{Clip, ClipId, SpeakerConfig};
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::{select, subsequence};

use super::fake::{self, Call, State, World};
use super::{LOST, NONE};
use crate::sound::{Fault, Mixer};
use sim::Sim;

/// Random runs per property.
const CASES: u32 = 1_024;
/// Most steps in one run, after its first prepare.
const MAX_STEPS: usize = 60;
/// Most voices per clip a prepare asks for.
const MAX_VOICES: usize = 3;
/// Longest idle time a prepare asks for.
const MAX_IDLE_MS: u32 = 500;
/// Longest gap between quick plays: a few clips long, so voices still play.
const QUICK_MS: u32 = 20;
/// Longest silence: past the longest clip and idle time, so the engine stops.
const SILENCE_MS: u32 = 3 * MAX_IDLE_MS;
/// The highest raw id a play asks for: one past the catalog, so no prepare keeps it.
const UNKNOWN: u16 = 9;
/// The most faults one step queues: one more than the retry takes.
const MAX_FAULTS: usize = 3;
/// How far a gain goes past the port's 0–1, both ways.
const GAIN_REACH: f32 = 2.0;
/// The bytes every catalog clip takes its samples from.
static BYTES: [u8; 3_200] = [0; 3_200];

/// Which of the fake's fault queues a step fills.
#[derive(Copy, Clone, Debug)]
enum Queue {
    Starts,
    Masters,
    Plays,
}

impl Queue {
    /// This queue in `state`.
    fn of(self, state: &mut State) -> &mut VecDeque<Fault> {
        match self {
            Self::Starts => &mut state.starts,
            Self::Masters => &mut state.masters,
            Self::Plays => &mut state.plays,
        }
    }
}

/// One thing that happens after a gap.
#[derive(Clone, Debug)]
enum Step {
    /// Clip `id` plays at a gain.
    Play(ClipId, f32),
    /// These clips are prepared.
    Prepare(Vec<Clip>, SpeakerConfig),
    /// The fake's next calls of a kind fail with this fault, this many times.
    Fault(Queue, Fault, usize),
}

impl Step {
    /// Asks `mixer` for this step at `now`, or queues its faults in `world`.
    fn apply(&self, mixer: &mut Mixer<World>, world: &World, now: Mono) {
        match self {
            Self::Play(id, gain) => mixer.play(*id, *gain, now),
            Self::Prepare(clips, config) => mixer.prepare(clips.clone(), *config),
            Self::Fault(queue, fault, n) => {
                let mut state = world.state();
                queue.of(&mut state).extend(std::iter::repeat_n(*fault, *n));
            }
        }
    }
}

/// `ms` milliseconds.
fn millis(ms: u32) -> Duration {
    Duration::from_millis(u64::from(ms))
}

/// The clips a prepare picks from: good ones of 10, 100 and 1 ms, then a partial sample, no bytes, a rate the engine
/// refuses, one the port refuses, and a zero rate.
fn catalog() -> Vec<Clip> {
    let (fast, slow) = (*fake::RATES.end() + 1, *Clip::RATES.start() - 1);
    let specs: [(usize, u32); 8] = [
        (160, 8_000),
        (3_200, 16_000),
        (96, 48_000),
        (19, 8_000),
        (0, 8_000),
        (160, fast),
        (160, slow),
        (160, 0),
    ];
    let clip = |(&(bytes, rate), id)| Clip {
        id: ClipId::new(id),
        rate,
        samples: &BYTES[..bytes],
    };
    specs.iter().zip(1..).map(clip).collect()
}

/// A play of any id, the unknown one too, at a gain in or out of range, infinite or NaN.
fn play() -> impl Strategy<Value = Step> {
    let odd = select(vec![f32::NAN, f32::INFINITY, f32::NEG_INFINITY]);
    let gain = prop_oneof![4 => -GAIN_REACH..=GAIN_REACH, 1 => odd];
    (1..=UNKNOWN, gain).prop_map(|(id, gain)| Step::Play(ClipId::new(id), gain))
}

/// A prepare of some catalog clips, in order, with `voices` and an idle time.
fn prepare(voices: RangeInclusive<usize>) -> impl Strategy<Value = Step> {
    let clips = subsequence(catalog(), 0..=catalog().len());
    (clips, voices, 0..=MAX_IDLE_MS).prop_map(|(clips, voices, idle_ms)| {
        Step::Prepare(clips, SpeakerConfig { voices, idle_ms })
    })
}

/// Lost devices, other failed plays, failed engine starts, and no device.
fn fault() -> impl Strategy<Value = Step> {
    let n = 1..=MAX_FAULTS;
    prop_oneof![
        n.clone().prop_map(|n| Step::Fault(Queue::Plays, LOST, n)),
        Just(Step::Fault(Queue::Plays, NONE, 1)),
        select(vec![LOST, NONE]).prop_map(|f| Step::Fault(Queue::Starts, f, 1)),
        n.prop_map(|n| Step::Fault(Queue::Masters, NONE, n)),
    ]
}

/// No time, a quick gap, or up to a long silence.
fn gap() -> impl Strategy<Value = u32> {
    prop_oneof![2 => Just(0), 3 => 0..=QUICK_MS, 1 => 0..=SILENCE_MS]
}

/// A prepare, then `step`s, each after a gap.
fn run(
    voices: RangeInclusive<usize>,
    step: impl Strategy<Value = Step>,
) -> impl Strategy<Value = Vec<(u32, Step)>> {
    let steps = vec((gap(), step), 0..=MAX_STEPS);
    (prepare(voices), steps).prop_map(|(first, mut steps)| {
        steps.insert(0, (0, first));
        steps
    })
}

/// Plays and prepares with a whole device.
fn calm() -> impl Strategy<Value = Step> {
    prop_oneof![8 => play(), 1 => prepare(1..=MAX_VOICES)]
}

/// Plays and prepares with `voices`, and faults.
fn rough(voices: RangeInclusive<usize>) -> impl Strategy<Value = Step> {
    prop_oneof![8 => play(), 1 => prepare(voices), 3 => fault()]
}

/// Runs `steps` on a mixer alone, waking at each step as the thread does, then drops it; true when every part made
/// is gone.
fn bare(steps: &[(u32, Step)]) -> bool {
    let world = World::default();
    let mut mixer = Mixer::new(world.clone());
    let mut now = Mono::default();
    for (gap, step) in steps {
        now = now.saturating_add(millis(*gap));
        mixer.idle(now);
        step.apply(&mut mixer, &world, now);
        mixer.idle(now);
    }
    drop(mixer);
    let calls = world.take();
    let made = calls.iter().filter(|c| matches!(c, Call::Make(_))).count();
    made == calls.iter().filter(|c| matches!(c, Call::Gone(_))).count()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    /// With a whole device every play the port takes sounds on the voice the rules pick, and the engine stops exactly
    /// once every clip ended and the idle time passed.
    #[test]
    fn a_calm_run_sounds_every_play_and_stops_when_idle(steps in run(1..=MAX_VOICES, calm())) {
        let mut sim = Sim::new();
        sim.run(&steps)?;
        prop_assert_eq!(sim.dropped(), 0, "a whole device sounds every play");
        sim.end()?;
    }

    /// With lost devices, failed starts and plays, and no device, the rules still hold: one rebuild and one retry,
    /// only a second loss in a row drops a play, and each warning kind once until a play sounds.
    #[test]
    fn a_rough_run_rebuilds_once_and_warns_each_kind_once(steps in run(1..=MAX_VOICES, rough(1..=MAX_VOICES))) {
        let mut sim = Sim::new();
        sim.run(&steps)?;
        sim.end()?;
    }

    /// With any voices, even none, no run panics, and the dropped mixer leaves no part it made.
    #[test]
    fn any_run_with_any_voices_leaves_nothing_made(steps in run(0..=MAX_VOICES, rough(0..=MAX_VOICES))) {
        prop_assert!(bare(&steps));
    }
}
