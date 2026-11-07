//! The lists a place is picked from (design "The backup box"): the suggested places, or Another folder and the paged
//! folder list, drives first. Each list is read on the job thread, as a sleeping drive may take long; its page shows
//! at once and fills when the list comes, and a folder that does not list steps back with its note.

use kx_module_api::ports::{BOX_EXT, FillNote, Place, PlaceError, PlaceKind};

use super::flow::{Flow, Then};
use super::jobs::Job;
use super::state::{Backup, Flowed, Purpose};
use crate::import::keepass_ext;

impl Backup {
    /// The suggested places, to pick one for `then`.
    pub(super) fn places(&mut self, then: Then) -> Flowed {
        self.flow = Some(Flow::Places {
            at: 0,
            spots: Vec::new(),
            then,
        });
        let round = self.post(Job::Suggested);
        self.asked = Some((round, Purpose::Listing));
        Flowed::Show
    }

    /// The folder list at the end of `trail`, or the drives, to pick one for `then`; Bring back lists backups too, and
    /// Import `KeePass` files.
    pub(super) fn folders_at(&mut self, trail: Vec<Place>, then: Then) -> Flowed {
        let folder = trail.last().map(|p| p.path.clone());
        self.flow = Some(Flow::Folders {
            trail,
            rows: Vec::new(),
            at: 0,
            then,
        });
        let files = match then {
            Then::Bring => Some(BOX_EXT),
            Then::Import => Some(keepass_ext()),
            Then::Setup | Then::Add | Then::Change => None,
        };
        let round = self.post(Job::Folders { folder, files });
        self.asked = Some((round, Purpose::Listing));
        Flowed::Show
    }

    /// Opens the folder list's row `i`: a drive or a folder lists, and a file is picked, to bring back or to import.
    pub(super) fn open_folder(&mut self, i: usize) -> Flowed {
        let Some(Flow::Folders {
            trail, rows, then, ..
        }) = &self.flow
        else {
            return Flowed::Stay;
        };
        let Some(next) = rows.get(i).cloned() else {
            return Flowed::Stay;
        };
        let (mut trail, then) = (trail.clone(), *then);
        if next.kind == PlaceKind::File {
            return if then == Then::Import {
                self.pick_kdbx(trail, next)
            } else {
                self.pick_file(trail, next)
            };
        }
        trail.push(next);
        self.folders_at(trail, then)
    }

    /// Takes the suggested places of `round`, for the page that waits for them.
    pub(super) fn suggested(&mut self, round: u32, places: Vec<Place>) -> Flowed {
        if self.answered(round).is_none() {
            return Flowed::Stay;
        }
        match &mut self.flow {
            Some(Flow::Places { spots, .. }) => *spots = places,
            _ => return Flowed::Stay,
        }
        Flowed::Show
    }

    /// Takes the folder list of `round`; one that did not list steps back as Back does, with its note.
    pub(super) fn listed(&mut self, round: u32, listed: Result<Vec<Place>, PlaceError>) -> Flowed {
        if self.answered(round).is_none() {
            return Flowed::Stay;
        }
        let Some(Flow::Folders { rows, then, .. }) = &mut self.flow else {
            return Flowed::Stay;
        };
        let error = match listed {
            Ok(listed) => {
                *rows = listed;
                return Flowed::Show;
            }
            Err(error) => error,
        };
        tracing::warn!(%error, "a folder did not list");
        let note = list_note(*then);
        let back = self.back();
        self.note = Some(note);
        back
    }
}

/// The note of a list that failed for `then`: Import and Bring back only read, and the others pick a place to write.
fn list_note(then: Then) -> FillNote {
    match then {
        Then::Import => FillNote::FileUnread,
        Then::Bring => FillNote::Unread,
        Then::Setup | Then::Add | Then::Change => FillNote::ReadOnly,
    }
}
