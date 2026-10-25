//! What the keyboard and the box tell: pieces, focus and clicks, and what the tracker hands back to learn or read. A
//! password box, or prediction turned off, follows nothing.

use super::Actor;
use super::store::Blob;
use crate::pack::Pack;
use crate::score::knows;
use crate::track::Told;
use kx_module_api::ports::{FieldChange, FieldKind, Piece};

impl Actor {
    /// Follows one piece the keyboard typed; true, as the lane changes.
    pub(super) fn typed(&mut self, seed: Option<&Pack<'_>>, piece: &Piece) -> bool {
        self.stats.pieces += 1;
        self.menu.key();
        self.pending = false;
        if self.following() {
            let told = self.tracker.typed(piece, &|w: &str| knows(seed, w));
            self.act(seed, told);
        }
        true
    }

    /// Focus moved: a new box resets the tracker and is read when it is a text box.
    pub(super) fn focus(&mut self, seed: Option<&Pack<'_>>, change: FieldChange) -> bool {
        self.kind = change.kind;
        self.menu.key();
        self.pending = false;
        let told = self.tracker.focus(change);
        if self.following() {
            self.act(seed, told);
        } else {
            self.tracker.reset();
        }
        true
    }

    /// A press outside our windows ends the word and reads the box again.
    pub(super) fn clicked(&mut self, seed: Option<&Pack<'_>>) -> bool {
        self.menu.key();
        self.pending = false;
        let told = self.tracker.reset();
        if self.following() {
            self.act(seed, told);
        }
        true
    }

    /// True when the tracker follows the box: prediction is on and the box is no password box.
    pub(super) fn following(&self) -> bool {
        self.settings.on && self.kind != FieldKind::Password
    }

    /// Learns, saves an email or reads, as the tracker asks.
    pub(super) fn act(&mut self, seed: Option<&Pack<'_>>, told: Vec<Told>) {
        for item in told {
            match item {
                Told::Learn { word, context } => {
                    self.learn_word(seed, word.expose(), context.expose());
                }
                Told::Email(email) => self.learn_email(email.expose()),
                Told::Read => self.read(seed),
            }
        }
    }

    /// Reads at most `context_words` words before the cursor of a text box; a failed read counts as no context.
    pub(super) fn read(&mut self, seed: Option<&Pack<'_>>) {
        if !self.settings.on || self.kind != FieldKind::Text {
            return;
        }
        let Some(run) = self.ports.field.as_mut() else {
            return;
        };
        let read = run.before_cursor(self.settings.context_words);
        self.stats.reads += 1;
        let text = read.unwrap_or_else(|error| {
            tracing::debug!(%error, "the focused box was not read");
            None
        });
        let text = text.as_ref().map(|t| t.expose().as_str());
        self.tracker.read(text, &|w: &str| knows(seed, w));
    }

    /// True when finished words and emails are learned: prediction and learning are on, in a text box.
    fn learning(&self) -> bool {
        self.settings.on && self.settings.learn && self.kind == FieldKind::Text
    }

    /// Learns `word` after `context` today, when learning.
    fn learn_word(&mut self, seed: Option<&Pack<'_>>, word: &str, context: &[String]) {
        if !self.learning() {
            return;
        }
        let Some(learner) = self.learner.as_mut() else {
            return;
        };
        let context: Vec<&str> = context.iter().map(String::as_str).collect();
        learner.learn(seed, word, &context, self.ports.days.today());
        self.stats.learned += 1;
        self.changed(Blob::Learned);
    }

    /// Puts `email` first in the known emails, when learning.
    fn learn_email(&mut self, email: &str) {
        if self.learning() && self.emails.add(email, self.email_cfg.max_emails) {
            self.stats.emails += 1;
            self.changed(Blob::Emails);
        }
    }
}
