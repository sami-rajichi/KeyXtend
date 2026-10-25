//! Picks: a chip of the lane's round types through the keyboard with one `TypeWord`, erasing only what the box
//! confirms; in forget mode a word or known-email chip is forgotten instead, and the list's rows type their email.

use super::Actor;
use super::store::Blob;
use crate::lane::Pick;
use crate::learn::Learner;
use crate::pack::Pack;
use crate::track::{EmailAsk, Plan};
use kx_module_api::Redacted;
use kx_module_api::ports::{ChipPicked, FieldKind, TypeWord};

impl Actor {
    /// A chip was tapped; a tap on an older lane, or on no chip, does nothing. True when the lane changed.
    pub(super) fn picked(&mut self, seed: Option<&Pack<'_>>, pick: ChipPicked) -> bool {
        if pick.round != self.round {
            self.stats.stale += 1;
            return false;
        }
        let at = usize::from(pick.index);
        let Some(chosen) = self.chips.picks.get(at).cloned() else {
            return false;
        };
        match chosen {
            Pick::Word { stored, .. } if self.menu.forgetting => self.forget(seed, &stored, at),
            Pick::Email(plan) if self.menu.forgetting => self.forget_email(&plan.text),
            _ if self.menu.forgetting => return false,
            Pick::Word { plan, .. } | Pick::Email(plan) | Pick::Domain(plan) => {
                self.send(seed, &plan);
            }
            Pick::More => return false,
            Pick::Undo => self.undo(),
        }
        true
    }

    /// Picks row `row` of the open list's page: its email replaces the typed `@@`, and the list closes. A row past the
    /// page does nothing.
    pub(super) fn list_pick(&mut self, seed: Option<&Pack<'_>>, row: u8) {
        let Some(at) = self.menu.list.and_then(|list| list.at(row)) else {
            return;
        };
        let ask = self.show.as_ref().and_then(|s| s.emails.clone());
        let email = self.emails.list().get(at);
        let plan = match (ask, email) {
            (Some(ask @ EmailAsk::All(_)), Some(email)) => ask.plan(email),
            _ => None,
        };
        self.menu.cancel();
        if let Some(plan) = plan {
            self.send(seed, &plan);
        }
    }

    /// Brings back the word just forgotten, while its Undo chip shows.
    pub(super) fn undo(&mut self) {
        if self.menu.undo() && self.learner.as_mut().is_some_and(Learner::undo_forget) {
            self.changed(Blob::Learned);
        }
    }

    /// True when the lane lists every known email, so the list may open.
    pub(super) fn lists_all(&self) -> bool {
        let ask = self.show.as_ref().and_then(|s| s.emails.as_ref());
        matches!(ask, Some(EmailAsk::All(_)))
    }

    /// Asks the keyboard to type by `plan` as far as the box confirms it: all of it when the box still ends with what
    /// it erases, only the rest of the chip when the box cannot tell, nothing and a new read when the box moved on.
    fn send(&mut self, seed: Option<&Pack<'_>>, plan: &Plan) {
        let sent = match self.ends(plan) {
            Some(true) => Some(plan.clone()),
            Some(false) => {
                self.stats.moved += 1;
                let told = self.tracker.reset();
                self.act(seed, told);
                return;
            }
            None => plan.rest(),
        };
        let Some(sent) = sent else {
            self.stats.unsure += 1;
            return;
        };
        let word = TypeWord {
            round: self.round,
            erase: sent.erase,
            text: sent.text,
        };
        self.publish(&word);
        self.stats.typed += 1;
        self.pending = true;
    }

    /// Whether the box still ends with what `plan` erases, as a text box tells; `None` when another box, or the app,
    /// cannot tell. A box the watch already sees as another kind has moved on.
    fn ends(&mut self, plan: &Plan) -> Option<bool> {
        if self.moved() {
            return Some(false);
        }
        if plan.erase == 0 {
            return Some(true);
        }
        if self.kind != FieldKind::Text {
            return None;
        }
        self.ports.field.as_mut()?.ends_with(&plan.gone)
    }

    /// Forgets the word stored as `stored`, whose chip sat at `at`; its Undo chip takes the place.
    fn forget(&mut self, seed: Option<&Pack<'_>>, stored: &Redacted<String>, at: usize) {
        let day = self.ports.days.today();
        let learner = self.learner.as_mut();
        let gone = learner.is_some_and(|l| l.forget(seed, stored.expose(), day));
        self.menu.forgot(gone.then_some(at));
        if gone {
            self.stats.forgot += 1;
            self.changed(Blob::Learned);
        }
    }

    /// Removes the known `email` its chip types; forget mode ends.
    fn forget_email(&mut self, email: &Redacted<String>) {
        self.menu.forgot(None);
        if self.emails.remove(email.expose()) {
            self.stats.forgot += 1;
            self.changed(Blob::Emails);
        }
    }
}
