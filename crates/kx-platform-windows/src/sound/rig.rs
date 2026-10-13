//! The engine with its mastering voice and a voice pool per clip, built and torn down in `XAudio2`'s order.

use kx_module_api::Mono;
use kx_platform::ui::Clip;

use super::{Fault, Os, pick};

/// One source voice and when its last play started.
struct Slot<V> {
    voice: V,
    started: Mono,
}

/// One clip and its voices.
type Pool<V> = (Clip, Vec<Slot<V>>);

/// The engine and its voices. Fields drop in order: the source voices, the mastering voice, then the engine, as
/// `XAudio2` requires.
pub(super) struct Rig<O: Os> {
    /// One pool per clip, in the clips' order.
    pools: Vec<Pool<O::Voice>>,
    /// Kept for its drop: the source voices send to it.
    _master: O::Master,
    pub(super) engine: O::Engine,
    /// When the running engine may stop; `None` while it is stopped.
    pub(super) stop_at: Option<Mono>,
}

impl<O: Os> Rig<O> {
    /// The engine, its mastering voice and `voices` source voices per clip, stopped; a failure leaves nothing made.
    pub(super) fn new(os: &O, clips: &[Clip], voices: usize) -> Result<Self, Fault> {
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
            pools.push((*clip, (0..voices).map(slot).collect::<Result<_, Fault>>()?));
        }
        Ok(Self {
            pools,
            _master: master,
            engine,
            stop_at: None,
        })
    }

    /// Starts the engine if it stopped, then plays clip `at` at `gain` on an idle voice, or on a new voice in place
    /// of the one that started earliest.
    pub(super) fn sound(&mut self, os: &O, at: usize, gain: f32, now: Mono) -> Result<(), Fault> {
        if self.stop_at.is_none() {
            // After a critical error every call fails with an unnamed code, so a failed start counts as a lost device.
            os.start(&self.engine)
                .map_err(|fault| Fault::Lost(fault.code()))?;
            // Running from now, so a play that fails here lets it stop at the next turn.
            self.stop_at = Some(now);
        }
        let Some((clip, pool)) = self.pools.get_mut(at) else {
            return Ok(());
        };
        let chosen = pick(pool.iter().map(|s| (os.busy(&s.voice), s.started)));
        let Some((slot, busy)) = chosen.and_then(|(i, busy)| Some((pool.get_mut(i)?, busy))) else {
            return Ok(());
        };
        if busy {
            // A stop is not immediate, so a playing voice is replaced; dropping it destroys it, which waits.
            slot.voice = os.voice(&self.engine, clip.rate)?;
        }
        os.play(&slot.voice, clip.samples, gain)?;
        slot.started = now;
        Ok(())
    }
}
