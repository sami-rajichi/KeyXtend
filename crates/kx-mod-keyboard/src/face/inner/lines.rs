//! The line prediction follows (ADR-0040 decisions 6 and 7): the open panel's while aimed at, else none. A change waits
//! after the pieces typed before it, so both reach the bus in the order they happened.

use super::Inner;
use kx_module_api::ports::{PanelLine, Piece};

/// One thing prediction hears, in the order it happened.
pub(in crate::face) enum ToPredict {
    /// A piece that reached the app.
    Piece(Piece),
    /// The line it follows now.
    Line(PanelLine),
}

/// The line prediction heard last, and what waits behind a change.
#[derive(Default)]
pub(super) struct Lines {
    /// The line last set to go out.
    told: PanelLine,
    /// The pieces typed before a change, then that change, oldest first.
    waiting: Vec<ToPredict>,
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

    /// What prediction hears next, oldest first.
    pub(in crate::face) fn take_told(&mut self) -> Vec<ToPredict> {
        let mut told = std::mem::take(&mut self.lines.waiting);
        told.extend(self.typing.take_pieces().into_iter().map(ToPredict::Piece));
        told
    }

    /// Puts the line in effect behind the pieces typed so far.
    fn queue_line(&mut self) {
        let line = self.aimed_line().clone();
        let typed = self.typing.take_pieces().into_iter().map(ToPredict::Piece);
        self.lines.waiting.extend(typed);
        self.lines.waiting.push(ToPredict::Line(line.clone()));
        self.lines.told = line;
    }
}
