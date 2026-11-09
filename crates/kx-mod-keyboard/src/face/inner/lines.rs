//! The line prediction follows (ADR-0040 decisions 6 and 7): the open panel's while aimed at, else none. A change waits
//! after the pieces typed before it, and so does an edit's answer, so each reaches the bus in the order it happened.

use super::Inner;
use kx_module_api::ports::{EditDone, PanelLine, Piece};

/// One thing the bus hears from the pieces' sender, in the order it happened.
pub(in crate::face) enum Queued {
    /// A piece that reached the app.
    Piece(Piece),
    /// The line prediction follows now.
    Line(PanelLine),
    /// An edit's answer to the selection module, after the pieces before it and before its own break.
    Done(EditDone),
}

/// The line prediction heard last, and what waits behind a change.
#[derive(Default)]
pub(super) struct Lines {
    /// The line last set to go out.
    told: PanelLine,
    /// The pieces typed before a change or an answer, then that change or answer, oldest first.
    waiting: Vec<Queued>,
}

impl Inner {
    /// Sets the line in effect to go out when prediction last heard another.
    pub(super) fn note_line(&mut self) {
        if *self.aimed_line() != self.lines.told {
            self.queue_line();
        }
    }

    /// A prediction module asked: the line in effect goes out again, changed or not.
    pub(in crate::face) fn line_again(&mut self) {
        self.queue_line();
    }

    /// What the bus hears next, oldest first.
    pub(in crate::face) fn take_told(&mut self) -> Vec<Queued> {
        let mut told = std::mem::take(&mut self.lines.waiting);
        told.extend(self.typing.take_pieces().into_iter().map(Queued::Piece));
        told
    }

    /// Puts the pieces typed so far in the queue, so what comes next waits behind them.
    pub(super) fn flush(&mut self) {
        let typed = self.typing.take_pieces().into_iter().map(Queued::Piece);
        self.lines.waiting.extend(typed);
    }

    /// Puts `done` behind what was flushed, ahead of the pieces typed since, such as the edit's own break.
    pub(super) fn queue_done(&mut self, done: EditDone) {
        self.lines.waiting.push(Queued::Done(done));
    }

    /// Puts the line in effect behind the pieces typed so far.
    fn queue_line(&mut self) {
        let line = self.aimed_line().clone();
        self.flush();
        self.lines.waiting.push(Queued::Line(line.clone()));
        self.lines.told = line;
    }
}
