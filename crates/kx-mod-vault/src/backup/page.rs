//! The Backup page (design "Backup page"): each place's state, Back up now, Add place, Remove place, which asks once
//! and leaves the box in place, Change password, Bring back, Import, and the memory checks switch, which writes the
//! module's own `memory_checks` setting through the kernel. With no padlock the Backup button opens setup instead.

use kx_module_api::ports::FillButton;

use super::flow::{Asking, Flow, Goal, Then};
use super::keepass::Came;
use super::state::{At, Backup, Flowed};
use crate::book::Notebook;
use crate::entry::Line;
use crate::settings::MEMORY_CHECKS;

impl Backup {
    /// The Backup button: setup while there is no padlock, else the Backup page.
    pub(super) fn open(&mut self, _: &At<'_>) -> Flowed {
        (self.asked, self.note, self.bring.offer) = (None, None, false);
        if self.kept.padlock.is_none() {
            return self.setup();
        }
        self.home();
        Flowed::Show
    }

    /// Row `i` of the shown list: the Backup page asks to remove that place, a place, a file or a found box is
    /// picked, and a drive or a folder opens.
    pub(super) fn row(&mut self, i: usize, _: &At<'_>) -> Flowed {
        let len = self.len();
        match &mut self.flow {
            Some(Flow::Home { at, asking }) if i < len => {
                (*at, *asking) = (i, Some(Asking::Remove(i)));
                Flowed::Show
            }
            Some(Flow::Places { spots, then, .. }) => match spots.get(i).cloned() {
                Some(place) => {
                    let then = *then;
                    self.pick(place, then)
                }
                None => Flowed::Stay,
            },
            Some(Flow::Folders { .. }) => self.open_folder(i),
            Some(Flow::Found { .. }) => self.pick_found(i),
            _ => Flowed::Stay,
        }
    }

    /// A panel button on the shown backup page.
    pub(super) fn button(&mut self, button: FillButton, at: &At<'_>) -> Flowed {
        match (&self.flow, button) {
            (Some(Flow::Home { .. }), FillButton::BackUpNow) => self.back_up_now(at),
            (Some(Flow::Home { .. }), FillButton::AddPlace) => self.places(Then::Add),
            (Some(Flow::Home { .. }), FillButton::ChangePassword) => self.change_password(),
            (Some(Flow::Home { .. }), FillButton::BringBack) => self.bring_from(None),
            (Some(Flow::Places { then, .. }), FillButton::BringBack) => {
                let then = Some(*then);
                self.bring_from(then)
            }
            (Some(Flow::Home { .. }), FillButton::Import) => self.import_from(Came::Home),
            (Some(Flow::Places { then, .. }), FillButton::Import) => {
                let then = *then;
                self.import_from(Came::Places(then))
            }
            (Some(Flow::Places { then, .. }), FillButton::OtherFolder) => {
                let then = *then;
                self.folders_at(Vec::new(), then)
            }
            (Some(Flow::Found { .. }), FillButton::OtherFolder) => {
                self.folders_at(Vec::new(), Then::Bring)
            }
            (Some(Flow::Folders { .. }), FillButton::Here) => self.here(),
            (Some(Flow::Password { .. }), FillButton::Done) => self.done_typing(),
            _ => Flowed::Stay,
        }
    }

    /// Change password: the new password page, whose padlock rewrites every box.
    fn change_password(&mut self) -> Flowed {
        let line = Line::new(self.cfg.field_chars);
        let goal = Goal::New {
            place: None,
            then: Then::Change,
        };
        self.flow = Some(Flow::Password { line, goal });
        Flowed::Show
    }

    /// Use this folder: picks the folder opened, unless the list picks a file.
    fn here(&mut self) -> Flowed {
        let Some(Flow::Folders { trail, then, .. }) = &self.flow else {
            return Flowed::Stay;
        };
        match (trail.last(), *then) {
            (_, Then::Bring | Then::Import) | (None, _) => Flowed::Stay,
            (Some(place), then) => {
                let place = place.clone();
                self.pick(place, then)
            }
        }
    }

    /// Answers the shown question: Yes removes the place asked about, replaces the backup while held, or asks the
    /// password of a box from another computer.
    pub(super) fn answer(&mut self, yes: bool, at: &At<'_>) -> Flowed {
        if !yes {
            return self.no();
        }
        let asking = match &self.flow {
            Some(Flow::Home { asking, .. }) => *asking,
            Some(Flow::Foreign { .. }) => return self.foreign_yes(),
            _ => None,
        };
        match asking {
            Some(Asking::Remove(i)) => self.remove(i),
            Some(Asking::Replace) => {
                self.release(at.book, at.now);
                self.home();
                Flowed::Show
            }
            None => Flowed::Stay,
        }
    }

    /// No to the shown question: the Backup page's goes away, and a box from another computer skips its place.
    pub(super) fn no(&mut self) -> Flowed {
        match &mut self.flow {
            Some(Flow::Home {
                asking: asking @ Some(_),
                ..
            }) => {
                *asking = None;
                Flowed::Show
            }
            Some(Flow::Foreign { then, .. }) => {
                let then = *then;
                self.places(then)
            }
            _ => Flowed::Stay,
        }
    }

    /// Removes place `i`, leaving its box there; its mark stays, so adding it again knows the box as this computer's.
    fn remove(&mut self, i: usize) -> Flowed {
        if i >= self.kept.sites.len() {
            return Flowed::Stay;
        }
        let site = self.kept.sites.remove(i);
        if let Some(mark) = site.mark.filter(|m| !self.kept.left.contains(m)) {
            self.kept.left.push(mark);
        }
        self.save();
        let last = self.kept.sites.len().saturating_sub(1);
        self.flow = Some(Flow::Home {
            at: i.min(last),
            asking: None,
        });
        Flowed::Show
    }

    /// True once, when `book` holds its first entry and no padlock is made: the first offer asks, and never again. It
    /// waits while setup shows after an import from its places, for the next save if setup is left.
    pub(crate) fn offers(&mut self, book: &Notebook) -> bool {
        let on_setup = self.flow.is_some() && self.import.came == Came::Places(Then::Setup);
        if self.kept.offered || self.kept.padlock.is_some() || book.is_empty() || on_setup {
            return false;
        }
        self.kept.offered = true;
        self.save();
        true
    }

    /// The memory checks switch, on the Backup page: it writes `memory_checks`, a refused write keeps the switch, and
    /// off forgets a banner that asks, so on asks again.
    pub(super) fn switch(&mut self, on: bool) -> Flowed {
        if !matches!(self.flow, Some(Flow::Home { .. })) || on == self.checks {
            return Flowed::Stay;
        }
        match self
            .links
            .handle
            .set(MEMORY_CHECKS, toml::Value::Boolean(on))
        {
            Ok(()) => (self.checks, self.raised) = (on, self.raised && on),
            Err(error) => tracing::warn!(%error, "the memory checks switch was not saved"),
        }
        Flowed::Show
    }
}
