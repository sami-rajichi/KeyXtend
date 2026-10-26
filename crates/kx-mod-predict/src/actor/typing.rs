//! What the keyboard and the box tell: pieces, focus and clicks, and what the tracker hands back to learn or read. A
//! password box, or prediction turned off, follows nothing, and the kind the watch or a read sees wins over a late
//! focus event.

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
        self.end_word(seed);
        true
    }

    /// Ends the word unlearned, then reads the box again when it is followed.
    pub(super) fn end_word(&mut self, seed: Option<&Pack<'_>>) {
        let told = self.drop_word();
        if self.following() {
            self.act(seed, told);
        }
    }

    /// Ends the word unlearned and drops a pick still waiting; gives what the tracker asks for.
    pub(super) fn drop_word(&mut self) -> Vec<Told> {
        self.pending = false;
        self.tracker.reset()
    }

    /// True when the tracker follows the box: prediction is on and the box, as the watch sees it now, is no password
    /// box.
    pub(super) fn following(&mut self) -> bool {
        self.moved();
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

    /// Reads at most `context_words` words before the cursor of a text box the watch still sees; a failed read counts
    /// as no context, and another kind the read saw wins at once.
    pub(super) fn read(&mut self, seed: Option<&Pack<'_>>) {
        if !self.settings.on || self.moved() || !self.kind.reads() {
            return;
        }
        let Some(run) = self.ports.field.as_mut() else {
            return;
        };
        let read = run.before_cursor(self.settings.context_words);
        self.stats.reads += 1;
        let text = match read {
            Ok(read) if self.seen(read.kind) => return,
            Ok(read) => read.text,
            Err(error) => {
                tracing::debug!(%error, "the focused box was not read");
                None
            }
        };
        if self.moved() {
            return;
        }
        let text = text.as_ref().map(|t| t.expose().as_str());
        self.tracker.read(text, &|w: &str| knows(seed, w));
    }

    /// True when finished words and emails are learned: prediction and learning are on, in a text box the watch still
    /// sees.
    fn learning(&mut self) -> bool {
        self.learns() && !self.moved() && self.kind == FieldKind::Text
    }

    /// True when prediction and learning are on.
    pub(super) fn learns(&self) -> bool {
        self.settings.on && self.settings.learn
    }

    /// True when the watch already sees another kind of box, and no text box, while its focus event is still on its
    /// way: that kind wins at once, and tracking stops.
    pub(super) fn moved(&mut self) -> bool {
        let seen = self.ports.field.as_ref().map_or(self.kind, |r| r.kind());
        self.seen(seen)
    }

    /// True when `seen`, the kind the platform saw, is another kind than the box followed and no text box: it wins,
    /// and tracking stops.
    fn seen(&mut self, seen: FieldKind) -> bool {
        let moved = seen != FieldKind::Text && seen != self.kind;
        if moved {
            self.kind = seen;
            self.tracker.reset();
        }
        moved
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
