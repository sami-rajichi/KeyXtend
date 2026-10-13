//! Tests of the speaker over a fake system: the mixer's rules at set times, then the thread's queue and teardown.

use std::collections::{HashSet, VecDeque};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use kx_module_api::{Clock, Mono, US_PER_MS};
use kx_platform::ui::{Clip, ClipId, Speaker, SpeakerConfig};

use super::{Actor, Fault, Mixer, Os, Warn, level, pick};

const KEY: ClipId = ClipId::new(1);
const DROP: ClipId = ClipId::new(2);
/// The fake clips' rate: 1 kHz, so their 20 bytes play for 10 ms.
const RATE: u32 = 1_000;
static SAMPLES: [u8; 20] = [0; 20];
const IDLE_MS: u32 = 500;
/// A lost device, and no device, with the fake's codes.
const LOST: Fault = Fault::Lost(-1);
const NONE: Fault = Fault::Failed(-2);
/// How long a test waits on the thread; only a failure waits this long.
const LIMIT: Duration = Duration::from_secs(10);

/// A handle the fake made.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Part {
    Engine,
    Master,
    Voice(u32),
}

/// What the fake system was asked, in order.
#[derive(Clone, Debug, PartialEq)]
enum Call {
    Make(Part),
    Gone(Part),
    Start,
    Stop,
    Play(u32, f32),
    Warn(Warn, i32),
}

#[derive(Default)]
struct State {
    calls: Vec<Call>,
    /// Faults the next mastering voices give; empty means they are made.
    masters: VecDeque<Fault>,
    /// Faults the next plays give; empty means they play.
    plays: VecDeque<Fault>,
    /// Voices still playing.
    busy: HashSet<u32>,
    /// Voices made so far, which number them.
    made: u32,
    /// Blocks the next engine: it says it entered, then waits for the test to open it.
    gate: Option<(Sender<()>, Receiver<()>)>,
}

/// The fake system, shared by the test and the speaker thread.
#[derive(Clone, Default)]
struct World(Arc<Mutex<State>>);

impl World {
    fn state(&self) -> MutexGuard<'_, State> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The calls so far, forgotten.
    fn take(&self) -> Vec<Call> {
        std::mem::take(&mut self.state().calls)
    }

    fn note(&self, call: Call) {
        self.state().calls.push(call);
    }

    fn make(&self, part: Part) -> Handle {
        self.note(Call::Make(part));
        Handle(self.clone(), part)
    }
}

/// A fake engine or voice, which notes when it is gone.
struct Handle(World, Part);

impl Handle {
    fn voice(&self) -> u32 {
        if let Part::Voice(n) = self.1 { n } else { 0 }
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        self.0.note(Call::Gone(self.1));
    }
}

impl Os for World {
    type Engine = Handle;
    type Master = Handle;
    type Voice = Handle;

    fn engine(&self) -> Result<Handle, Fault> {
        let gate = self.state().gate.take();
        if let Some((entered, open)) = gate {
            let _ = entered.send(());
            let _ = open.recv();
        }
        Ok(self.make(Part::Engine))
    }

    fn master(&self, _: &Handle) -> Result<Handle, Fault> {
        let fault = self.state().masters.pop_front();
        fault.map_or_else(|| Ok(self.make(Part::Master)), Err)
    }

    fn voice(&self, _: &Handle, _: u32) -> Result<Handle, Fault> {
        let made = {
            let mut state = self.state();
            state.made += 1;
            state.made
        };
        Ok(self.make(Part::Voice(made)))
    }

    fn start(&self, _: &Handle) -> Result<(), Fault> {
        self.note(Call::Start);
        Ok(())
    }

    fn stop(&self, _: &Handle) {
        self.note(Call::Stop);
    }

    fn play(&self, voice: &Handle, _: &'static [u8], gain: f32) -> Result<(), Fault> {
        self.note(Call::Play(voice.voice(), gain));
        let mut state = self.state();
        if let Some(fault) = state.plays.pop_front() {
            return Err(fault);
        }
        state.busy.insert(voice.voice());
        Ok(())
    }

    fn busy(&self, voice: &Handle) -> bool {
        self.state().busy.contains(&voice.voice())
    }

