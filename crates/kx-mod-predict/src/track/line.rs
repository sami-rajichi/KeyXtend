//! Following a panel's line (ADR-0040): its text reads as a box's, and focus moves in the app behind are only noted.

use super::Tracker;
use kx_module_api::ports::FieldChange;

impl Tracker {
    /// Notes the box focus moved to, keeping all it follows.
    pub fn note(&mut self, change: FieldChange) {
        self.field = Some(change.box_id);
    }

    /// Takes a panel's line as a read, with its open token counted as typed, so `@@` lists there as in a box; the module
    /// learns nothing on a line.
    pub fn read_line(&mut self, text: &str, knows: &dyn Fn(&str) -> bool) {
        self.read(Some(text), knows);
        self.token.mark_typed();
    }
}
