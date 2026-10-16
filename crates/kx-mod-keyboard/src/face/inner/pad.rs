//! The scroll pad in the face's state (spec §5.2): the scroll module's state as the bus tells it, and the pointer on
//! the pad.

use super::Inner;
use kx_module_api::ports::ScrollChanged;

impl Inner {
    /// Takes the scroll module's new state; the next relabel redraws when it changed.
    pub(in crate::face) fn heard_scroll(&mut self, now: ScrollChanged) {
        if self.scroll != now {
            self.scroll = now;
            self.unshown = true;
        }
    }

    /// True while the scroll module runs, so the pad works.
    pub(in crate::face) fn scrolls(&self) -> bool {
        self.scroll.live
    }

    /// Notes the pointer on the pad or off it.
    pub(in crate::face) fn hovered(&mut self, on: bool) {
        self.hover = on;
    }

    /// True while the pointer is on the pad.
    pub(in crate::face) fn hovering(&self) -> bool {
        self.hover
    }
}
