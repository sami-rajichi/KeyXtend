//! Picks: a chip of the lane's round types through the keyboard with one `TypeWord`, erasing only what differs once
//! the box confirms it; in forget mode a word or known-email chip is forgotten instead, with an Undo chip in its
//! place, and the list's rows type their email.

use super::Actor;
use super::store::Blob;
use crate::email::Emails;
use crate::lane::{self, Pick, fits};
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
            Pick::Email(plan) if self.menu.forgetting => self.forget_email(&plan.chip(), at),
            _ if self.menu.forgetting => return false,
            Pick::Word { plan, .. } | Pick::Email(plan) => {
                self.send(seed, &plan);
            }
            Pick::Domain(plan) => self.domain = self.send(seed, &plan),
            Pick::More => return false,
            Pick::Undo => self.undo(),
        }
        true
    }

    /// Picks row `row` of the open list's page: its email replaces the typed `@@`, and the list closes. A row with no
    /// email does nothing, as does any row while the lane is held empty.
    pub(super) fn list_pick(&mut self, seed: Option<&Pack<'_>>, row: u8) {
        let at = self.menu.list.and_then(|list| list.at(row));
        let Some(at) = at.filter(|_| !self.held()) else {
            return;
        };
        let ask = self.show.as_ref().and_then(|s| s.emails.clone());
        let listed = self.listed();
        let plan = match (ask, listed.list().get(at)) {
            (Some(ask @ EmailAsk::All(_)), Some(email)) => ask.plan(email),
            _ => None,
        };
        if let Some(plan) = plan {
            self.menu.cancel();
            self.send(seed, &plan);
        }
    }

    /// Brings back the word or email just forgotten, while its Undo chip shows.
    pub(super) fn undo(&mut self) {
        if !self.menu.undo() {
            return;
        }
        if let Some((at, email)) = self.lost.take() {
            self.emails.put(at, email.into_inner());
            self.changed(Blob::Emails);
        } else if self.learner.as_mut().is_some_and(Learner::undo_forget) {
            self.changed(Blob::Learned);
        }
    }

    /// True when the open `@@` can send a known email, so the list may open; never in forget mode.
    pub(super) fn lists_all(&self) -> bool {
        !self.menu.forgetting && !self.listed().list().is_empty()
    }

    /// The known emails the open `@@` can send, which the list pages; none without one.
    pub(super) fn listed(&self) -> Emails {
        match self.show.as_ref().and_then(|s| s.emails.as_ref()) {
            Some(ask @ EmailAsk::All(_)) => {
                lane::listed(&self.emails, ask, self.mode, self.confirms())
            }
            _ => Emails::default(),
        }
    }

    /// True when the box confirms what a pick erases: a text box whose last read gave text and whose checks since could
    /// tell.
    pub(super) fn confirms(&self) -> bool {
        self.kind == FieldKind::Text && !self.blind
    }

    /// Asks the keyboard to type by `plan` as far as the box confirms it, and gives the text asked for: all of it when
    /// the box still ends with what the chip replaces, a plan that erases nothing when the box cannot tell, nothing and
    /// a new read when it moved on. A plan past the keyboard's text cap sends nothing, as the keyboard would refuse it.
    fn send(&mut self, seed: Option<&Pack<'_>>, plan: &Plan) -> Option<Redacted<String>> {
        if !fits(plan, self.mode) {
            self.stats.long += 1;
            return None;
        }
        let sent = match self.ends(plan) {
            Some(true) => Some(plan.clone()),
            Some(false) => {
                self.stats.moved += 1;
                let told = self.tracker.reset();
                self.act(seed, told);
                return None;
            }
            None => plan.rest(),
        };
        let Some(sent) = sent else {
            self.stats.unsure += 1;
            return None;
        };
        let word = TypeWord {
            round: self.round,
            erase: sent.erase,
            text: sent.text.clone(),
        };
        self.publish(&word);
        self.stats.sent += 1;
        self.hold = true;
        Some(sent.text)
    }

    /// Whether the box still ends with what `plan` replaces, as a text box tells, even when the plan replaces nothing,
    /// so the kind its check sees wins for every pick; `None` when another box, or the app, cannot tell, which leaves a
    /// text box blind. A box that cannot confirm never confirms an erase, and a selection's no holds the lane empty.
    fn ends(&mut self, plan: &Plan) -> Option<bool> {
        if self.moved() {
            return Some(false);
        }
        if self.kind != FieldKind::Text {
            return plan.gone.expose().is_empty().then_some(true);
        }
        let confirms = self.confirms();
        let checked = self.ports.field.as_mut()?.ends_with(&plan.gone);
        if self.seen(checked.kind) {
            return Some(false);
        }
        self.blind |= checked.ends.is_none();
        self.hold |= checked.selected;
        checked.ends.filter(|&ends| confirms || !ends)
    }

    /// Forgets the word stored as `stored`, whose chip sat at `at`; its Undo chip takes the place.
    fn forget(&mut self, seed: Option<&Pack<'_>>, stored: &Redacted<String>, at: usize) {
        let day = self.ports.days.today();
        let learner = self.learner.as_mut();
        let gone = learner.is_some_and(|l| l.forget(seed, stored.expose(), day));
        self.menu.forgot(gone.then_some(at));
        if gone {
            self.lost = None;
            self.stats.forgot += 1;
            self.changed(Blob::Learned);
        }
    }

    /// Removes the known `email` its chip, at `at`, types; its Undo chip takes the place.
    fn forget_email(&mut self, email: &Redacted<String>, at: usize) {
        let removed = self.emails.remove(email.expose());
        self.menu.forgot(removed.is_some().then_some(at));
        if let Some((place, email)) = removed {
            self.lost = Some((place, Redacted::new(email)));
            self.stats.forgot += 1;
            self.changed(Blob::Emails);
        }
    }
}
