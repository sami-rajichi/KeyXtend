//! The language in front: a switch saves the old learner and resets the tracker, then the loop opens only the new
//! language's pack, parses it once, and loads its learner reconciled with it. With prediction off nothing opens.

use super::store::Blob;
use super::{Actor, Flow};
use crate::pack::Pack;
use kx_module_api::ports::{LayoutAsk, PackView, TypingModeAsk, fits_iso};

impl Actor {
    /// The keyboard shows a layout in `iso`, named `name`: the word ends unlearned, and a new language saves the old
    /// learner.
    pub(super) fn lang(&mut self, seed: Option<&Pack<'_>>, iso: String, name: String) -> Flow {
        self.lang_name = name;
        self.menu.key();
        if iso == self.iso {
            self.end_word(seed);
            self.tell(seed);
            return Flow::Go;
        }
        self.drop_word();
        self.save(Blob::Learned);
        self.leave();
        self.code = self
            .ports
            .packs
            .pack_of(&iso)
            .unwrap_or_else(|| iso.clone());
        self.iso = iso;
        self.stats.switches += 1;
        Flow::Switch
    }

    /// Opens the pack serving the language in front, while prediction is on; a missing or unreadable one is told once
    /// if the build shipped it.
    pub fn open(&mut self) -> Option<Box<dyn PackView>> {
        if !self.settings.on || !fits_iso(&self.code) {
            return None;
        }
        let opened = self.ports.packs.open(&self.code);
        let view = opened.unwrap_or_else(|error| {
            tracing::warn!(%error, "word pack not opened");
            None
        });
        if view.is_none() {
            self.say_missing();
        }
        view
    }

    /// Reads `view` once as the pack serving the language in front; one that does not read, or names another language
    /// than its words' code, counts as missing.
    pub fn parse<'v>(&mut self, view: Option<&'v dyn PackView>) -> Option<Pack<'v>> {
        let parsed = Pack::parse(view?.bytes());
        let pack = match parsed {
            Ok(pack) if pack.iso() == self.code => Some(pack),
            Ok(_) => {
                tracing::warn!("word pack of another language");
                None
            }
            Err(error) => {
                tracing::warn!(%error, "word pack not read");
                None
            }
        };
        if pack.is_none() {
            self.say_missing();
        }
        pack
    }

    /// Settles in the language whose pack is `seed`: loads its learner, and the known emails once, while prediction is
    /// on, purges, reads the box and tells the lane.
    pub fn arrive(&mut self, seed: Option<&Pack<'_>>) {
        if self.settings.on {
            self.load_learner(seed);
            self.load_emails();
        }
        self.saves.day = None;
        self.new_day();
        self.read(seed);
        self.tell(seed);
    }

    /// Says the module is here: takes the box's kind and asks for the language in front and the typing mode, as the
    /// layouts and the keyboard may have started first.
    pub fn begin(&mut self) {
        if let Some(run) = self.ports.field.as_ref() {
            self.kind = run.kind();
        }
        self.publish(&LayoutAsk);
        self.publish(&TypingModeAsk);
    }
}
