//! The health check (ADR-0027 decision 6): Windows silently removes a low-level hook that runs too long, so the host
//! compares Windows' last input with the newest event its hooks saw, and puts the hooks back when they lag.

use std::time::Duration;

use kx_module_api::Mono;
use kx_module_api::ports::HookConfig;

/// When the hooks last saw input, and when they last went back.
#[derive(Debug)]
pub(crate) struct Health {
    /// The tick time of the newest event any of our hooks saw.
    seen: u32,
    /// When the hooks last went back.
    fixed: Option<Mono>,
    /// The check's period.
    period: Duration,
    /// How far, in ms, Windows' last input may be newer than `seen`.
    slack_ms: u32,
}

impl Health {
    /// A check as `cfg` says, counting the hooks current up to tick `seen`.
    pub(crate) fn new(cfg: &HookConfig, seen: u32) -> Self {
        Self {
            seen,
            fixed: None,
            period: Duration::from_millis(u64::from(cfg.health_ms)),
            slack_ms: cfg.health_slack_ms,
        }
    }

    /// One of our hooks saw an event stamped `time`; an older stamp leaves the newest.
    pub(crate) fn saw(&mut self, time: u32) {
        if newer(time, self.seen) > 0 {
            self.seen = time;
        }
    }

    /// True when the hooks must go back at `now`: Windows' last input is newer than theirs by more than the slack,
    /// and none went back within the period. That input then counts as handled.
    pub(crate) fn due(&mut self, now: Mono, last_input: Option<u32>) -> bool {
        let Some(last) = last_input else {
            return false;
        };
        let lag = newer(last, self.seen);
        let late = lag > 0 && lag.unsigned_abs() > self.slack_ms;
        let rested = self
            .fixed
            .is_none_or(|t| now.saturating_since(t) >= self.period);
        if !(late && rested) {
            return false;
        }
        self.fixed = Some(now);
        self.seen = last;
        true
    }
}

/// How many ms tick `a` is after tick `b`, across the tick count's wrap; negative when it is before.
fn newer(a: u32, b: u32) -> i32 {
    a.wrapping_sub(b).cast_signed()
}
