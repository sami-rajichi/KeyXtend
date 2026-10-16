//! Mouse assist in the face's state (spec §5.1): the mouse module's state as the bus tells it, the latches a mouse
//! click used, and the latched set the bus hears.

use super::Inner;
use kx_module_api::Bus;
use kx_module_api::ports::{AssistChanged, Scan};

impl Inner {
    /// Takes the mouse module's new state; the next relabel redraws when it changed.
    pub(in crate::face) fn heard(&mut self, now: AssistChanged) {
        if self.assist != now {
            self.assist = now;
            self.unshown = true;
        }
    }

    /// True while the mouse module runs, so its side keys work.
    pub(in crate::face) fn live(&self) -> bool {
        self.assist.live
    }

    /// A mouse click used the latches: they let go as their mode says, and the next relabel redraws them.
    pub(in crate::face) fn clicked(&mut self) {
        if self.typing.clicked() {
            self.unshown = true;
        }
    }

    /// The latched set as the bus carries it: each latch's scan code, in a fixed order.
    pub(in crate::face) fn held(&self) -> Vec<Scan> {
        self.typing.latches().held(&[])
    }

    /// The bus, as the kernel last handed it.
    pub(in crate::face) fn bus(&self) -> Bus {
        self.ports.bus.clone()
    }

    /// True when the bus changed what the state shows since the last state view.
    pub(in crate::face) fn unshown(&self) -> bool {
        self.unshown
    }
}
