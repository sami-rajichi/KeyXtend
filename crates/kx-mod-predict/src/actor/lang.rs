//! The language in front: a switch saves the old learner and resets the tracker, then the loop opens only the new
//! language's pack, parses it once, and loads its learner reconciled with it.

use super::store::Blob;
use super::{Actor, Flow};
use crate::pack::Pack;
use kx_module_api::ports::{PackView, fits_iso};

impl Actor {
    /// The keyboard shows a layout in `iso`, named `name`: the word ends unlearned, and a new language saves the old
    /// learner.
    pub(super) fn lang(
        &mut self,
        seed: Option<&Pack<'_>>,
        iso: String,
        name: String,
        rtl: bool,
    ) -> Flow {
        self.rtl = rtl;
        self.lang_name = name;
        self.menu.key();
        self.pending = false;
        let told = self.tracker.reset();
        if iso == self.iso {
            if self.following() {
                self.act(seed, told);
            }
            self.tell(seed);
            return Flow::Go;
        }
        self.save(Blob::Learned);
        self.leave();
        self.iso = iso;
        self.stats.switches += 1;
        Flow::Switch
    }

    /// Opens the language in front's pack; a missing or unreadable one is told once when the build shipped it.
    pub fn open(&mut self) -> Option<Box<dyn PackView>> {
        if !fits_iso(&self.iso) {
            return None;
        }
        let opened = self.ports.packs.open(&self.iso);
        let view = opened.unwrap_or_else(|error| {
            tracing::warn!(%error, "word pack not opened");
            None
        });
        if view.is_none() {
            self.say_missing();
        }
        view
    }

    /// Reads `view` once as the language in front's pack; one that does not read, or names another language, counts
    /// as missing.
    pub fn parse<'v>(&mut self, view: Option<&'v dyn PackView>) -> Option<Pack<'v>> {
        let parsed = Pack::parse(view?.bytes());
        let pack = match parsed {
            Ok(pack) if pack.iso() == self.iso => Some(pack),
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

    /// Settles in the language whose pack is `seed`: loads its learner, purges, reads the box and tells the lane.
    pub fn arrive(&mut self, seed: Option<&Pack<'_>>) {
        self.load_learner(seed);
        self.saves.day = None;
        self.new_day();
        self.read(seed);
        self.tell(seed);
    }

    /// Says the module is here: loads the known emails, takes the box's kind and asks for the language in front.
    pub fn begin(&mut self) {
        if let Some(run) = self.ports.field.as_ref() {
            self.kind = run.kind();
        }
        self.load_emails();
        self.publish(&kx_module_api::ports::LayoutAsk);
    }
}
