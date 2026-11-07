//! The backup pages as the panel draws them: places by kind and short name, never by path, each box's state and date,
//! the folder list, the password page, which shows what is typed (owner), a box from another computer and the boxes
//! found.

use kx_module_api::Redacted;
use kx_module_api::ports::{
    BackupAsk, BackupSheet, BoxState, FillNote, FolderSheet, Found, FoundRow, LocalTime,
    NoteCounts, PageNote, PasswordAsk, PasswordSheet, Pick, Place, PlaceRow, Sheet, When,
};

use super::Site;
use super::bring::{Picked, Source};
use super::flow::{Asking, Flow, Goal, Then, when};
use super::state::{Backup, spot};
use crate::entry::Stamp;
use crate::panel::copy;
use crate::view::{Paged, Window, small};

impl Backup {
    /// The shown backup page over `rows` a page at wall time `now`, noting where its list starts; `None` while none
    /// shows.
    pub(crate) fn draw(&mut self, rows: usize, now: &LocalTime) -> Option<Paged> {
        let rows = rows.max(1);
        let len = self.len();
        let at = self.cursor().map(|a| *a);
        let window = Window::of(at, len, rows, at.is_some());
        self.start = window.start;
        let sheet = self.sheet(&window, now)?;
        let note = self.note.map(|note| self.page_note(note));
        Some(Paged {
            sheet,
            window,
            note,
        })
    }

    /// The shown page's sheet over `w`.
    fn sheet(&self, w: &Window, now: &LocalTime) -> Option<Sheet> {
        let sheet = match self.flow.as_ref()? {
            Flow::Home { asking, .. } => {
                let shown = &self.kept.sites[w.start..w.end];
                Sheet::Backup(BackupSheet {
                    places: shown.iter().map(|s| row(s, now)).collect(),
                    checks: self.checks,
                    ask: asking.and_then(|a| ask(a, w)),
                })
            }
            Flow::Places { spots, .. } => {
                Sheet::Places(spots[w.start..w.end].iter().map(spot).collect())
            }
            Flow::Folders {
                trail, rows, then, ..
            } => Sheet::Folders(FolderSheet {
                pick: picks(*then),
                folder: trail.last().map(|p| p.name.clone()),
                rows: rows[w.start..w.end].iter().map(spot).collect(),
            }),
            Flow::Password { line, goal } => Sheet::Password(PasswordSheet {
                ask: self.asks(goal),
                text: copy(line.text()),
                caret: Redacted::new(line.drawn()),
            }),
            Flow::Foreign { place, when, .. } => foreign(place, *when),
            Flow::Found { rows, .. } => {
                let shown = rows[w.start..w.end].iter();
                Sheet::Found(shown.map(|f| found(f, now)).collect())
            }
        };
        Some(sheet)
    }

    /// What the password page asks for `goal`.
    fn asks(&self, goal: &Goal) -> PasswordAsk {
        match goal {
            Goal::New { .. } => PasswordAsk::New {
                min: u8::try_from(self.cfg.min_chars).unwrap_or(u8::MAX),
            },
            Goal::Check => PasswordAsk::Check {
                tries: self.tries_left(),
            },
            Goal::Open(Source {
                picked: Picked::Foreign(_),
                ..
            }) => PasswordAsk::Foreign,
            Goal::Open(_) => PasswordAsk::Open {
                own: self.kept.padlock.is_some(),
            },
            Goal::Keepass(_) => PasswordAsk::Keepass,
        }
    }

    /// `note` as the page shows it: only `Imported` names numbers, what a Bring back or an import brought in.
    pub(super) fn page_note(&self, note: FillNote) -> PageNote {
        let counts = match note {
            FillNote::Imported => self.counts,
            _ => NoteCounts::default(),
        };
        PageNote { note, counts }
    }
}

/// A found box's row: its place and when it was written.
fn found(found: &Found, now: &LocalTime) -> FoundRow {
    FoundRow {
        place: spot(&found.place),
        when: when(Stamp::from_unix_ms(found.modified_ms), now),
    }
}

/// The Backup page's question as `w` shows it: Remove place on its row while that row shows, or Replace.
fn ask(asking: Asking, w: &Window) -> Option<BackupAsk> {
    match asking {
        Asking::Remove(i) if (w.start..w.end).contains(&i) => {
            Some(BackupAsk::Remove(small(i - w.start)))
        }
        Asking::Remove(_) => None,
        Asking::Replace => Some(BackupAsk::Replace),
    }
}

/// A place's row: Updated with its time once its box is written and not behind, else Waiting.
fn row(site: &Site, now: &LocalTime) -> PlaceRow {
    let state = match site.written.filter(|_| !site.waiting) {
        Some(at) => BoxState::Updated(when(at, now)),
        None => BoxState::Waiting,
    };
    PlaceRow {
        place: spot(&site.place),
        state,
    }
}

/// What the folder list picks for `then`: a backup file for Bring back, a `KeePass` file for Import, else a folder.
fn picks(then: Then) -> Pick {
    match then {
        Then::Bring => Pick::Backup,
        Then::Import => Pick::Keepass,
        Then::Setup | Then::Add | Then::Change => Pick::Folder,
    }
}

/// The question about the box from another computer in `place`, with its date when the system tells it.
fn foreign(place: &Place, when: Option<When>) -> Sheet {
    let place = spot(place);
    match when {
        Some(when) => Sheet::Foreign(FoundRow { place, when }),
        None => Sheet::ForeignUndated(place),
    }
}
