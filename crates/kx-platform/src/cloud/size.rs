//! A speech-net job's size and times: its memory cap from its run's sound, its requests in flight and their replies,
//! and how long a piece may take once the one before it is told.

use crate::worker::{Launch, NEVER_IDLE_S, Times};
use kx_ipc::{
    MAX_NET_REPLY_BYTES, MIB, NET_JOB, NET_TRY_WAITS, NET_UPLOAD_TIMES, SOUND_BYTES_PER_S,
};
use kx_module_api::MS_PER_S;
use kx_module_api::ports::JobLimits;
use std::time::Duration;

/// The room a run needs past `job_mb`, in MB: its `sound`, then `at_once` requests in flight, each a body of its
/// longest `piece` and a reply at `MAX_NET_REPLY_BYTES`.
pub(super) fn run_mb(sound: usize, at_once: u8, piece: usize) -> u32 {
    let one = piece.saturating_add(MAX_NET_REPLY_BYTES);
    let bytes = sound.saturating_add(one.saturating_mul(at_once.into()));
    u32::try_from(bytes.div_ceil(MIB)).unwrap_or(u32::MAX)
}

/// The speech-net job capped at `job_mb` and `room_mb`.
pub(super) fn launch(job: &JobLimits, room_mb: u32) -> Launch {
    Launch {
        args: vec![NET_JOB.to_owned()],
        memory_mb: job.job_mb.saturating_add(room_mb),
    }
}

/// A run's job's times: `job`'s, but it never idles out.
pub(super) const fn times(job: &JobLimits) -> Times {
    Times {
        deadline_ms: job.ask_ms,
        idle_s: NEVER_IDLE_S,
        restart_ms: job.restart_ms,
        restart_max_ms: job.restart_max_ms,
    }
}

/// How long a piece of `bytes` may take once the one before it is told: each of its `tries` waits `pace_s` for its
/// turn, then `NET_TRY_WAITS` of `wait_s` and an upload of `NET_UPLOAD_TIMES` its length; a 429 adds one `wait_s`.
pub(super) fn bound(bytes: usize, wait_s: u16, tries: u8, pace_s: u16) -> Duration {
    let wait = Duration::from_secs(wait_s.into());
    let (bytes, per_s) = (u64::try_from(bytes), u64::try_from(SOUND_BYTES_PER_S));
    let (bytes, per_s) = (bytes.unwrap_or(u64::MAX), per_s.unwrap_or(u64::MAX));
    let sound = Duration::from_millis(bytes.saturating_mul(MS_PER_S.into()) / per_s);
    let pace = Duration::from_secs(pace_s.into());
    let one = pace
        .saturating_add(wait.saturating_mul(NET_TRY_WAITS))
        .saturating_add(sound.saturating_mul(NET_UPLOAD_TIMES));
    one.saturating_mul(tries.into()).saturating_add(wait)
}
