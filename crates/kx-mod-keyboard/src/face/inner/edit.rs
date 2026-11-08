//! The selection chips' edit keys in the face's state (ADR-0041 decision 4): unlike a paste, every latch stays.

use super::Inner;
use kx_module_api::ports::Edit;

impl Inner {
    /// Ends a held key as its let-go does and takes the aim, which the next relabel redraws, then sends `edit` with
    /// nothing held; true when all of it went. `None` while the module is stopped or a fill waits, which touches nothing.
    pub(in crate::face) fn edit(&mut self, edit: Edit, terminal: bool) -> Option<bool> {
        if !self.running || self.filling() {
            return None;
        }
        self.unshown |= self.ended().state;
        self.lose_aim();
        let table = self.ports.layouts.current();
        let cells = self.layer.cells();
        let went = self
            .typing
            .edit(&self.ports.io(&table, &cells), edit, terminal);
        Some(went.is_ok())
    }
}
