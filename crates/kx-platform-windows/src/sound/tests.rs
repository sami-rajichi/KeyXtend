//! Tests of the speaker's rules over a fake system at set times; the thread's tests are in `actor`.

mod actor;
mod fake;

use std::time::Duration;

use kx_module_api::{Mono, US_PER_MS};
use kx_platform::ui::{Clip, ClipId, SpeakerConfig};

use super::{Fault, Mixer, Warn, pick};
use fake::{Call, Part, World};

const KEY: ClipId = ClipId::new(1);
const DROP: ClipId = ClipId::new(2);
/// The fake clips' rate: 8 kHz, so their 160 bytes play for 10 ms.
const RATE: u32 = 8_000;
static SAMPLES: [u8; 160] = [0; 160];
const IDLE_MS: u32 = 500;
/// A lost device, and no device, with the fake's codes.
const LOST: Fault = Fault::Lost(-1);
const NONE: Fault = Fault::Failed(-2);
/// How long a test waits on the thread; only a failure waits this long.
const LIMIT: Duration = Duration::from_secs(10);

fn clip(id: ClipId) -> Clip {
    Clip {
        id,
        rate: RATE,
        samples: &SAMPLES,
    }
}

fn config(voices: usize) -> SpeakerConfig {
    SpeakerConfig {
        voices,
        idle_ms: IDLE_MS,
    }
}

fn ms(ms: u64) -> Mono {
    Mono::from_us(ms * u64::from(US_PER_MS))
}

/// A mixer over a fresh world, with the key and drop clips at `voices` voices each.
fn mixer(voices: usize) -> (Mixer<World>, World) {
    let world = World::default();
    let mut mixer = Mixer::new(world.clone());
    mixer.prepare(vec![clip(KEY), clip(DROP)], config(voices));
    (mixer, world)
}

/// The calls that tear down a rig of `voices` voices, from voice `first`, and build the next one.
fn rebuilt(first: u32, voices: u32) -> Vec<Call> {
    let old = (first..first + voices).map(|n| Call::Gone(Part::Voice(n)));
    let gone = old.chain([Call::Gone(Part::Master), Call::Gone(Part::Engine)]);
    let made = [Call::Make(Part::Engine), Call::Make(Part::Master)];
    let new = (first + voices..first + 2 * voices).map(|n| Call::Make(Part::Voice(n)));
    gone.chain(made).chain(new).collect()
}

/// Only the warnings among `calls`.
fn warns(calls: Vec<Call>) -> Vec<Call> {
    calls
        .into_iter()
        .filter(|c| matches!(c, Call::Warn(..)))
        .collect()
}

#[test]
fn quick_plays_of_one_clip_use_each_voice_then_replace_the_earliest() {
    let (mut mixer, world) = mixer(2);
    for t in 0..3 {
        mixer.play(KEY, 1.0, ms(t));
    }
    let made = [Part::Engine, Part::Master]
        .into_iter()
        .chain((1..=4).map(Part::Voice));
    let mut want: Vec<Call> = made.map(Call::Make).collect();
    want.extend([Call::Start, Call::Play(1, 1.0), Call::Play(2, 1.0)]);
    want.extend([Call::Make(Part::Voice(5)), Call::Gone(Part::Voice(1))]);
    want.push(Call::Play(5, 1.0));
    assert_eq!(world.take(), want, "the earliest voice is replaced");
    world.state().busy.remove(&2);
    mixer.play(KEY, 1.0, ms(3));
    assert_eq!(
        world.take(),
        [Call::Play(2, 1.0)],
        "an idle voice comes first"
    );
}

#[test]
fn the_pick_takes_an_idle_voice_else_the_earliest_and_none_from_no_voice() {
    let idle = pick([(true, ms(5)), (false, ms(9)), (true, ms(1))]);
    assert_eq!(idle, Some((1, false)));
    let earliest = pick([(true, ms(5)), (true, ms(1)), (true, ms(1))]);
    assert_eq!(earliest, Some((1, true)), "the first of equal starts");
    assert_eq!(pick([]), None);
}

