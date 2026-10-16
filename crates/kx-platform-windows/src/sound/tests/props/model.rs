//! A model of what the fake system holds, built from the mixer's calls: it checks each call here and in `parts`, and
//! each step in `settle`.

mod parts;
mod settle;

use std::collections::{HashMap, VecDeque};
use std::time::Duration;

use kx_module_api::{Mono, US_PER_S};
use kx_platform::ui::{Clip, ClipId, SpeakerConfig, level};
use proptest::prelude::*;

use super::super::fake::{self, Call, Part};
use super::Queue;
use crate::sound::{Fault, Warn};

/// What a step asked of the mixer.
pub(super) enum Ask {
    Play(ClipId, f32),
    Prepare(Vec<Clip>, SpeakerConfig),
    /// The thread woke for the idle stop.
    Wake,
    /// The mixer was dropped.
    Drop,
}

/// The fake's fault queues, copied so the model knows how each call ended.
#[derive(Default)]
pub(super) struct Faults {
    starts: VecDeque<Fault>,
    masters: VecDeque<Fault>,
    plays: VecDeque<Fault>,
}

impl Faults {
    /// Queues `fault` `n` times in `queue`, as the step did in the fake.
    pub(super) fn add(&mut self, queue: Queue, fault: Fault, n: usize) {
        let queue = match queue {
            Queue::Starts => &mut self.starts,
            Queue::Masters => &mut self.masters,
            Queue::Plays => &mut self.plays,
        };
        queue.extend(std::iter::repeat_n(fault, n));
    }
}

/// A voice of a clip's pool, and when its last play started.
#[derive(Copy, Clone)]
struct Slot {
    voice: u32,
    started: Mono,
}

/// The engine the calls made.
#[derive(Default)]
struct Engine {
    /// The mastering voice is there.
    master: bool,
    /// The mastering voice was made, so an engine gone without one was a failed build.
    mastered: bool,
    /// One pool per kept clip, filled in order.
    pools: Vec<Vec<Slot>>,
    /// When it started, while it runs.
    since: Option<Mono>,
    /// When the last clip played since the start ends, plus the idle time.
    quiet: Option<Mono>,
}

impl Engine {
    /// When the running engine may stop.
    fn due(&self) -> Option<Mono> {
        self.since
            .map(|since| self.quiet.map_or(since, |q| q.max(since)))
    }

    fn voices(&self) -> impl Iterator<Item = u32> {
        self.pools.iter().flatten().map(|s| s.voice)
    }
}

/// What the current step did.
#[derive(Default)]
struct Trace {
    calls: usize,
    had_engine: bool,
    /// The kinds warned before the step.
    before: Vec<Warn>,
    /// How each try to sound the clip ended; a failed start counts as a lost device.
    tries: Vec<Result<(), Fault>>,
    /// The tries made before each engine was made.
    builds: Vec<usize>,
    /// The tries made before a build failed, and its fault.
    broke: Option<(usize, Fault)>,
    warns: Vec<(Warn, Option<i32>)>,
    /// The voice made in place of a playing one, until that one goes.
    fresh: Option<u32>,
}

/// The fake system as the calls left it, and the step being checked.
#[derive(Default)]
pub(super) struct Model {
    pub(super) faults: Faults,
    /// Plays the port takes that sounded nothing.
    pub(super) dropped: usize,
    kept: Vec<Clip>,
    voices: usize,
    idle: Duration,
    engine: Option<Engine>,
    /// When each voice that sounded ends.
    ends: HashMap<u32, Mono>,
    /// The kinds warned since a play last sounded.
    warned: Vec<Warn>,
    /// Parts made and not gone.
    alive: usize,
    now: Mono,
    /// The kept clip a play asks for and its level; `None` when it must do nothing.
    target: Option<(usize, f32)>,
    trace: Trace,
}

/// `value`, or a failed case saying `why`.
fn some<T>(value: Option<T>, why: &'static str) -> Result<T, TestCaseError> {
    value.ok_or_else(|| TestCaseError::fail(why))
}

/// True while `voice` plays at `now`.
fn plays(ends: &HashMap<u32, Mono>, voice: u32, now: Mono) -> bool {
    ends.get(&voice).is_some_and(|&end| end > now)
}

/// True if the port and the engine take `clip`.
fn fits(clip: &Clip) -> bool {
    clip.playable() && fake::RATES.contains(&clip.rate)
}

/// How long `clip` plays: its frames over its rate.
fn lasts(clip: &Clip) -> Duration {
    let frame = usize::from(Clip::CHANNELS * Clip::SAMPLE_BYTES);
    let frames = u64::try_from(clip.samples.len() / frame).unwrap_or(u64::MAX);
    Duration::from_micros(frames * u64::from(US_PER_S) / u64::from(clip.rate))
}

