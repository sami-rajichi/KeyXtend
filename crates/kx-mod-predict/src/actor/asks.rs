//! The menu's taps (spec §6 "Controls"): forget mode, Cancel, Pause and Off saved through the module's own settings
//! handle without a restart, Undo, and the email list's paging by the rows the UI tells.

use super::Actor;
use crate::pack::Pack;
use crate::settings::{LEARN_KEY, ON_KEY};
use kx_module_api::ports::WordsAsk;

impl Actor {
    /// Does what the tap asks; true when the lane changes, false for `Again`, which tells the last lane as it was.
    pub(super) fn ask(&mut self, seed: Option<&Pack<'_>>, ask: WordsAsk) -> bool {
        match ask {
            WordsAsk::Again => {
                self.again();
                return false;
            }
            WordsAsk::Forget => self.menu.forget(),
            WordsAsk::Cancel => self.menu.cancel(),
            WordsAsk::Pause(paused) => {
                self.settings.learn = !paused;
                self.write(LEARN_KEY, self.settings.learn);
            }
            WordsAsk::Power(on) => self.power(seed, on),
            WordsAsk::Undo => self.undo(),
            WordsAsk::More { rows } => {
                if self.lists_all() {
                    self.menu.open(rows);
                }
            }
            WordsAsk::Page { page, rows } => self.menu.turn(page, rows, self.emails.list().len()),
            WordsAsk::ListPick(row) => self.list_pick(seed, row),
        }
        true
    }

    /// Turns prediction on or off: the menu closes, the word ends, and turning on reads the box again.
    fn power(&mut self, seed: Option<&Pack<'_>>, on: bool) {
        self.settings.on = on;
        self.write(ON_KEY, on);
        self.menu.off();
        self.pending = false;
        let told = self.tracker.reset();
        if self.following() {
            self.act(seed, told);
        }
    }

    /// Saves `key` as `on` in the module's own section; a refusal is logged and the tap still holds this run.
    fn write(&self, key: &str, on: bool) {
        if let Err(error) = self.ports.handle.set(key, toml::Value::Boolean(on)) {
            tracing::warn!(%error, "a menu switch was not saved");
        }
    }
}
