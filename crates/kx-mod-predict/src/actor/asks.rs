//! The menu's taps (spec §6 "Controls"): forget mode, Cancel, Pause and Off saved through the module's own settings
//! handle without a restart, Undo, and the email list's paging by the rows the UI tells.

use super::store::Blob;
use super::{Actor, Flow};
use crate::pack::Pack;
use crate::settings::{LEARN_KEY, ON_KEY};
use kx_module_api::ports::WordsAsk;

impl Actor {
    /// Does what the tap asks and tells the lane; `Again` tells the last lane as it was, and turning prediction on or
    /// off opens the language again.
    pub(super) fn ask(&mut self, seed: Option<&Pack<'_>>, ask: WordsAsk) -> Flow {
        match ask {
            WordsAsk::Again => {
                self.again();
                return Flow::Go;
            }
            WordsAsk::Power(on) => return self.power(seed, on),
            WordsAsk::Forget => self.menu.forget(),
            WordsAsk::Cancel => self.menu.cancel(),
            WordsAsk::Pause(paused) => {
                self.settings.learn = !paused;
                self.write(LEARN_KEY, self.settings.learn);
            }
            WordsAsk::Undo => self.undo(),
            WordsAsk::More { rows } => {
                if self.lists_all() {
                    self.menu.open(rows);
                }
            }
            WordsAsk::Page { page, rows } => {
                let count = self.listed().list().len();
                self.menu.turn(page, rows, count);
            }
            WordsAsk::ListPick(row) => self.list_pick(seed, row),
        }
        self.tell(seed);
        Flow::Go
    }

    /// Turns prediction on or off: the menu closes and the word ends. A change saves and drops the learner and the known
    /// emails, and the loop opens the language again, so prediction off holds no pack or learner; emails whose save
    /// was refused stay for the next save.
    fn power(&mut self, seed: Option<&Pack<'_>>, on: bool) -> Flow {
        let flip = on != self.settings.on;
        self.settings.on = on;
        self.write(ON_KEY, on);
        self.menu.off();
        if flip {
            self.drop_word();
            self.save(Blob::Learned);
            self.leave();
            self.drop_emails();
            return Flow::Switch;
        }
        self.end_word(seed);
        self.tell(seed);
        Flow::Go
    }

    /// Saves `key` as `on` in the module's own section; a refusal is logged and the tap still holds this run.
    fn write(&self, key: &str, on: bool) {
        if let Err(error) = self.ports.handle.set(key, toml::Value::Boolean(on)) {
            tracing::warn!(%error, "a menu switch was not saved");
        }
    }
}