impl Model {
    /// True while `voice` plays at `now`.
    pub(super) fn busy(&self, voice: u32, now: Mono) -> bool {
        plays(&self.ends, voice, now)
    }

    /// How long until the engine may stop; `None` while it is stopped.
    pub(super) fn wait(&self, now: Mono) -> Option<Duration> {
        let due = self.engine.as_ref()?.due()?;
        Some(due.saturating_since(now))
    }

    /// Starts checking a step that asks `ask` at `now`.
    pub(super) fn begin(&mut self, ask: &Ask, now: Mono) {
        self.now = now;
        self.target = None;
        self.trace = Trace {
            had_engine: self.engine.is_some(),
            before: self.warned.clone(),
            ..Trace::default()
        };
        match ask {
            Ask::Play(id, gain) => {
                let at = self.kept.iter().position(|c| c.id == *id);
                self.target = at.zip(level(*gain));
            }
            Ask::Prepare(clips, config) => {
                self.kept = clips.iter().copied().filter(fits).collect();
                self.voices = config.voices;
                self.idle = Duration::from_millis(u64::from(config.idle_ms));
            }
            Ask::Wake | Ask::Drop => {}
        }
    }

    /// Checks one call of the step, then applies it.
    pub(super) fn on(&mut self, call: &Call) -> Result<(), TestCaseError> {
        self.trace.calls += 1;
        match *call {
            Call::Make(Part::Engine) => self.made_engine(),
            Call::Make(Part::Master) => self.made_master(),
            Call::Make(Part::Voice(n)) => self.made_voice(n),
            Call::Gone(Part::Voice(n)) => self.gone_voice(n),
            Call::Gone(Part::Master) => self.gone_master(),
            Call::Gone(Part::Engine) => self.gone_engine(),
            Call::Start => self.start(),
            Call::Stop => self.stop(),
            Call::Play(n, gain) => self.play(n, gain),
            Call::Warn(warn, code) => {
                prop_assert!(!self.warned.contains(&warn), "{warn:?} warned twice");
                self.warned.push(warn);
                self.trace.warns.push((warn, code));
                Ok(())
            }
        }
    }

    fn start(&mut self) -> Result<(), TestCaseError> {
        let engine = some(self.engine.as_mut(), "a start needs an engine")?;
        prop_assert!(
            engine.since.is_none(),
            "a running engine is not started again"
        );
        let built = engine.pools.iter().all(|p| p.len() == self.voices);
        prop_assert!(
            built && self.target.is_some(),
            "an engine starts for a play, once built"
        );
        match self.faults.starts.pop_front() {
            Some(fault) => self.trace.tries.push(Err(Fault::Lost(fault.code()))),
            None => engine.since = Some(self.now),
        }
        Ok(())
    }

    /// The engine stops only while it runs, once every clip ended and the idle time passed.
    fn stop(&mut self) -> Result<(), TestCaseError> {
        let engine = some(self.engine.as_mut(), "a stop needs an engine")?;
        let due = some(engine.due(), "only a running engine stops")?;
        prop_assert!(self.now >= due, "the engine stops only after the idle time");
        let playing = engine.voices().any(|v| plays(&self.ends, v, self.now));
        prop_assert!(!playing, "the engine stops only once every clip ended");
        (engine.since, engine.quiet) = (None, None);
        Ok(())
    }

    /// A play goes on a running engine, at the port's level, on the clip's first voice that is not playing.
    fn play(&mut self, n: u32, gain: f32) -> Result<(), TestCaseError> {
        let (at, level) = some(self.target, "a play that must do nothing played")?;
        let same = gain.to_bits() == level.to_bits();
        prop_assert!(same, "gain {gain} is not the port's level {level}");
        prop_assert!(
            self.trace.fresh.is_none(),
            "the replaced voice goes before the play"
        );
        let engine = some(self.engine.as_mut(), "a play needs an engine")?;
        prop_assert!(
            engine.since.is_some(),
            "a voice plays only on a running engine"
        );
        let pool = some(engine.pools.get_mut(at), "the clip has a pool")?;
        let idle = pool
            .iter_mut()
            .find(|s| !plays(&self.ends, s.voice, self.now));
        let slot = some(idle, "a play takes a voice that is not playing")?;
        prop_assert_eq!(slot.voice, n, "the clip's first idle voice plays");
        let Some(fault) = self.faults.plays.pop_front() else {
            let clip = some(self.kept.get(at), "the clip is kept")?;
            let end = self.now.saturating_add(lasts(clip));
            slot.started = self.now;
            self.ends.insert(n, end);
            engine.quiet = engine.quiet.max(Some(end.saturating_add(self.idle)));
            self.warned.clear();
            self.trace.tries.push(Ok(()));
            return Ok(());
        };
        self.trace.tries.push(Err(fault));
        Ok(())
    }
}
