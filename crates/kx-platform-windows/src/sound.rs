//! The speaker: a thread of its own owns the audio engine, so `play` only sends a message. The voice choice, the idle
//! stop, the rebuild-once policy and the gain clamp are pure and tested everywhere; `win` calls `XAudio2`.
#![cfg_attr(
    not(windows),
    allow(
        dead_code,
        reason = "Only the Windows speaker runs this logic; tests cover it everywhere."
    )
)]

#[cfg(test)]
mod tests;
#[cfg(windows)]
mod win;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use kx_module_api::{Clock, Mono, US_PER_S};
use kx_platform::ui::{Clip, ClipId, Speaker, SpeakerConfig};

#[cfg(windows)]
pub use win::WinSpeaker;

/// The speaker thread's name.
const THREAD: &str = "kx-speaker";
/// The lowest and highest gain a play takes.
const SILENT: f32 = 0.0;
const FULL: f32 = 1.0;

/// A failed audio call, with the system's error code for the log.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Fault {
    /// The device was lost or changed, so a rebuild may help.
    Lost(i32),
    /// Any other failure.
    Failed(i32),
}

impl Fault {
    /// The system's error code.
    fn code(self) -> i32 {
        match self {
            Self::Lost(code) | Self::Failed(code) => code,
        }
    }
}

/// What went wrong; each kind warns once until a play succeeds.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Warn {
    /// The engine or its voices could not be made, as with no audio device.
    Open,
    /// The rebuild after a lost device failed.
    Rebuild,
    /// A play failed and was dropped.
    Play,
}

/// What the speaker thread asks of the system: `XAudio2` and the log on Windows, a recorder in tests. Each handle
/// undoes itself when dropped.
pub(crate) trait Os {
    /// The engine; dropping it releases the engine.
    type Engine;
    /// The mastering voice; dropping it destroys the voice.
    type Master;
    /// A source voice; dropping it destroys the voice.
    type Voice;

    /// Makes the engine.
    fn engine(&self) -> Result<Self::Engine, Fault>;

    /// Makes the mastering voice on the default device.
    fn master(&self, engine: &Self::Engine) -> Result<Self::Master, Fault>;

    /// Makes a source voice for the clips' format at `rate`.
    fn voice(&self, engine: &Self::Engine, rate: u32) -> Result<Self::Voice, Fault>;

    /// Starts audio processing.
    fn start(&self, engine: &Self::Engine) -> Result<(), Fault>;

    /// Stops audio processing, so the idle engine costs no processor time.
    fn stop(&self, engine: &Self::Engine);

