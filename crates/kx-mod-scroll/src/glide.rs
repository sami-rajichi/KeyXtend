//! The glide: the lines owed at each tick are the speed times the time since the last reckoning, with each gap
//! capped at `check_ms` so a late tick or a sleep never jumps.

use std::time::Duration;

use crate::ScrollSettings;
use kx_module_api::Mono;

/// The glide's settings, from `[scroll]`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct GlideCfg {
    /// The slow speed, in lines a second.
    pub slow: u32,
    /// The fast speed, in lines a second.
    pub fast: u32,
    /// The longest gap one reckoning counts: `check_ms`.
    pub cap: Duration,
}

impl From<&ScrollSettings> for GlideCfg {
    fn from(s: &ScrollSettings) -> Self {
        Self {
            slow: s.slow,
            fast: s.fast,
            cap: Duration::from_millis(u64::from(s.check_ms)),
        }
    }
}

impl GlideCfg {
    /// The speed in lines a second, fast or slow.
    #[must_use]
    pub const fn speed(&self, fast: bool) -> u32 {
        if fast { self.fast } else { self.slow }
    }
}

/// One scroll's glide: its speed and where its reckoning stands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Glide {
    /// The speeds and the cap.
    cfg: GlideCfg,
    /// The speed now, in lines a second.
    speed: u32,
    /// The last reckoning; it never moves back.
    last: Mono,
}

impl Glide {
    /// A glide from `now`, fast or slow, with nothing owed yet.
    #[must_use]
    pub const fn start(cfg: GlideCfg, fast: bool, now: Mono) -> Self {
        Self {
            cfg,
            speed: cfg.speed(fast),
            last: now,
        }
    }

    /// The lines owed since the last reckoning, which moves to `now`; always finite and at least 0. A time before
    /// the last reckoning owes nothing and leaves it where it is.
    #[must_use]
    pub fn tick(&mut self, now: Mono) -> f64 {
        let gap = now.saturating_since(self.last).min(self.cfg.cap);
        self.last = self.last.max(now);
        f64::from(self.speed) * gap.as_secs_f64()
    }

    /// Settles the lines owed so far at the old speed, then reckons afresh from `now`, fast or slow. The actor turns
    /// them the old way or drops them, so no time is lost or counted twice.
    #[must_use]
    pub fn restart(&mut self, fast: bool, now: Mono) -> f64 {
        let owed = self.tick(now);
        self.speed = self.cfg.speed(fast);
        owed
    }
}

#[cfg(test)]
mod props;
#[cfg(test)]
mod tests;
