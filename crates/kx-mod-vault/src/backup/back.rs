//! Back on the backup pages: one page back, cancelling the job waited for. A list's top goes where it came from, a
//! password page to where its place or box was picked, a memory check leaves as Later did, and the Backup page leaves.

use kx_module_api::ports::Place;

use super::bring::Picked;
use super::flow::{Flow, Goal, Then};
use super::state::{Backup, Flowed};

/// Where Back goes.
enum To {
    /// The Backup page.
    Home,
    /// The suggested places, to pick one for this.
    Places(Then),
    /// The folder list at the end of this trail.
    Folders(Vec<Place>, Then),
    /// Bring back's boxes found, found again.
    Found,
    /// Out of Bring back's first page.
    Out,
    /// Where Import was pressed.
    Came,
    /// Out of the backup pages, to the list.
    List,
}

impl Backup {
    /// Back: one page back, cancelling the job waited for.
    pub(super) fn back(&mut self) -> Flowed {
        self.asked = None;
        match self.back_to() {
            To::Home => {
                self.home();
                Flowed::Show
            }
            To::Places(then) => self.places(then),
            To::Folders(trail, then) => self.folders_at(trail, then),
            To::Found => self.bring(),
            To::Out => self.out(),
            To::Came => self.came_back(),
            To::List => {
                self.flow = None;
                Flowed::Leave
            }
        }
    }

    /// Where Back goes from the shown page.
    fn back_to(&self) -> To {
        let Some(flow) = &self.flow else {
            return To::List;
        };
        match flow {
            Flow::Folders { trail, then, .. } => {
                let mut trail = trail.clone();
                match (trail.pop(), *then) {
                    (Some(_), then) => To::Folders(trail, then),
                    (None, Then::Bring) => To::Out,
                    (None, Then::Import) => To::Came,
                    (None, then) => To::Places(then),
                }
            }
            Flow::Places {
                then: Then::Add, ..
            } => To::Home,
            Flow::Password { goal, .. } => Self::password_back(goal),
            Flow::Foreign { then, .. } => To::Places(*then),
            Flow::Found { .. } => To::Out,
            Flow::Places { .. } | Flow::Home { .. } => To::List,
        }
    }

    /// Where Back goes from a password page for `goal`.
    fn password_back(goal: &Goal) -> To {
        match goal {
            Goal::New {
                then: Then::Change, ..
            } => To::Home,
            Goal::New { then, .. } => To::Places(*then),
            Goal::Check => To::List,
            Goal::Open(source) => match &source.picked {
                Picked::Found => To::Found,
                Picked::Folder(trail) => To::Folders(trail.clone(), Then::Bring),
                Picked::Foreign(then) => To::Places(*then),
            },
            Goal::Keepass(file) => To::Folders(file.trail.clone(), Then::Import),
        }
    }

    /// Out of Bring back's first page: the list when it showed by itself, else the page it came from.
    pub(super) fn out(&mut self) -> Flowed {
        if std::mem::take(&mut self.bring.offer) {
            self.flow = None;
            return Flowed::Leave;
        }
        match self.bring.from {
            Some(then) => self.places(then),
            None if self.kept.padlock.is_none() => self.places(Then::Setup),
            None => {
                self.home();
                Flowed::Show
            }
        }
    }
}
