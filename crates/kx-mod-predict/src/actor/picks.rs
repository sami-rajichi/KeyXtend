//! Picks: a chip of the lane's round types through the keyboard with one `TypeWord`, after checking that a word read
//! from the box still ends it; in forget mode a word chip is forgotten instead, and the list's rows type their email.

use super::Actor;
use super::store::Blob;
use crate::lane::Pick;
use crate::learn::Learner;
use crate::pack::Pack;
use crate::track::{EmailAsk, Plan};
use kx_module_api::Redacted;
use kx_module_api::ports::{ChipPicked, TypeWord};

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
            _ if self.menu.forgetting => return false,
            Pick::Word { plan, .. } | Pick::Email(plan) => self.send(seed, &plan),
            Pick::More => return false,
            Pick::Undo => self.undo(),
        }
        true
    }

    /// Picks row `row` of the open list's page: its email replaces the typed `@@`, and the list closes.
    pub(super) fn list_pick(&mut self, seed: Option<&Pack<'_>>, row: u8) {
        let Some(list) = self.menu.list else {
            return;
        };
        let ask = self.show.as_ref().and_then(|s| s.emails.clone());
        let email = self.emails.list().get(list.at(row));
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

    /// Asks the keyboard to erase and type by `plan`; a box that no longer ends with a read word gets nothing and a
    /// new read.
    fn send(&mut self, seed: Option<&Pack<'_>>, plan: &Plan) {
        if !self.still_ends(plan) {
            self.stats.moved += 1;
            let told = self.tracker.reset();
            self.act(seed, told);
            return;
        }
        let word = TypeWord {
            round: self.round,
            erase: plan.erase,
            text: plan.text.clone(),
        };
        self.publish(&word);
        self.stats.typed += 1;
        self.pending = true;
    }

    /// True when the box surely ends with what `plan` erases: a word typed here always does, a read one only when
    /// the box says so.
    fn still_ends(&mut self, plan: &Plan) -> bool {
        let read = self.show.as_ref().is_some_and(|s| s.read);
        if !read || plan.erase == 0 {
            return true;
        }
        let run = self.ports.field.as_mut();
        run.and_then(|run| run.ends_with(&plan.gone)) == Some(true)
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
}
