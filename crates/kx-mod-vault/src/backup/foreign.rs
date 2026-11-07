//! A box another computer wrote, in a place picked at setup or Add place (design "Bring back and padlocks"): it is
//! never written over unasked. Yes asks that box's password and brings its entries in, as Bring back does; No skips
//! the place.

use kx_module_api::Redacted;
use kx_module_api::ports::{Place, When, box_file};

use super::bring::{Picked, Source};
use super::flow::{Flow, Then};
use super::state::{Backup, Flowed};

impl Backup {
    /// Asks about the box from another computer in `place`, written `when` if told, picked for `then`.
    pub(super) fn foreign(&mut self, place: Place, when: Option<When>, then: Then) -> Flowed {
        self.flow = Some(Flow::Foreign { place, when, then });
        Flowed::Show
    }

    /// Yes to that box: its password, to bring its entries in; its place comes last among the places.
    pub(super) fn foreign_yes(&mut self) -> Flowed {
        let Some(Flow::Foreign { place, then, .. }) = &self.flow else {
            return Flowed::Stay;
        };
        let (place, then) = (place.clone(), *then);
        let file = Redacted::new(place.path.expose().join(box_file()));
        let picked = Picked::Foreign(then);
        self.ask_password(Source {
            file,
            place,
            picked,
        });
        Flowed::Show
    }
}
