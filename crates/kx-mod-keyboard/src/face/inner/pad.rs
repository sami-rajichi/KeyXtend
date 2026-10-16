//! The scroll pad in the face's state (spec §5.2): the scroll module's state as the bus tells it, and the pointer on
//! the pad.

use super::Inner;
use kx_module_api::ports::ScrollChanged;

impl Inner {
    /// Takes the scroll module's new state, and the next relabel redraws when it changed. True when the module came
    /// live while the pointer rests on the pad, so it must hear that.
    pub(in crate::face) fn heard_scroll(&mut self, now: ScrollChanged) -> bool {
        let woke = now.live && !self.scroll.live && self.hover;
        if self.scroll != now {
            self.scroll = now;
            self.unshown = true;
        }
        woke
    }

    /// True while the scroll module runs, so the pad works.
    pub(in crate::face) fn scrolls(&self) -> bool {
        self.scroll.live
    }

    /// Notes the pointer on the pad or off it; true when the running scroll module must hear the change.
    pub(in crate::face) fn hovered(&mut self, on: bool) -> bool {
        let changed = self.hover != on;
        self.hover = on;
        changed && self.scrolls()
    }
}
