//! Bring back (design "New computer", "Bring back and padlocks"): the boxes found in the suggested places, newest
//! first, or a backup file from the folder list; its place is looked at, then its password asked, and the box is read
//! within `box_mb` and opened in the worker (`given`). Back from its first page returns to the page it came from.

use std::path::{Path, PathBuf};

use kx_module_api::Redacted;
use kx_module_api::ports::{FillNote, Found, Place, PlaceKind, PlaceState};

use super::flow::{Flow, Goal, Then};
use super::given::Given;
use super::jobs::Job;
use super::state::{Backup, Flowed, Purpose};
use crate::book::Notebook;
use crate::entry::Line;

/// Where a box to open was picked, for Back.
#[derive(Clone, Debug)]
pub(crate) enum Picked {
    /// From the boxes found.
    Found,
    /// From the folder list, at the end of this trail.
    Folder(Vec<Place>),
    /// In a place picked for this, where another computer wrote it.
    Foreign(Then),
}

/// A box to open: its file, the place that holds it, and where it was picked.
#[derive(Clone, Debug)]
pub(crate) struct Source {
    pub(super) file: Redacted<PathBuf>,
    pub(super) place: Place,
    pub(super) picked: Picked,
}

/// Bring back's runtime.
#[derive(Debug, Default)]
pub(crate) struct Bring {
    /// The boxes found showed by themselves, so Back leaves them.
    pub(super) offer: bool,
    /// They showed so this run, so they never show so again.
    pub(super) offered: bool,
    /// The places page Bring back came from; `None` for the Backup page.
    pub(super) from: Option<Then>,
    /// The box opened, until the actor knows whether its entries fit.
    pub(super) given: Option<Given>,
}

impl Backup {
    /// The Bring back button, on setup's or Add place's places (`from`) or on the Backup page.
    pub(super) fn bring_from(&mut self, from: Option<Then>) -> Flowed {
        self.bring.from = from;
        self.bring()
    }

    /// Bring back's first page: the boxes found, or the folder list when none is.
    pub(super) fn bring(&mut self) -> Flowed {
        let rows = Vec::new();
        self.flow = Some(Flow::Found { at: 0, rows });
        let round = self.post(Job::Find);
        self.asked = Some((round, Purpose::Find { offer: false }));
        Flowed::Show
    }

    /// As the panel opens on an empty notebook with no backup of its own, or a held one, the boxes found offer
    /// themselves, until they have shown once this run.
    pub(crate) fn offer(&mut self, book: &Notebook) {
        let own = self.kept.padlock.is_some() && !self.kept.held;
        let busy = self.asked.is_some() || self.flow.is_some();
        if !book.is_empty() || own || busy || self.bring.offered {
            return;
        }
        let round = self.post(Job::Find);
        self.asked = Some((round, Purpose::Find { offer: true }));
    }

    /// The boxes found showed by themselves, so they never do so again this run.
    pub(crate) fn offer_shown(&mut self) {
        self.bring.offered = true;
    }

    /// Takes the boxes found of `round`, newest first: the offer shows them, if any; Bring back shows them, or the
    /// folder list when there are none.
    pub(super) fn found(&mut self, round: u32, found: Vec<Found>) -> Flowed {
        let Some(Purpose::Find { offer }) = self.answered(round) else {
            return Flowed::Stay;
        };
        if offer {
            if found.is_empty() {
                return Flowed::Stay;
            }
            self.bring.offer = true;
            self.flow = Some(Flow::Found { at: 0, rows: found });
            return Flowed::Offer;
        }
        if found.is_empty() {
            return self.folders_at(Vec::new(), Then::Bring);
        }
        match &mut self.flow {
            Some(Flow::Found { rows, .. }) => *rows = found,
            _ => return Flowed::Stay,
        }
        Flowed::Show
    }

    /// Picks the found box at `i`.
    pub(super) fn pick_found(&mut self, i: usize) -> Flowed {
        let Some(Flow::Found { rows, .. }) = &self.flow else {
            return Flowed::Stay;
        };
        let Some(found) = rows.get(i) else {
            return Flowed::Stay;
        };
        let (file, place) = (found.path.clone(), home_of(found));
        let picked = Picked::Found;
        self.look_first(Source {
            file,
            place,
            picked,
        })
    }

    /// Picks the backup `file` in the folder at the end of `trail`.
    pub(super) fn pick_file(&mut self, trail: Vec<Place>, file: Place) -> Flowed {
        let Some(place) = trail.last().cloned() else {
            return Flowed::Stay;
        };
        let (file, picked) = (file.path, Picked::Folder(trail));
        self.look_first(Source {
            file,
            place,
            picked,
        })
    }

    /// Looks at `source`'s place before its password, as a data folder pick is refused first.
    fn look_first(&mut self, source: Source) -> Flowed {
        let folder = source.place.path.clone();
        let round = self.post(Job::State { folder });
        self.asked = Some((round, Purpose::Ready(source)));
        Flowed::Show
    }

    /// Takes the state of a picked box's place: the data folder is refused, one gone says so, and any other asks the
    /// box's password.
    pub(super) fn readied(&mut self, round: u32, state: PlaceState) -> Flowed {
        let Some(Purpose::Ready(source)) = self.answered(round) else {
            return Flowed::Stay;
        };
        match state {
            PlaceState::Refused => self.note = Some(FillNote::DataFolder),
            PlaceState::Missing => self.note = Some(FillNote::Unread),
            PlaceState::Ready | PlaceState::Full => self.ask_password(source),
        }
        Flowed::Show
    }

    /// The password page for the box `source`.
    pub(super) fn ask_password(&mut self, source: Source) {
        let line = Line::new(self.cfg.field_chars);
        let goal = Goal::Open(source);
        self.flow = Some(Flow::Password { line, goal });
    }
}

/// The place a found box is in: the suggested place itself, or the folder one level down, named by its last part.
fn home_of(found: &Found) -> Place {
    let folder = found.path.expose().parent().map(Path::to_path_buf);
    let folder = folder.unwrap_or_default();
    if folder == *found.place.path.expose() {
        return found.place.clone();
    }
    let name = folder.file_name().map(|n| n.to_string_lossy().into_owned());
    Place {
        kind: PlaceKind::Folder,
        name: Redacted::new(name.unwrap_or_default()),
        path: Redacted::new(folder),
    }
}
