//! The hook's stop and the line it logs with the hook's numbers, never a point or a key.

use super::Actor;
use tracing::Level;

/// The message of the line logged with the hook's numbers each time the hook stops; kx-gates reads that line back.
pub const STOPPED_LINE: &str = "mouse hook stopped";

/// Logs the hook's numbers `s` at `level` under `STOPPED_LINE`, so the field list lives in one place.
macro_rules! stopped {
    ($level:expr, $s:expr) => {
        tracing::event!(
            $level,
            events = $s.events,
            p99_us = $s.p99_us,
            max_us = $s.max_us,
            max_wait_ms = $s.max_wait_ms,
            reinstalls = $s.reinstalls,
            send_errors = $s.send_errors,
            lost_echoes = $s.lost_echoes,
            passed_inside = $s.passed_inside,
            "{STOPPED_LINE}"
        )
    };
}

impl Actor {
    /// Stops the hook, which lets go of anything held, and logs its numbers. A run a mode or a latch held logs at info;
    /// one held only by scrolling or presses logs at debug, as those end at every word or scroll.
    pub(super) fn drop_run(&mut self) {
        let Some(live) = self.live.take() else {
            return;
        };
        let (s, loud) = (live.run.stats(), live.loud);
        drop(live);
        if loud {
            stopped!(Level::INFO, s);
        } else {
            stopped!(Level::DEBUG, s);
        }
    }
}
