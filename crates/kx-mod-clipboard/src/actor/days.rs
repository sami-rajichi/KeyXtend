//! Calendar days (spec §7 "Limits", edge 8): at local midnight, or when the clock is set to another day, the sweep
//! takes the days and media past their limits, the folders past `keep_days` drop whole, and the history saves.

use std::time::Duration;

use kx_module_api::MS_PER_S;
use kx_module_api::ports::{HOURS_PER_DAY, LocalTime, MIN_PER_HOUR, S_PER_MIN};

use super::Actor;
use crate::keep::sweep;

/// Milliseconds in a minute.
const MIN_MS: i64 = S_PER_MIN * MS_PER_S as i64;
/// Minutes in a day.
const DAY_MIN: i64 = HOURS_PER_DAY * MIN_PER_HOUR;

impl Actor {
    /// Sweeps when the calendar shows another day than the last sweep's, forward or back.
    pub(super) fn new_day(&mut self) {
        let today = self.ports.calendar.now().date;
        if today != self.today {
            self.today = today;
            self.sweep();
        }
    }

    /// Takes out what passed its limits on today, drops the folders past `keep_days`, then saves, as after the load.
    pub(super) fn sweep(&mut self) {
        for gone in sweep(&mut self.history, self.today, &self.limits) {
            self.forget(&gone);
        }
        if let Err(error) = self.store.drop_past(self.today, self.limits.keep_days) {
            tracing::warn!(%error, "an old clipboard day stays until the next sweep");
        }
        self.save();
        self.room_stale = true;
        self.redraw = true;
    }
}

/// How long from `now` until the next local midnight; a summer-time change moves it, and each wake checks the day.
pub(super) fn until_midnight(now: &LocalTime) -> Duration {
    let minutes = i64::from(now.hour) * MIN_PER_HOUR + i64::from(now.minute);
    let into_day = minutes * MIN_MS + now.unix_ms.rem_euclid(MIN_MS);
    let left = (DAY_MIN * MIN_MS - into_day).max(0);
    Duration::from_millis(u64::try_from(left).unwrap_or_default())
}
