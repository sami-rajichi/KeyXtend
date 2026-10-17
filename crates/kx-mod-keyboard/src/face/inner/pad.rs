//! The scroll pad in the face's state (spec §5.2): the scroll module's state as the bus tells it, and the pointer on
//! the pad.

use super::Inner;
use kx_module_api::ports::ScrollChanged;

impl Inner {
    /// Takes the scroll module's new state, starting a new run when it comes live; the next relabel redraws a change.
    pub(in crate::face) fn heard_scroll(&mut self, now: ScrollChanged) {
        if now.live && !self.scroll.live {
            self.new_run();
        }
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

    /// The scroll module's current run, counted each time it comes live or the keyboard starts.
    pub(in crate::face) fn scroll_run(&self) -> u64 {
        self.run
    }

    /// Starts a new run, whose scroll module has heard nothing of the pointer yet.
    pub(in crate::face) fn new_run(&mut self) {
        self.run = self.run.wrapping_add(1);
    }
}