    fn warn(&self, warn: Warn, code: i32) {
        self.note(Call::Warn(warn, code));
    }
}

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
fn quick_plays_of_one_clip_use_each_voice_then_restart_the_earliest() {
    let (mut mixer, world) = mixer(2);
    for t in 0..3 {
        mixer.play(KEY, 1.0, ms(t));
    }
    let made = [Part::Engine, Part::Master]
        .into_iter()
        .chain((1..=4).map(Part::Voice));
    let mut want: Vec<Call> = made.map(Call::Make).collect();
    want.extend([
        Call::Start,
        Call::Play(1, 1.0),
        Call::Play(2, 1.0),
        Call::Play(1, 1.0),
    ]);
    assert_eq!(world.take(), want);
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
    assert_eq!(
        pick([(true, ms(5)), (false, ms(9)), (true, ms(1))]),
        Some(1)
    );
    assert_eq!(pick([(true, ms(5)), (true, ms(1)), (true, ms(1))]), Some(1));
    assert_eq!(pick([]), None);
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
    want.extend([Call::Start, Call::Play(5, 1.0), Call::Warn(Warn::Play, -1)]);
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
    assert_eq!(warns(world.take()), [Call::Warn(Warn::Open, -2)]);
    assert_eq!(mixer.wait(ms(3)), None, "no engine, no timer");
    mixer.play(KEY, 1.0, ms(10));
    assert!(warns(world.take()).is_empty(), "the device is back");
    world.state().plays.push_back(LOST);
    world.state().masters.extend([NONE; 3]);
    for t in 20..23 {
        mixer.play(KEY, 1.0, ms(t));
    }
    let both = [Call::Warn(Warn::Rebuild, -2), Call::Warn(Warn::Open, -2)];
    assert_eq!(warns(world.take()), both, "the success cleared the flags");
    assert_eq!(mixer.wait(ms(23)), None, "a failed rebuild leaves no timer");
}

#[test]
fn a_gain_out_of_range_is_clamped_and_nan_or_an_unknown_clip_plays_nothing() {
    assert_eq!(level(2.0), Some(1.0));
    assert_eq!(level(-0.5), Some(0.0));
    assert_eq!(level(0.25), Some(0.25));
    assert_eq!(level(f32::INFINITY), Some(1.0));
    assert_eq!(level(f32::NAN), None);
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
    assert_eq!(plays, [Call::Play(1, 1.0), Call::Play(1, 0.0)]);
}

/// A clock that never moves, so the thread never stops its engine.
struct Still;

impl Clock for Still {
    fn now(&self) -> Mono {
        Mono::default()
    }
}

fn actor(world: &World) -> Actor {
    let world = world.clone();
    Actor::spawn(move || world, Arc::new(Still))
}

#[test]
fn play_returns_while_the_thread_is_blocked() {
    let world = World::default();
    let ((entered, inside), (open, gate)) = (mpsc::channel(), mpsc::channel());
    world.state().gate = Some((entered, gate));
    let speaker = Arc::new(actor(&world));
    speaker.prepare(vec![clip(KEY)], config(1));
    speaker.play(KEY, 1.0);
    inside
        .recv_timeout(LIMIT)
        .expect("the thread blocks making the engine");
    let (caller, (done, returned)) = (Arc::clone(&speaker), mpsc::channel());
    let plays = std::thread::spawn(move || {
        (0..3).for_each(|_| caller.play(KEY, 1.0));
        let _ = done.send(());
    });
    let answer = returned.recv_timeout(LIMIT);
    let _ = open.send(());
    assert!(answer.is_ok(), "play waited on the blocked thread");
    plays.join().expect("the caller ends");
    drop(speaker);
    let calls = world
        .take()
        .into_iter()
        .filter(|c| matches!(c, Call::Play(..)));
    assert_eq!(calls.count(), 4, "every play ran once the thread was free");
}

#[test]
fn dropping_the_speaker_destroys_the_voices_then_the_master_then_the_engine() {
    let world = World::default();
    let speaker = actor(&world);
    speaker.prepare(vec![clip(KEY), clip(DROP)], config(2));
    speaker.play(KEY, 1.0);
    drop(speaker);
    let gone: Vec<Call> = world
        .take()
        .into_iter()
        .filter(|c| matches!(c, Call::Gone(_)))
        .collect();
    let parts = (1..=4).map(Part::Voice).chain([Part::Master, Part::Engine]);
    assert_eq!(gone, parts.map(Call::Gone).collect::<Vec<_>>());
}
