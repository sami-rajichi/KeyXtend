//! Whether the guard's hook still runs: Windows may remove a low-level hook without a word, so kx-gates' own clicks,
//! which every live hook counts, must be counted within `alive_ms`.

use std::time::Duration;

use kx_module_api::Mono;

use super::MouseGuard;

/// Clicks kx-gates sent: when, and how many own events the hooks had counted just before.
#[derive(Debug, Clone, Copy)]
struct Sent {
    at: Mono,
    base: u64,
}

/// Watches that the running hook counts kx-gates' own clicks in time.
#[derive(Debug, Clone)]
pub struct Watch {
    /// How soon a live hook counts them.
    alive: Duration,
    /// The oldest send no own event was counted after.
    pending: Option<Sent>,
}

impl Watch {
    /// A watch with nothing sent, for `cfg`.
    pub fn new(cfg: &MouseGuard) -> Self {
        Self {
            alive: Duration::from_millis(cfg.alive_ms),
            pending: None,
        }
    }

    /// Notes clicks sent at `at` after the hooks had counted `base` own events; an older send still uncounted stays
    /// the one watched.
    pub fn sent(&mut self, at: Mono, base: u64) {
        if self.pending.is_none_or(|p| p.base != base) {
            self.pending = Some(Sent { at, base });
        }
    }

    /// True when the hooks, now at `passed` own events, counted none since the watched send, `alive` or more after
    /// it; any count ends the watch.
    pub fn lost(&mut self, now: Mono, passed: u64) -> bool {
        let Some(p) = self.pending else {
            return false;
        };
        if passed != p.base {
            self.pending = None;
            return false;
        }
        now.saturating_since(p.at) >= self.alive
    }

    /// Forgets the watched send, as the guard removed its hook itself.
    pub fn clear(&mut self) {
        self.pending = None;
    }
}

#[cfg(test)]
mod tests {
    use kx_module_api::US_PER_MS;

    use super::*;
    use crate::mouseguard::fake::cfg;

    /// The point `ms` milliseconds after the guard started.
    fn at(ms: u64) -> Mono {
        Mono::from_us(ms * u64::from(US_PER_MS))
    }

    #[test]
    fn with_nothing_sent_the_hook_is_never_lost() {
        let mut w = Watch::new(&cfg());
        assert!(!w.lost(at(cfg().alive_ms * 10), 0));
    }

    #[test]
    fn own_clicks_not_counted_within_alive_ms_mean_the_hook_is_lost() {
        let alive = cfg().alive_ms;
        let mut w = Watch::new(&cfg());
        w.sent(at(100), 7);
        assert!(!w.lost(at(100 + alive - 1), 7), "still in time");
        assert!(w.lost(at(100 + alive), 7));
    }

    #[test]
    fn own_clicks_counted_in_time_end_the_watch() {
        let alive = cfg().alive_ms;
        let mut w = Watch::new(&cfg());
        w.sent(at(100), 7);
        assert!(!w.lost(at(101), 8));
        assert!(!w.lost(at(100 + alive * 10), 8), "nothing is watched now");
    }

    #[test]
    fn a_later_send_keeps_the_oldest_uncounted_one_and_a_counted_one_gives_way() {
        let alive = cfg().alive_ms;
        let mut w = Watch::new(&cfg());
        w.sent(at(100), 7);
        w.sent(at(100 + alive / 2), 7);
        assert!(w.lost(at(100 + alive), 7), "timed from the first send");
        let mut w = Watch::new(&cfg());
        w.sent(at(100), 7);
        w.sent(at(100 + alive / 2), 9);
        assert!(!w.lost(at(100 + alive), 9), "the first was counted");
        assert!(w.lost(at(100 + alive / 2 + alive), 9), "the second was not");
    }

    #[test]
    fn a_watch_cleared_with_its_hook_forgets_the_send() {
        let mut w = Watch::new(&cfg());
        w.sent(at(100), 7);
        w.clear();
        assert!(!w.lost(at(100 + cfg().alive_ms * 10), 7));
    }
}
