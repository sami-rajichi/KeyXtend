//! The selection chips' edit keys in the face's state (ADR-0041 decision 4): unlike a paste, every latch stays, and
//! the answer waits in the pieces' queue, so the bus hears earlier pieces, the answer, then the edit's break.

use super::Inner;
use kx_module_api::ports::{Edit, EditAsk, EditDone};

impl Inner {
    /// Sends the edit `ask` names and queues its answer after the pieces typed before it, sent or not, `ok` only when
    /// all of it went; true when it was sent at all. While the module is stopped or a fill waits, nothing is sent.
    pub(in crate::face) fn edit(&mut self, ask: EditAsk) -> bool {
        let sent = self.send_edit(ask.edit, ask.terminal);
        let ok = sent == Some(true);
        self.queue_done(EditDone {
            round: ask.round,
            ok,
        });
        sent.is_some()
    }

    /// Ends a held key as its let-go does and takes the aim, which the next relabel redraws, even when the send is then
    /// refused, as a paste does; then sends `edit` with nothing held, true when all of it went. `None` while the module
    /// is stopped or a fill waits, which sends nothing and leaves every latch. Either way the pieces typed so far are
    /// queued first.
    fn send_edit(&mut self, edit: Edit, terminal: bool) -> Option<bool> {
        if !self.running || self.filling() {
            self.flush();
            return None;
        }
        self.unshown |= self.ended().state;
        self.lose_aim();
        // The pieces typed so far go out before the answer; the edit's own break waits behind it.
        self.flush();
        let table = self.ports.layouts.current();
        let cells = self.layer.cells();
        let went = self
            .typing
            .edit(&self.ports.io(&table, &cells), edit, terminal);
        Some(went.is_ok())
    }
}