    /// Restarts `voice` on `samples` at `gain`, cutting what it played.
    fn play(&self, voice: &Self::Voice, samples: &'static [u8], gain: f32) -> Result<(), Fault>;

    /// True while `voice` still has samples queued.
    fn busy(&self, voice: &Self::Voice) -> bool;

    /// Logs `warn` with the system's error `code`.
    fn warn(&self, warn: Warn, code: i32);
}

/// How many kinds of `Warn` there are: the last one's index plus one.
const WARNS: usize = Warn::Play as usize + 1;

/// One source voice and when its last play started.
struct Slot<V> {
    voice: V,
    started: Mono,
}

/// One clip's samples and its voices.
type Pool<V> = (&'static [u8], Vec<Slot<V>>);

/// The engine and its voices. Fields drop in order: the source voices, the mastering voice, then the engine, as
/// `XAudio2` requires.
struct Rig<O: Os> {
    /// One pool per clip, in the clips' order.
    pools: Vec<Pool<O::Voice>>,
    /// Kept for its drop: the source voices send to it.
    _master: O::Master,
    engine: O::Engine,
    /// When the running engine may stop; `None` while it is stopped.
    stop_at: Option<Mono>,
}

impl<O: Os> Rig<O> {
    /// The engine, its mastering voice and `voices` source voices per clip, stopped; a failure leaves nothing made.
    fn new(os: &O, clips: &[Clip], voices: usize) -> Result<Self, Fault> {
        // Locals drop in reverse, so a failure gives back the voices made, then the master, then the engine.
        let engine = os.engine()?;
        let master = os.master(&engine)?;
        let mut pools = Vec::with_capacity(clips.len());
        for clip in clips {
            let slot = |_| {
                let voice = os.voice(&engine, clip.rate)?;
                Ok(Slot {
                    voice,
                    started: Mono::default(),
                })
            };
            pools.push((
                clip.samples,
                (0..voices).map(slot).collect::<Result<_, Fault>>()?,
            ));
        }
        Ok(Self {
            pools,
            _master: master,
            engine,
            stop_at: None,
        })
    }

    /// Starts the engine if it stopped, then restarts the chosen voice of pool `at` at `gain`.
    fn sound(&mut self, os: &O, at: usize, gain: f32, now: Mono) -> Result<(), Fault> {
        if self.stop_at.is_none() {
            os.start(&self.engine)?;
            // Running from now, so a play that fails here lets it stop at the next turn.
            self.stop_at = Some(now);
        }
        let Some((samples, pool)) = self.pools.get_mut(at) else {
            return Ok(());
        };
        let chosen = pick(pool.iter().map(|s| (os.busy(&s.voice), s.started)));
        let Some(slot) = chosen.and_then(|i| pool.get_mut(i)) else {
            return Ok(());
        };
        os.play(&slot.voice, samples, gain)?;
        slot.started = now;
        Ok(())
    }
}

/// The speaker thread's state: the clips, the engine once a play built it, and the warnings given.
pub(crate) struct Mixer<O: Os> {
    /// Declared before `os`, so it drops first: on Windows, COM must outlive the engine.
    rig: Option<Rig<O>>,
    os: O,
    clips: Vec<Clip>,
    voices: usize,
    idle: Duration,
    /// One flag per `Warn` kind, so each warns once until a play succeeds (the hub's `refused` pattern).
    warned: [AtomicBool; WARNS],
}

impl<O: Os> Mixer<O> {
    /// A mixer with no clip, which builds nothing until a play.
    pub(crate) fn new(os: O) -> Self {
        Self {
            rig: None,
            os,
            clips: Vec::new(),
            voices: 0,
            idle: Duration::ZERO,
            warned: Default::default(),
        }
    }

    /// Keeps `clips` and `config`; the old engine goes, and the next play builds one for them.
    pub(crate) fn prepare(&mut self, clips: Vec<Clip>, config: SpeakerConfig) {
        self.rig = None;
        self.clips = clips;
        self.voices = config.voices;
        self.idle = Duration::from_millis(u64::from(config.idle_ms));
    }

    /// Plays clip `id` at `gain`. A lost device rebuilds the engine and retries once; a second failure drops the play.
    pub(crate) fn play(&mut self, id: ClipId, gain: f32, now: Mono) {
        let (Some(gain), Some(at)) = (level(gain), self.clips.iter().position(|c| c.id == id))
        else {
            return;
        };
        if self.rig.is_none() {
            self.build(Warn::Open);
        }
        let Some(rig) = self.rig.as_mut() else {
            return;
        };
        let mut tried = rig.sound(&self.os, at, gain, now);
        if matches!(tried, Err(Fault::Lost(_))) {
            self.build(Warn::Rebuild);
            let Some(rig) = self.rig.as_mut() else {
                return;
            };
            tried = rig.sound(&self.os, at, gain, now);
        }
        match tried {
            Ok(()) => self.played(at, now),
            Err(fault) => self.fail(Warn::Play, fault),
        }
    }

    /// How long the thread may wait before `idle` is due; `None` while the engine is stopped.
    pub(crate) fn wait(&self, now: Mono) -> Option<Duration> {
        let stop_at = self.rig.as_ref()?.stop_at?;
        Some(stop_at.saturating_since(now))
    }

    /// Stops the engine once the idle time passed after the last sound's end.
    pub(crate) fn idle(&mut self, now: Mono) {
        let Some(rig) = self.rig.as_mut() else {
            return;
        };
        if rig.stop_at.is_some_and(|at| now >= at) {
            self.os.stop(&rig.engine);
            rig.stop_at = None;
        }
    }

    /// Makes the engine afresh once the old one's voices and engine are gone; a failure warns `warn` and leaves none.
    fn build(&mut self, warn: Warn) {
        self.rig = None;
        match Rig::new(&self.os, &self.clips, self.voices) {
            Ok(rig) => self.rig = Some(rig),
            Err(fault) => self.fail(warn, fault),
        }
    }

    /// Notes clip `at` playing from `now`: the warnings clear, and the engine may stop once it ended and stayed idle.
    fn played(&mut self, at: usize, now: Mono) {
        for flag in &self.warned {
            flag.store(false, Ordering::Relaxed);
        }
        let quiet = now.saturating_add(length(&self.clips[at]).saturating_add(self.idle));
        if let Some(rig) = self.rig.as_mut() {
            // `None` sorts first, so this keeps the later of the two stops.
            rig.stop_at = rig.stop_at.max(Some(quiet));
        }
    }

    /// Logs `warn` with `fault`'s code, once until a play succeeds.
    fn fail(&self, warn: Warn, fault: Fault) {
        if !self.warned[warn as usize].swap(true, Ordering::Relaxed) {
            self.os.warn(warn, fault.code());
        }
    }
}

/// `gain` within 0.0 and 1.0; `None` for NaN, which plays nothing.
pub(crate) fn level(gain: f32) -> Option<f32> {
    (!gain.is_nan()).then(|| gain.clamp(SILENT, FULL))
}

/// The voice a new play takes: the first idle one, else the one that started earliest; `None` with no voice.
pub(crate) fn pick(voices: impl IntoIterator<Item = (bool, Mono)>) -> Option<usize> {
    let mut earliest: Option<(usize, Mono)> = None;
    for (i, (busy, started)) in voices.into_iter().enumerate() {
        if !busy {
            return Some(i);
        }
        if earliest.is_none_or(|(_, first)| started < first) {
            earliest = Some((i, started));
        }
    }
    earliest.map(|(i, _)| i)
}

/// How long `clip` plays.
fn length(clip: &Clip) -> Duration {
    let frames = clip.samples.len() / usize::from(Clip::CHANNELS * Clip::SAMPLE_BYTES);
    let us = u64::try_from(frames).map_or(u64::MAX, |f| f.saturating_mul(u64::from(US_PER_S)));
    us.checked_div(u64::from(clip.rate))
        .map_or(Duration::ZERO, Duration::from_micros)
}

/// A request to the speaker thread.
enum Ask {
    Prepare(Vec<Clip>, SpeakerConfig),
    Play(ClipId, f32),
}

/// The speaker thread and its queue, so `Speaker` calls only send. Dropping it closes the queue and joins the thread,
/// which drops the voices, then the engine.
#[derive(Debug)]
pub(crate) struct Actor(Option<(Sender<Ask>, JoinHandle<()>)>);

impl Actor {
    /// Starts the thread, which makes its system with `make` and reads the time from `clock`; without it nothing plays.
    pub(crate) fn spawn<O: Os + 'static>(
        make: impl FnOnce() -> O + Send + 'static,
        clock: Arc<dyn Clock>,
    ) -> Self {
        let (asks, queue) = mpsc::channel();
        let spawned = thread::Builder::new()
            .name(THREAD.to_owned())
            .spawn(move || run(make(), &queue, clock.as_ref()))
            .inspect_err(
                |e| tracing::warn!(error = %e, "no sound: the speaker thread did not start"),
            );
        Self(spawned.ok().map(|join| (asks, join)))
    }

    /// Queues `ask`; a thread that ended drops it, and the call still returns at once.
    fn send(&self, ask: Ask) {
        if let Some((asks, _)) = &self.0 {
            let _ = asks.send(ask);
        }
    }
}

impl Speaker for Actor {
    fn prepare(&self, clips: Vec<Clip>, config: SpeakerConfig) {
        self.send(Ask::Prepare(clips, config));
    }

    fn play(&self, id: ClipId, gain: f32) {
        self.send(Ask::Play(id, gain));
    }
}

impl Drop for Actor {
    fn drop(&mut self) {
        if let Some((asks, join)) = self.0.take() {
            drop(asks);
            let _ = join.join();
        }
    }
}

/// The thread: it serves asks and stops the idle engine until the queue closes, then drops the voices and the engine.
fn run<O: Os>(os: O, queue: &Receiver<Ask>, clock: &dyn Clock) {
    let mut mixer = Mixer::new(os);
    loop {
        let ask = match mixer.wait(clock.now()) {
            Some(wait) => queue.recv_timeout(wait),
            None => queue.recv().map_err(|_| RecvTimeoutError::Disconnected),
        };
        match ask {
            Ok(Ask::Prepare(clips, config)) => mixer.prepare(clips, config),
            Ok(Ask::Play(id, gain)) => mixer.play(id, gain, clock.now()),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }
        mixer.idle(clock.now());
    }
}
