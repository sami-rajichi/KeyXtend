//! A click on the shortcuts layer (spec §4.2): a cell's plan, and its shortcut sent with every latch held, after which
//! the latches let go as after any key, a dead key's wait ends and word prediction hears a break.

use super::{Io, TapError, Typing};
use crate::plan::{Plan, layer_plan};
use kx_module_api::ports::{Piece, ShortcutError, ShortcutId};

impl Typing {
    /// Sends shortcut `id` with every latched key held around each step; once any of it reached the app, the latches
    /// are used and a dead key's wait ends, even for one that types nothing such as F5 (P6 edge case 9). One that sent
    /// nothing keeps the latches and the wait, as any key that did not reach the app does.
    pub(super) fn shortcut(&mut self, io: &Io<'_>, id: &ShortcutId) -> Result<(), TapError> {
        let went = io.shortcuts.send(id, &self.latches.held(&[]));
        if reached(went) {
            self.dead = false;
            self.used();
            self.tell(Piece::Break);
        }
        Ok(went?)
    }
}

/// The plan of the layer key `id` drawn now, if the layer draws one.
pub(super) fn planned(io: &Io<'_>, id: &str) -> Option<Plan> {
    let cell = io.layer.iter().find(|c| c.key.id() == id)?;
    Some(layer_plan(&cell.key))
}

/// True when any of a shortcut reached the app: all of it, or the events the system took before refusing the rest.
fn reached(went: Result<(), ShortcutError>) -> bool {
    match went {
        Ok(()) => true,
        Err(ShortcutError::Refused { sent, .. }) => sent > 0,
        Err(ShortcutError::Unknown | ShortcutError::Failed) => false,
    }
}
