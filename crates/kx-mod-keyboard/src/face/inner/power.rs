//! The Power key in the face's state (spec §8.5): the power module's state as the bus tells it.

use super::Inner;
use kx_module_api::ports::PowerChanged;

impl Inner {
    /// Takes the power module's new state; the next relabel redraws when it changed.
    pub(in crate::face) fn heard_power(&mut self, now: PowerChanged) {
        if self.power != now {
            self.power = now;
            self.unshown = true;
        }
    }

    /// True while the power module runs, so the Power key works.
    pub(in crate::face) fn powers(&self) -> bool {
        self.power.live
    }
}
