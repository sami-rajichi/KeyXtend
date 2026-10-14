//! The health check (ADR-0027 decision 6): Windows silently removes a low-level hook that runs too long, so the host
//! compares Windows' last input with the tick when its hooks last ran a callback, and puts them back when they lag.
//! Input on another desktop, such as the secure desktop or the lock screen, never reaches the hooks, so it is not lag.

use std::time::Duration;

use kx_module_api::Mono;
use kx_module_api::ports::HookConfig;

/// When the hooks last ran a callback, and when they last went back.
#[derive(Debug)]
pub(crate) struct Health {
    /// The tick count when one of our hooks last ran a callback or our input went out; never an event's own stamp.
    seen: u32,
    /// The input desktop was another one at the last check.
    away: bool,
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
            away: false,
            fixed: None,
            period: Duration::from_millis(u64::from(cfg.health_ms)),
            slack_ms: cfg.health_slack_ms,
        }
    }

    /// One of our hooks ran a callback, or our input went out, at tick `tick`; an older tick leaves the newest.
    pub(crate) fn saw(&mut self, tick: u32) {
        if newer(tick, self.seen) > 0 {
            self.seen = tick;
        }
    }

    /// True when the hooks must go back at `now`: Windows' last input is newer than theirs by more than the slack,
    /// and none went back within the period. That input then counts as handled, as it does while the input desktop is
    /// another one (`away`) and at the first check back.
    pub(crate) fn due(&mut self, now: Mono, last_input: Option<u32>, away: bool) -> bool {
        let was = std::mem::replace(&mut self.away, away);
        let Some(last) = last_input else {
            return false;
        };
        if away || was {
            self.saw(last);
            return false;
        }
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
