//! The notebook's emails (Quick-fill's `VaultEmails`): held only while prediction is on and asked for at start and at
//! turn-on. A saved pick marks the token the keyboard types it into, so nothing in that token is learned.

use super::Actor;
use crate::email::{Offer, Saved};
use kx_module_api::ports::{FillAsk, Piece};
use kx_module_api::{Redacted, Secret, Zeroizing};

impl Actor {
    /// Takes the notebook's `emails` in place of the last set, which is wiped as it goes, and none while prediction is
    /// off; true when a new set meets a lane that shows emails, so it is told again.
    pub(super) fn notebook(&mut self, emails: Vec<Secret>) -> bool {
        if !self.settings.on {
            return false;
        }
        let saved = Saved::new(emails);
        // Quick-fill tells its state at its start and at each ask, so the same set again changes nothing.
        if saved == self.saved {
            return false;
        }
        self.saved = saved;
        self.relist();
        self.show.as_ref().is_some_and(|s| s.emails.is_some())
    }

    /// Keeps the open list on a page the offered emails fill, and closes it when none is left.
    fn relist(&mut self) {
        let Some(list) = self.menu.list else {
            return;
        };
        let count = self.listed().len();
        if count == 0 {
            self.menu.cancel();
        } else {
            self.menu.turn(0, list.rows, count);
        }
    }

    /// Asks Quick-fill to say its state again, the notebook's emails among it, while prediction is on.
    pub(super) fn ask_saved(&self) {
        if self.settings.on {
            self.publish(&FillAsk);
        }
    }

    /// Lets the notebook's emails and a waiting saved pick go, wiped, so prediction off holds none.
    pub(super) fn drop_saved(&mut self) {
        (self.saved, self.noted) = (Saved::default(), None);
    }

    /// The emails offered now, by forget mode's one rule.
    pub(super) fn offer(&self) -> Offer<'_> {
        Offer::shown(&self.emails, &self.saved, self.menu.forgetting)
    }

    /// Notes `sent`, the text a saved pick sent, until the keyboard tells it, another pick or turn-off.
    pub(super) fn note_saved(&mut self, sent: Option<Redacted<String>>) {
        self.noted = sent.map(|s| Redacted::new(Zeroizing::new(s.into_inner())));
    }

    /// True when `piece` is the text the saved pick sent last, wherever focus went since, which ends that wait; any
    /// other piece leaves it, so a racing key cannot unmark a late echo.
    pub(super) fn echoed(&mut self, piece: &Piece) -> bool {
        let Piece::Text(text) = piece else {
            return false;
        };
        self.noted
            .take_if(|sent| sent.expose().as_str() == text.expose().as_str())
            .is_some()
    }
}
