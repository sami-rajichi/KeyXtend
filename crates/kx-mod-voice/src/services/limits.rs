//! A row's limits (ruling 2026-11-17): the frames' ranges, and a largest piece, by size and by time, that leaves
//! `room_kb` for the WAV header and the form's other parts under the service's own cap, grown by a third when the shape
//! sends Base64.

use kx_module_api::ports::{CloudLimits, MB, SOUND_BYTES_PER_S, SpeechAsk};
use serde::Deserialize;

use super::ServiceError;

/// Bytes in a megabyte as a service's cap reads: the smaller, decimal one, so a piece never passes it.
const CAP_MB: u64 = 1_000_000;
/// Base64 writes `B64_OUT` characters for each `B64_IN` bytes.
const B64_IN: u64 = 3;
/// See `B64_IN`.
const B64_OUT: u64 = 4;

/// The limits as written.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawLimits {
    /// The service's own cap on a request, in megabytes.
    cap_mb: u32,
    /// The service's own cap on a request's sound, in seconds, when it names one.
    #[serde(default)]
    cap_s: Option<u32>,
    /// A piece's most MiB of raw sound.
    max_mb: u32,
    /// A piece's most seconds.
    max_s: u16,
    /// Requests a minute.
    per_min: u16,
    /// Requests at once.
    at_once: u8,
}

/// The row's limits: each within the frames' range, with a piece that leaves room under the service's cap.
pub(super) fn checked(
    raw: &RawLimits,
    base64: bool,
    room: u64,
) -> Result<CloudLimits, ServiceError> {
    let limits = CloudLimits {
        max_mb: raw.max_mb,
        max_s: raw.max_s,
        per_min: raw.per_min,
        at_once: raw.at_once,
    };
    let ranged = CloudLimits::MAX_MB.contains(&limits.max_mb)
        && CloudLimits::MAX_S.contains(&limits.max_s)
        && CloudLimits::PER_MIN.contains(&limits.per_min)
        && CloudLimits::AT_ONCE.contains(&limits.at_once);
    if !ranged || raw.cap_mb == 0 {
        return Err(ServiceError::Limits);
    }
    let cap = u64::from(raw.cap_mb) * CAP_MB;
    let fits = |bytes: u64| grown(bytes, base64).saturating_add(room) <= cap;
    let per_s = SOUND_BYTES_PER_S as u64;
    let by_size = u64::from(raw.max_mb) * MB;
    let by_time = u64::from(raw.max_s) * per_s;
    let timed = raw.cap_s.is_none_or(|s| u32::from(raw.max_s) <= s);
    if fits(by_size) && fits(by_time) && timed {
        Ok(limits)
    } else {
        Err(ServiceError::Room)
    }
}

/// `bytes` as a request carries them: a third larger in Base64.
fn grown(bytes: u64, base64: bool) -> u64 {
    if base64 {
        bytes.div_ceil(B64_IN) * B64_OUT
    } else {
        bytes
    }
}

/// The longest piece a listen-only run cuts for a row's `limits`: the whole seconds its size and time caps both allow
/// (`piece_bytes`), within `SpeechAsk::PIECE_S`.
#[must_use]
pub fn piece_s(limits: &CloudLimits) -> u8 {
    let (low, high) = (*SpeechAsk::PIECE_S.start(), *SpeechAsk::PIECE_S.end());
    let secs = limits.piece_bytes() / SOUND_BYTES_PER_S;
    u8::try_from(secs).map_or(high, |s| s.clamp(low, high))
}
