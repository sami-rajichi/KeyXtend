//! History's acts on the ticked texts, in tick order: Insert, Copy, Pin or Unpin, Delete; Delete all asks first, and
//! its question counts the unpinned texts `History::ask_all` names. Insert leaves the order as it is.

use kx_module_api::ports::PanelButton;

use super::{Do, Panel};
use crate::view::Facts;

impl Panel {
    /// A History button.
    pub(super) fn history_button(&mut self, button: PanelButton, facts: &Facts<'_>) -> Vec<Do> {
        let ids = self.ticked_texts();
        let done = match button {
            PanelButton::Insert => Do::Insert(ids),
            PanelButton::Copy => return vec![Do::Copy(ids)],
            PanelButton::Pin if self.unpins(facts) => Do::Unpin(ids),
            PanelButton::Pin => Do::Pin(ids),
            PanelButton::Delete => Do::Delete(ids),
            PanelButton::DeleteAll => {
                self.asked = facts.history.ask_all();
                return Vec::new();
            }
            _ => return Vec::new(),
        };
        self.ticks.clear();
        vec![done]
    }
}
