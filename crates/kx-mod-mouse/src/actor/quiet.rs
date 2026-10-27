//! The needs that start the hook quietly, telling no one when it refuses: held latches, a scrolling pad and presses
//! word prediction wants told.

use super::Actor;
use crate::hook_engine::Cmd;
use kx_module_api::ports::Scan;

impl Actor {
    /// Keeps the keyboard's latches; they start the hook when they hold a key.
    pub(super) fn latch(&mut self, held: Vec<Scan>) {
        self.latches.clone_from(&held);
        if self.live.is_some() || !held.is_empty() {
            self.command(Cmd::Latches(held));
        }
    }

    /// Keeps whether the pad scrolls; the user moved it, so a change re-arms a spent restart.
    pub(super) fn scroll(&mut self, on: bool) {
        let changed = std::mem::replace(&mut self.scrolling, on) != on;
        self.restarted &= !changed;
        self.need(changed, on, Cmd::Scroll(on));
    }

    /// Keeps whether word prediction wants presses, and logs each change, never a point. Prediction flips it by
    /// itself, so it never re-arms a spent restart and starts no run while one is spent.
    pub(super) fn presses(&mut self, on: bool) {
        let changed = std::mem::replace(&mut self.wanted, on) != on;
        if changed {
            tracing::debug!(on, "mouse presses wanted");
        }
        self.need(changed, on && !self.restarted, Cmd::Presses(on));
    }

    /// A quiet need changed: the running engine gets `cmd`, or with no hook a need that may `start` starts one.
    fn need(&mut self, changed: bool, start: bool, cmd: Cmd) {
        match self.live {
            Some(_) if changed => self.command(cmd),
            None if start => self.start(Some(cmd)),
            _ => {}
        }
    }
}
