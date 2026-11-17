//! The store on the voice shelf: opened at the start, History's rules after it and after each `[voice]` change, each
//! text, recording, count, check and Compare result saved. A failed save tells once and is tried with the next change
//! and at the stop; a shelf that does not open keeps nothing this run.

use std::sync::Arc;

use kx_module_api::SecretBytes;
use kx_module_api::keys;
use kx_module_api::ports::{MAX_BLOB_BYTES, SealedError};

use super::Actor;
use crate::VoiceSettings;
use crate::flow::{Count, Kept};
use crate::history::{Edit, Rules, Stamp};
use crate::panel::Do;
use crate::store::Store;

/// The store and History's rules.
pub(super) struct Keep {
    /// The store; `None` when the shelf did not open, so nothing is kept.
    store: Option<Store>,
    /// What History keeps now.
    pub rules: Rules,
    /// The run's saves would be hidden (ADR-0037).
    hidden: bool,
    /// The last save failed and its note showed.
    unsaved: bool,
    /// The numbers whose last save failed.
    redo: Redo,
}

/// The numbers whose last save failed; each is saved whole, so the next save of it or the stop puts it right.
#[derive(Copy, Clone, Default)]
struct Redo {
    counts: bool,
    checks: bool,
    scores: bool,
}

impl Keep {
    /// No store yet, keeping nothing until it opens.
    pub fn new(settings: &VoiceSettings, hidden: bool) -> Self {
        Self {
            store: None,
            rules: Rules::of(settings, true),
            hidden,
            unsaved: false,
            redo: Redo::default(),
        }
    }
}

impl Actor {
    /// Opens the store and takes what it holds; a newer or locked one keeps nothing, and damage is told by kind.
    pub(super) fn open_store(&mut self) {
        match Store::open(Arc::clone(&self.ports.shelf), MAX_BLOB_BYTES) {
            Ok(opened) => {
                self.keep.store = Some(opened.store);
                self.held.history = opened.history;
                self.held.counts = opened.counts;
                self.held.checks = opened.checks;
                self.held.scores = opened.scores;
                if opened.damaged.texts > 0 {
                    self.say(keys::VOICE_TEXTS_DAMAGED);
                }
                if opened.damaged.numbers > 0 {
                    self.say(keys::VOICE_NUMBERS_DAMAGED);
                }
            }
            Err(error) => {
                tracing::warn!(%error, "the dictation history could not be opened");
                self.say(keys::VOICE_LOCKED);
            }
        }
        self.follow_rules();
    }

    /// Takes History's rules from `[voice]` and trims History to them.
    pub(super) fn follow_rules(&mut self) {
        let shut = self.keep.hidden || self.keep.store.is_none();
        self.keep.rules = Rules::of(&self.held.settings, shut);
        let edit = self.held.history.trim(&self.keep.rules);
        self.apply(edit);
    }

    /// Keeps a dictated text in History.
    pub(super) fn keep_text(&mut self, kept: Kept) {
        let at = Stamp::of(&self.ports.calendar.now());
        let edit = self.held.history.keep(kept, at, &self.keep.rules);
        self.apply(edit);
    }

    /// Keeps dictation `take`'s recording with its text.
    pub(super) fn keep_sound(&mut self, take: u32, pieces: &[SecretBytes]) {
        let Some(store) = self.keep.store.as_mut() else {
            return;
        };
        let kept = store.keep_sound(&mut self.held.history, take, pieces, &self.keep.rules);
        self.saved(kept.map(drop));
    }

    /// Counts a written dictation on today's date.
    pub(super) fn count(&mut self, count: &Count) {
        let today = self.ports.calendar.now().date;
        self.held.counts.add(today, count);
        self.save_counts();
    }

    /// Saves the counts.
    fn save_counts(&mut self) {
        if let Some(store) = &self.keep.store {
            let saved = store.save_counts(&mut self.held.counts);
            self.keep.redo.counts = saved.is_err();
            self.saved(saved);
        }
    }

    /// Saves the cards' checks.
    pub(super) fn save_checks(&mut self) {
        if let Some(store) = &self.keep.store {
            let saved = store.save_checks(&self.held.checks);
            self.keep.redo.checks = saved.is_err();
            self.saved(saved);
        }
    }

    /// Saves Compare's results.
    pub(super) fn save_scores(&mut self) {
        if let Some(store) = &self.keep.store {
            let saved = store.save_scores(&self.held.scores);
            self.keep.redo.scores = saved.is_err();
            self.saved(saved);
        }
    }

    /// History's Pin, Unpin, Delete and Delete all.
    pub(super) fn edit(&mut self, edit: Do) {
        let history = &mut self.held.history;
        let edit = match edit {
            Do::Pin(ids) => history.pin(&ids),
            Do::Unpin(ids) => history.unpin(&ids, Stamp::of(&self.ports.calendar.now())),
            Do::Delete(ids) => history.delete(&ids),
            Do::DeleteAll(asked) => history.delete_all(asked),
            _ => return,
        };
        self.apply(edit);
    }

    /// Tries once more what failed to save, at the stop: History's changes, then each number whose save failed.
    pub(super) fn retry_saves(&mut self) {
        self.apply(Edit::default());
        let redo = std::mem::take(&mut self.keep.redo);
        if redo.counts {
            self.save_counts();
        }
        if redo.checks {
            self.save_checks();
        }
        if redo.scores {
            self.save_scores();
        }
    }

    /// Carries out `edit` on the shelf, with what failed before.
    fn apply(&mut self, edit: Edit) {
        if let Some(store) = self.keep.store.as_mut() {
            let saved = store.apply(&self.held.history, edit);
            self.saved(saved);
        }
    }

    /// A save's end: a failure is logged by kind and told once until a save succeeds.
    fn saved(&mut self, saved: Result<(), SealedError>) {
        match saved {
            Ok(()) => self.keep.unsaved = false,
            Err(error) => {
                tracing::warn!(%error, "the dictation history could not be saved");
                if !std::mem::replace(&mut self.keep.unsaved, true) {
                    self.say(keys::VOICE_UNSAVED);
                }
            }
        }
    }
}
