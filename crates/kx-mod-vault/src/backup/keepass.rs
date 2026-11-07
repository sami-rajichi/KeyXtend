//! Import's pages (design "Import"): Import, pressed on the invitation, the Backup page or a places page,
//! opens the folder list for a `KeePass` file, then its password; the file is read and opened on the job thread. The
//! actor brings the entries in (`crate::import`), and the result goes back where Import was pressed.

use std::path::PathBuf;

use kx_module_api::Redacted;
use kx_module_api::ports::{BoxError, FillNote, KeepassImport, NoteCounts, Place, PlaceError};

use super::flow::{Flow, Goal, Then};
use super::jobs::Shut;
use super::state::{Backup, Flowed, Purpose};
use crate::entry::Line;

/// Where Import was pressed.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum Came {
    /// The empty notebook's invitation, on the list.
    #[default]
    List,
    /// The Backup page.
    Home,
    /// The places page of this flow.
    Places(Then),
}

/// A `KeePass` file picked: the file, and the folders opened to it.
#[derive(Clone, Debug)]
pub(crate) struct Kdbx {
    pub(super) file: Redacted<PathBuf>,
    pub(super) trail: Vec<Place>,
}

/// Import's runtime.
#[derive(Debug, Default)]
pub(crate) struct Import {
    /// Where Import was pressed, for Back and the result.
    pub(super) came: Came,
    /// What the file gave, until the actor takes it.
    read: Option<KeepassImport>,
}

impl Backup {
    /// Import, pressed at `came`: the folder list from the drives.
    pub(super) fn import_from(&mut self, came: Came) -> Flowed {
        (self.note, self.bring.offer) = (None, false);
        self.import.came = came;
        self.folders_at(Vec::new(), Then::Import)
    }

    /// Picks the `KeePass` `file` in the folder at the end of `trail`: its password comes next.
    pub(super) fn pick_kdbx(&mut self, trail: Vec<Place>, file: Place) -> Flowed {
        let line = Line::new(self.cfg.field_chars);
        let file = file.path;
        let goal = Goal::Keepass(Kdbx { file, trail });
        self.flow = Some(Flow::Password { line, goal });
        Flowed::Show
    }

    /// Takes what the file of `round` gave: a refusal says why and the password page stays, so the user can try again;
    /// else its entries wait for the actor.
    pub(super) fn kdbx_read(&mut self, round: u32, read: Result<KeepassImport, Shut>) -> Flowed {
        let Some(Purpose::Keepass) = self.answered(round) else {
            return Flowed::Stay;
        };
        match read {
            Ok(import) => {
                self.import.read = Some(import);
                Flowed::Keepass
            }
            Err(shut) => {
                self.note = Some(shut_note(shut));
                Flowed::Show
            }
        }
    }

    /// What the file gave, once, for the actor to bring in.
    pub(crate) fn take_import(&mut self) -> Option<KeepassImport> {
        self.import.read.take()
    }

    /// After the actor brought the entries in: back where Import was pressed, and the note says `counts`.
    pub(crate) fn settle_import(&mut self, counts: NoteCounts) -> Flowed {
        let flowed = self.came_back();
        (self.note, self.counts) = (Some(FillNote::Imported), counts);
        flowed
    }

    /// Back where Import was pressed.
    pub(super) fn came_back(&mut self) -> Flowed {
        match self.import.came {
            Came::List => {
                self.flow = None;
                Flowed::Leave
            }
            Came::Home => {
                self.home();
                Flowed::Show
            }
            Came::Places(then) => self.places(then),
        }
    }
}

/// The note of a file that did not open; only its kind is logged.
fn shut_note(shut: Shut) -> FillNote {
    tracing::warn!(?shut, "a KeePass file did not open");
    match shut {
        Shut::Read(PlaceError::TooLarge) | Shut::Open(BoxError::TooLarge) => FillNote::FileTooLarge,
        Shut::Read(_) | Shut::Open(BoxError::TimedOut | BoxError::Failed) => FillNote::FileUnread,
        Shut::Open(BoxError::WrongPassword) => FillNote::FileLocked,
        Shut::Open(BoxError::Damaged) => FillNote::Damaged,
        Shut::Open(BoxError::Unsupported) => FillNote::Unsupported,
    }
}
