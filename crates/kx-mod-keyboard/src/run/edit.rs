//! The edit keys the selection chips ask for (ADR-0041 decision 4): exactly Copy, Cut or Paste, or a terminal's own
//! copy and paste keys, sent with nothing held, adding no latch and leaving every latch as it was.

use super::layer::reached;
use super::{Io, TapError, Typing};
use kx_module_api::ports::{
    COPY_ID, COPY_TERM_ID, CUT_ID, Edit, PASTE_ID, PASTE_TERM_ID, Piece, ShortcutError, ShortcutId,
};

impl Typing {
    /// Sends `edit` once with nothing held, by a terminal's keys when `terminal`. Once any of it reached the app, a
    /// dead key's wait ends, a picked word going out stops and prediction hears a break; the latches stay as they were.
    ///
    /// # Errors
    /// `Shortcut` when the edit did not all reach the app, or `Unknown`, sending nothing, for a Cut in a terminal.
    pub fn edit(&mut self, io: &Io<'_>, edit: Edit, terminal: bool) -> Result<(), TapError> {
        let id = edit_id(edit, terminal).and_then(ShortcutId::new);
        let id = id.ok_or(ShortcutError::Unknown)?;
        let went = io.shortcuts.send(&id, &[]);
        if reached(went) {
            self.dead = false;
            self.sent = self.sent.wrapping_add(1);
            self.tell(Piece::Break);
        }
        Ok(went?)
    }
}

/// The shortcut `edit` sends, a terminal's own when `terminal`; none for a Cut there, as no terminal key cuts.
fn edit_id(edit: Edit, terminal: bool) -> Option<&'static str> {
    match (edit, terminal) {
        (Edit::Copy, false) => Some(COPY_ID),
        (Edit::Copy, true) => Some(COPY_TERM_ID),
        (Edit::Cut, false) => Some(CUT_ID),
        (Edit::Cut, true) => None,
        (Edit::Paste, false) => Some(PASTE_ID),
        (Edit::Paste, true) => Some(PASTE_TERM_ID),
    }
}
