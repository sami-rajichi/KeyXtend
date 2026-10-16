//! Modifier + click (spec §4.4): the latched modifiers go down just before a left click outside our windows and up
//! just after its release.

use super::{Engine, Note};
use kx_module_api::ports::{Button, Point, PointerAct};

impl Engine {
    /// Takes the click's modifiers from the latches, so a later latch change never alters this click.
    pub(super) fn snap_mods(&mut self) {
        self.mods.clear();
        // `new` reserved room for `MAX_LATCHES`, and `set_latches` keeps no more, so this never allocates.
        self.mods.extend_from_slice(&self.latches);
    }

    /// Presses the click's modifiers, once.
    pub(super) fn mods_down(&mut self, acts: &mut Vec<PointerAct>) {
        if self.mods_down || self.mods.is_empty() {
            return;
        }
        acts.extend(self.mods.iter().map(|&key| PointerAct::KeyDown(key)));
        self.mods_down = true;
    }

    /// Lets go of the click's modifiers in reverse order and notes that the latches were used.
    pub(super) fn mods_up(&mut self, acts: &mut Vec<PointerAct>) {
        if self.mods_down {
            acts.extend(self.mods.iter().rev().map(|&key| PointerAct::KeyUp(key)));
            self.mods_down = false;
            self.note(Note::LatchesUsed);
        }
        self.mods.clear();
    }

    /// Sends a left press at `at` inside its modifiers, if it has any; true when the physical press is replaced.
    pub(super) fn press_through(&mut self, at: Point, acts: &mut Vec<PointerAct>) -> bool {
        if self.mods.is_empty() {
            return false;
        }
        self.mods_down(acts);
        acts.push(PointerAct::Down(Button::Left, at));
        true
    }

    /// Replaces the release of a click with modifiers: left up at `at`, then the modifiers.
    pub(super) fn click_up(&mut self, at: Point, acts: &mut Vec<PointerAct>) {
        acts.push(PointerAct::Up(Button::Left, at));
        self.mods_up(acts);
    }
}
