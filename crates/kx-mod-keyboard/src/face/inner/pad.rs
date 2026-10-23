//! The scroll pad in the face's state (spec §5.2): the scroll module's state as the bus tells it, and the pointer on
//! the pad.

use super::Inner;
use kx_module_api::ports::ScrollChanged;

impl Inner {
    /// Takes the scroll module's new state, starting a new run when it comes live; the next relabel redraws a change.
    pub(in crate::face) fn heard_scroll(&mut self, now: ScrollChanged) {
        if now.live && !self.scroll.live {
            self.run = self.run.wrapping_add(1);
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

    /// True while the pad scrolls, so its Stop shows.
    pub(in crate::face) fn scrolling(&self) -> bool {
        self.scroll.live && self.scroll.dir.is_some()
    }

    /// Notes the pointer on the pad or off it; while the layer hides the pad, the pointer is never on it.
    pub(in crate::face) fn hovered(&mut self, on: bool) {
        self.hover = on && !self.layered();
    }

    /// True while the pointer is on the pad.
    pub(in crate::face) fn hovering(&self) -> bool {
        self.hover
    }

    /// The scroll module's current run, counted each time it comes live.
    pub(in crate::face) fn scroll_run(&self) -> u64 {
        self.run
    }
}
