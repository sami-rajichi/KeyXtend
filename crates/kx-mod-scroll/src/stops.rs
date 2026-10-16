//! When scrolling stops, and why (spec §5.2): a tap that cannot scroll, the end, a press, the limit, or a lost
//! target. Pure: the actor reads the target and the time, and turns each `Halt` into a notice or a quiet stop.

use std::time::Duration;

use crate::ScrollSettings;
use crate::settings::NO_LIMIT;
use kx_module_api::Mono;
use kx_module_api::ports::{Reach, ScrollError};

/// Why scrolling stops, or never starts.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Halt {
    /// No point is known yet: the pointer never left the keyboard ("point first").
    NoPoint,
    /// Nothing there moves either way, or the window there closed ("nothing to scroll there").
    Nothing,
    /// The target is already at its end that way ("already at the end").
    AtEnd,
    /// The target reached its end while scrolling.
    End,
    /// A press outside our windows.
    Press,
    /// The limit after the last pad tap passed.
    Limit,
    /// The target closed, or was lost, while scrolling.
    Gone,
}

impl Halt {
    /// The user gets a notice: a tap that cannot scroll says why, and every other stop is quiet.
    #[must_use]
    pub const fn told(self) -> bool {
        matches!(self, Self::NoPoint | Self::Nothing | Self::AtEnd)
    }

    /// Why a tap that sets a new way cannot start, when the aim failed; `Refused` is no reason, the actor tries again.
    #[must_use]
    pub const fn unaimed(err: ScrollError) -> Option<Self> {
        match err {
            ScrollError::NoTarget => Some(Self::NoPoint),
            ScrollError::Gone => Some(Self::Nothing),
            ScrollError::Refused => None,
        }
    }

    /// Why a tap that sets a new way cannot start, from the target's reach `ahead` and, read only when that is the
    /// end, `back` the other way; `Unknown` counts as able to move.
    pub fn blocked(ahead: Reach, back: impl FnOnce() -> Reach) -> Option<Self> {
        if ahead != Reach::End {
            return None;
        }
        Some(if back() == Reach::End {
            Self::Nothing
        } else {
            Self::AtEnd
        })
    }

    /// Why a failed aim or turn ends a running scroll. `Refused` never does: the actor tries again next tick and drops
    /// that tick's lines, so nothing piles up, and the limit ends a scroll that stays refused.
    #[must_use]
    pub const fn failed(err: ScrollError) -> Option<Self> {
        match err {
            ScrollError::NoTarget | ScrollError::Gone => Some(Self::Gone),
            ScrollError::Refused => None,
        }
    }
}

/// The stop rules of a running scroll, from `[scroll]`; the actor calls `tapped` at every pad tap.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Stops {
    /// Stop when the target reaches its end.
    at_end: bool,
    /// Stop at a press outside our windows.
    on_press: bool,
    /// How long after the last pad tap scrolling stops; none for no limit.
    limit: Option<Duration>,
    /// The last pad tap.
    last_tap: Mono,
}

impl From<&ScrollSettings> for Stops {
    fn from(s: &ScrollSettings) -> Self {
        let limit = (s.limit_s != NO_LIMIT).then(|| Duration::from_secs(u64::from(s.limit_s)));
        Self {
            at_end: s.stop_at_end,
            on_press: s.stop_on_press,
            limit,
            last_tap: Mono::default(),
        }
    }
}

impl Stops {
    /// A pad tap at `now` restarts the limit's count.
    pub fn tapped(&mut self, now: Mono) {
        self.last_tap = now;
    }

    /// Whether reaching `reach` ends the scroll: only the end, and only with `stop_at_end` on.
    #[must_use]
    pub const fn reached(&self, reach: Reach) -> Option<Halt> {
        match reach {
            Reach::End if self.at_end => Some(Halt::End),
            _ => None,
        }
    }

    /// Whether a press ends the scroll: only one outside our windows, and only with `stop_on_press` on.
    #[must_use]
    pub const fn pressed(&self, ours: bool) -> Option<Halt> {
        if self.on_press && !ours {
            Some(Halt::Press)
        } else {
            None
        }
    }

    /// Whether the limit has passed at `now`, counted from the last pad tap; never with no limit.
    #[must_use]
    pub fn timed_out(&self, now: Mono) -> Option<Halt> {
        let since = now.saturating_since(self.last_tap);
        self.limit
            .filter(|&limit| since >= limit)
            .map(|_| Halt::Limit)
    }
}

#[cfg(test)]
mod props;
#[cfg(test)]
mod tests;