#[test]
fn a_busy_voice_is_replaced_before_it_plays_and_the_engine_stops_after_the_new_end() {
    let (mut mixer, world) = mixer(1);
    mixer.play(KEY, 1.0, ms(0));
    world.take();
    mixer.play(KEY, 0.5, ms(1));
    let want = [
        Call::Make(Part::Voice(3)),
        Call::Gone(Part::Voice(1)),
        Call::Play(3, 0.5),
    ];
    assert_eq!(
        world.take(),
        want,
        "the old voice is destroyed, not queued behind"
    );
    assert_eq!(mixer.wait(ms(1)), Some(Duration::from_millis(510)));
}

#[test]
fn a_failed_engine_start_counts_as_a_lost_device_and_rebuilds_once() {
    let (mut mixer, world) = mixer(1);
    mixer.play(KEY, 1.0, ms(0));
    mixer.idle(ms(510));
    world.take();
    world.state().busy.clear();
    world.state().starts.push_back(NONE);
    mixer.play(KEY, 1.0, ms(10_000));
    let mut want = vec![Call::Start];
    want.extend(rebuilt(1, 2));
    want.extend([Call::Start, Call::Play(3, 1.0)]);
    assert_eq!(world.take(), want, "rebuilt with no warning");
}

#[test]
fn prepare_skips_a_bad_clip_and_warns_each_kind_once_while_the_others_play() {
    static ODD: [u8; 19] = [0; 19];
    let world = World::default();
    let mut mixer = Mixer::new(world.clone());
    // Too fast for the engine, and too slow for the port though the engine takes it.
    let (fast, slow) = (*fake::RATES.end() + 1, Clip::RATES.start() - 1);
    assert!(fake::RATES.contains(&slow));
    let bad: [(&'static [u8], u32); 5] = [
        (&ODD, RATE),
        (&[], RATE),
        (&SAMPLES, fast),
        (&SAMPLES, slow),
        (&SAMPLES, 0),
    ];
    let mut clips = vec![clip(KEY)];
    clips.extend(bad.map(|(samples, rate)| Clip {
        id: ClipId::new(9),
        rate,
        samples,
    }));
    clips.push(clip(DROP));
    for _ in 0..2 {
        mixer.prepare(clips.clone(), config(1));
    }
    let both = [Call::Warn(Warn::Bytes, None), Call::Warn(Warn::Rate, None)];
    assert_eq!(
        world.take(),
        both,
        "one warning per kind, and nothing built"
    );
    for (id, t) in [(ClipId::new(9), 0), (KEY, 1), (DROP, 2)] {
        mixer.play(id, 1.0, ms(t));
    }
    let made = [Part::Engine, Part::Master, Part::Voice(1), Part::Voice(2)];
    let mut want: Vec<Call> = made.into_iter().map(Call::Make).collect();
    want.extend([Call::Start, Call::Play(1, 1.0), Call::Play(2, 1.0)]);
    assert_eq!(world.take(), want, "the good clips play");
}

#[test]
fn a_prepare_after_a_play_tears_the_engine_down_and_the_next_play_builds_afresh() {
    let (mut mixer, world) = mixer(1);
    mixer.play(KEY, 1.0, ms(0));
    world.take();
    let longer = SpeakerConfig {
        idle_ms: 2 * IDLE_MS,
        ..config(1)
    };
    mixer.prepare(vec![clip(KEY), clip(DROP)], longer);
    // Two voices, the master and the engine go; the rest is the next build.
    let mut gone = rebuilt(1, 2);
    let made = gone.split_off(4);
    assert_eq!(
        world.take(),
        gone,
        "the voices, the master, then the engine"
    );
    assert_eq!(mixer.wait(ms(1)), None, "no engine, no timer");
    mixer.play(KEY, 1.0, ms(2));
    let mut want = made;
    want.extend([Call::Start, Call::Play(3, 1.0)]);
    assert_eq!(world.take(), want, "made afresh at the next play");
    // The clip's 10 ms, then the new idle time.
    assert_eq!(mixer.wait(ms(2)), Some(Duration::from_millis(1_010)));
}

#[test]
fn silence_stops_the_engine_once_and_the_next_play_starts_it_first() {
    let (mut mixer, world) = mixer(1);
    assert_eq!(mixer.wait(ms(0)), None, "nothing runs before a play");
    mixer.play(KEY, 1.0, ms(0));
    mixer.play(DROP, 1.0, ms(300));
    world.take();
    // The drop sound ends at 310 ms, so the engine may stop 500 ms later.
    assert_eq!(mixer.wait(ms(310)), Some(Duration::from_millis(500)));
    mixer.idle(ms(809));
    assert!(world.take().is_empty(), "not before the idle time");
    mixer.idle(ms(810));
    mixer.idle(ms(9_000));
    assert_eq!(world.take(), [Call::Stop], "once");
    assert_eq!(
        mixer.wait(ms(9_000)),
        None,
        "a stopped engine sets no timer"
    );
    world.state().busy.clear();
    mixer.play(KEY, 1.0, ms(10_000));
    assert_eq!(world.take(), [Call::Start, Call::Play(1, 1.0)]);
}

#[test]
fn a_lost_device_rebuilds_once_and_retries_once_and_a_second_loss_drops_the_play() {
    let (mut mixer, world) = mixer(1);
    mixer.play(KEY, 1.0, ms(0));
    world.take();
    world.state().plays.push_back(LOST);
    mixer.play(DROP, 0.5, ms(100));
    let mut want = vec![Call::Play(2, 0.5)];
    want.extend(rebuilt(1, 2));
    want.extend([Call::Start, Call::Play(4, 0.5)]);
    assert_eq!(world.take(), want, "rebuilt and played");
    world.state().plays.extend([LOST, LOST, LOST]);
    mixer.play(KEY, 1.0, ms(200));
    let mut want = vec![Call::Play(3, 1.0)];
    want.extend(rebuilt(3, 2));
    want.extend([Call::Start, Call::Play(5, 1.0)]);
    want.push(Call::Warn(Warn::Play, Some(-1)));
    assert_eq!(world.take(), want, "no third try");
    assert_eq!(world.state().plays.len(), 1);
}

#[test]
fn with_no_device_each_kind_warns_once_until_a_play_succeeds() {
    let (mut mixer, world) = mixer(1);
    world.state().masters.extend([NONE; 3]);
    for t in 0..3 {
        mixer.play(KEY, 1.0, ms(t));
    }
    assert_eq!(warns(world.take()), [Call::Warn(Warn::Open, Some(-2))]);
    assert_eq!(mixer.wait(ms(3)), None, "no engine, no timer");
    mixer.play(KEY, 1.0, ms(10));
    assert!(warns(world.take()).is_empty(), "the device is back");
    world.state().plays.push_back(LOST);
    world.state().masters.extend([NONE; 3]);
    for t in 20..23 {
        mixer.play(KEY, 1.0, ms(t));
    }
    let both = [
        Call::Warn(Warn::Rebuild, Some(-2)),
        Call::Warn(Warn::Open, Some(-2)),
    ];
    assert_eq!(warns(world.take()), both, "the success cleared the flags");
    assert_eq!(mixer.wait(ms(23)), None, "a failed rebuild leaves no timer");
}

#[test]
fn a_gain_out_of_range_is_clamped_and_nan_or_an_unknown_clip_plays_nothing() {
    let (mut mixer, world) = mixer(1);
    mixer.play(KEY, f32::NAN, ms(0));
    mixer.play(ClipId::new(9), 1.0, ms(0));
    assert!(world.take().is_empty(), "nothing is built");
    mixer.play(KEY, 7.0, ms(1));
    mixer.play(KEY, -7.0, ms(2));
    let plays: Vec<Call> = world
        .take()
        .into_iter()
        .filter(|c| matches!(c, Call::Play(..)))
        .collect();
    assert_eq!(plays, [Call::Play(1, 1.0), Call::Play(3, 0.0)]);
}
