//! A Quick-fill fill in the face's state (ADR-0038 decision 4, design "Filling"): one at a time, its first parts at
//! once and the rest after each Tab's gap, at the next call from the UI, never more than `fill_late_ms` past due, never
//! to another app in front, and not after a click outside our windows.

use super::Inner;
use crate::run::{Fill, Filled};
use kx_module_api::ports::FillType;

/// A fill waiting its gap, with how many clicks outside our windows were heard as it started.
pub(super) struct Waiting {
    fill: Fill,
    clicks: u64,
}

impl Inner {
    /// Starts fill `ask`, with `clicks` outside clicks heard so far: a held key drops, every latch lets go and the aim
    /// goes, then its parts go up to the first wait, or it is refused, touching nothing, while the module is stopped,
    /// another fill waits, or typing refuses it. The answer once done; `None` while a part waits.
    pub(in crate::face) fn start_fill(&mut self, ask: &FillType, clicks: u64) -> Option<bool> {
        if !self.running || self.fill.is_some() {
            return Some(false);
        }
        let Some(fill) = self.typing.fill(ask) else {
            return Some(false);
        };
        self.repeat.leave();
        self.lose_aim();
        self.unshown = true;
        self.go(Waiting { fill, clicks })
    }

    /// The waiting fill's next parts once due, with `clicks` outside clicks heard now: one heard since it started ends
    /// it unsent, as the caret may have moved. Its round and answer once done.
    pub(in crate::face) fn fill_due(&mut self, clicks: u64) -> Option<(u32, bool)> {
        let now = self.ports.clock.now();
        let waiting = self
            .fill
            .take_if(|w| w.clicks != clicks || w.fill.due(now))?;
        let round = waiting.fill.round();
        if waiting.clicks != clicks {
            return Some((round, false));
        }
        self.go(waiting).map(|ok| (round, ok))
    }

    /// Ends a waiting fill that a click outside our windows or its bound has overtaken, with `clicks` outside clicks
    /// heard now, sending nothing; its round, for the answer no. One still inside its bound is left alone.
    pub(in crate::face) fn expire(&mut self, clicks: u64) -> Option<u32> {
        let now = self.ports.clock.now();
        let typing = &self.typing;
        let over = |w: &mut Waiting| w.clicks != clicks || typing.overdue(&w.fill, now);
        self.fill.take_if(over).map(|w| w.fill.round())
    }

    /// The waiting fill ends with nothing more sent, as the module stops; its round, for the answer.
    pub(in crate::face) fn drop_fill(&mut self) -> Option<u32> {
        self.fill.take().map(|w| w.fill.round())
    }

    /// True while a fill waits its gap, so our keys reach only the open panel.
    pub(in crate::face) fn filling(&self) -> bool {
        self.fill.is_some()
    }

    /// Sends the fill's parts up to its next wait, keeping it while it waits; its answer once done.
    fn go(&mut self, mut waiting: Waiting) -> Option<bool> {
        let table = self.ports.layouts.current();
        let cells = self.layer.cells();
        let io = self.ports.io(&table, &cells);
        let now = self.ports.clock.now();
        match self.typing.fill_on(&io, &mut waiting.fill, now) {
            Filled::Wait(_) => {
                self.fill = Some(waiting);
                None
            }
            Filled::Done(ok) => Some(ok),
        }
    }
}
