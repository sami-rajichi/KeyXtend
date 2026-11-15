//! A voice job's shape: the model it serves, its memory cap and times, which together say when a run needs another
//! job, and the waits each ask is watched with, all from the ask's own limits.

use crate::worker::{Launch, Spawner, Split, Stream, Times, VoiceFrames};
use kx_ipc::{MAX_RECORD_BYTES, READ_TIMES, SAMPLE_BYTES, SOUND_HZ, VOICE_JOB};
use kx_module_api::Redacted;
use kx_module_api::ports::{JobLimits, S_PER_MIN, SpeechAsk};
use std::path::PathBuf;
use std::time::Duration;

/// Bytes in one MB of a job's cap.
const MIB: u64 = 1 << 20;
/// A check reads the clip twice, on the card and then on the processor, each after its own load.
const CHECK_RUNS: u32 = 2;

/// The dictation job and the key it was started for; neither until the first run.
pub(super) struct Live<S: Spawner<VoiceFrames, Worker: Split<VoiceFrames>>> {
    pub(super) stream: Option<Stream<S, VoiceFrames>>,
    pub(super) key: Option<Key>,
}

impl<S: Spawner<VoiceFrames, Worker: Split<VoiceFrames>>> Default for Live<S> {
    fn default() -> Self {
        Self {
            stream: None,
            key: None,
        }
    }
}

/// What makes two runs share a job: the same model file and runner, the same cap and the same times; its path never
/// prints.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Key {
    model: Option<(Redacted<PathBuf>, String)>,
    memory_mb: u32,
    times: Times,
}

impl Key {
    /// The job `ask` needs: capped by its model's memory, `job_mb` and the room of its longest recording.
    pub(super) fn of(ask: &SpeechAsk) -> Self {
        let model = ask.model.as_ref();
        let need = model.map_or(0, |m| m.cpu_mb);
        let room = room_mb(record_bytes(ask.record_max_s));
        Self {
            model: model.map(|m| (m.file.clone(), m.runner.clone())),
            memory_mb: ask.job.job_mb.saturating_add(need).saturating_add(room),
            times: times(&ask.job),
        }
    }

    /// The command line and cap that start this job, with `extra_mb` more room.
    pub(super) fn launch(&self, extra_mb: u32) -> Launch {
        voice(self.memory_mb.saturating_add(extra_mb))
    }

    /// The job's times.
    pub(super) const fn times(&self) -> Times {
        self.times
    }
}

/// The voice job capped at `memory_mb`.
pub(super) fn voice(memory_mb: u32) -> Launch {
    Launch {
        args: vec![VOICE_JOB.to_owned()],
        memory_mb,
    }
}

/// A one-ask job's launch: `job_mb` and `need_mb`, a model's memory or a file's size.
pub(super) fn single(job: &JobLimits, need_mb: u32) -> Launch {
    voice(job.job_mb.saturating_add(need_mb))
}

/// The room a recording of `bytes` holds in a job while its buffer grows, in MB.
pub(super) fn room_mb(bytes: usize) -> u32 {
    let held = u64::try_from(bytes.saturating_mul(READ_TIMES)).unwrap_or(u64::MAX);
    u32::try_from(held.div_ceil(MIB)).unwrap_or(u32::MAX)
}

/// The more room a handed-in recording of `bytes` needs past the room its new ask's `record_max_s` gives, in MB.
pub(super) fn extra_mb(bytes: usize, record_max_s: u16) -> u32 {
    room_mb(bytes).saturating_sub(room_mb(record_bytes(record_max_s)))
}

/// The bytes of a recording `seconds` long as hand-over sound, never past the longest the frames allow.
fn record_bytes(seconds: u16) -> usize {
    (usize::from(seconds) * SOUND_HZ * SAMPLE_BYTES).min(MAX_RECORD_BYTES)
}

/// The host's times from `job`.
pub(super) fn times(job: &JobLimits) -> Times {
    let per_min = u32::try_from(S_PER_MIN).unwrap_or(u32::MAX);
    Times {
        deadline_ms: job.ask_ms,
        idle_s: u32::from(job.idle_min).saturating_mul(per_min),
        restart_ms: job.restart_ms,
        restart_max_ms: job.restart_max_ms,
    }
}

/// How long each ask may wait for the worker, from a job's limits.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) struct Waits {
    /// An answer to a plain ask: the microphone opening, a Cancel, a Close or each kept piece.
    pub(super) ask: Duration,
    /// Each note while the job writes: a load, a warm-up and a piece in the worst case.
    pub(super) write: Duration,
    /// A device list or a file's facts.
    pub(super) read: Duration,
    /// Each note of a card's check: two loads and two reads of the clip beside the warm-up.
    pub(super) check: Duration,
}

/// The waits `job` allows.
pub(super) fn waits(job: &JobLimits) -> Waits {
    let s = |n: u16| Duration::from_secs(u64::from(n));
    let (load, warm, piece) = (s(job.load_s), s(job.warm_s), s(job.piece_wait_s));
    Waits {
        ask: Duration::from_millis(u64::from(job.ask_ms)),
        write: load + warm + piece,
        read: load,
        check: (load + piece) * CHECK_RUNS + warm,
    }
}
