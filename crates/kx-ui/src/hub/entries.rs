//! The end of the keyboard's entry watch: the scroll pad's target point is kept only while the scroll module runs
//! (ADR-0029), so the guard forgets it once the module stops.

use super::Hub;

impl Hub {
    /// Stops keeping the pointer's last point outside our windows titled `title`, shown or not, and forgets the point
    /// kept; `watch_entries` starts again. Call it on the UI thread.
    pub fn unwatch_entries(&self, title: &str) {
        self.ports.guard.unwatch_entries(title);
    }
}
