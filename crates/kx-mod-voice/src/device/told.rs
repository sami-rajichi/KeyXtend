//! The device lines, at debug level: why a dictation runs on the processor although a card is picked, and how a card's
//! check ended. They hold kinds and numbers only, never text, sound, a key or a path.

use kx_module_api::ports::{CheckDone, SpeechError, Verdict};

use super::Why;

/// The field naming why the processor serves.
pub const WHY: &str = "why";
/// The field naming a check's verdict.
pub const VERDICT: &str = "verdict";
/// The field holding what a card must have free for the model, in MB.
pub const NEED_MB: &str = "need_mb";
/// The field holding a card's free memory, in MB.
pub const FREE_MB: &str = "free_mb";
/// The field holding the card's time on the check's clip, in ms.
pub const CARD_MS: &str = "card_ms";
/// The field holding the processor's time on the check's clip, in ms.
pub const CPU_MS: &str = "cpu_ms";
/// The field naming a check's error.
pub const ERROR: &str = "error";
/// The line as a dictation runs on the processor although a card is picked.
pub const CPU_SERVES: &str = "a dictation runs on the processor although a card is picked";
/// The line as a card's check ends.
pub const CHECK_ENDED: &str = "a card's check ended";

/// A verdict's plain name, which a log line may hold; never a number it carries.
#[must_use]
pub const fn verdict_kind(verdict: Verdict) -> &'static str {
    match verdict {
        Verdict::Passed { .. } => "passed",
        Verdict::Slow { .. } => "slow",
        Verdict::Differs { .. } => "differs",
        Verdict::NoRoom => "no_room",
        Verdict::Failed => "failed",
    }
}

/// Logs why the processor serves a picked card: its kind, with a failed check's verdict or the memory it lacks.
pub fn cpu(why: Why) {
    let kind = why.kind();
    match why {
        Why::Failed(verdict) => {
            let verdict = verdict_kind(verdict);
            tracing::debug!({ WHY } = kind, { VERDICT } = verdict, "{CPU_SERVES}");
        }
        Why::NoRoom { need_mb, free_mb } => {
            tracing::debug!(
                { WHY } = kind,
                { NEED_MB } = need_mb,
                { FREE_MB } = free_mb,
                "{CPU_SERVES}"
            );
        }
        Why::Picked | Why::NoneChecked | Why::Unchecked | Why::Unread | Why::Gone => {
            tracing::debug!({ WHY } = kind, "{CPU_SERVES}");
        }
    }
}

/// Logs a check that ended with `verdict`, with each device's time on the clip.
pub fn checked(verdict: Verdict, done: &CheckDone) {
    let (verdict, card_ms, cpu_ms) = (verdict_kind(verdict), done.card_ms, done.cpu_ms);
    tracing::debug!(
        { VERDICT } = verdict,
        { CARD_MS } = card_ms,
        { CPU_MS } = cpu_ms,
        "{CHECK_ENDED}"
    );
}

/// Logs a check that ended with `error`, by its kind.
pub fn failed(error: SpeechError) {
    tracing::debug!({ ERROR } = error.kind(), "{CHECK_ENDED}");
}
