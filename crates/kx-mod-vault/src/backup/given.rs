//! What a box gave: its entries wait for the actor, and nothing else changes until they fit. Then its padlock and
//! place are kept, but a copy with the box's own file beside it keeps no place, and Bring back ends a hold and writes
//! every place, asking first when another box would be overwritten; another computer's box only writes with no hold.

use kx_module_api::ports::{
    BoxError, FillNote, NOTEBOOK_PART, NoteCounts, Opened, Padlock, PlaceError,
};

use super::Site;
use super::bring::{Picked, Source};
use super::flow::{Asking, Flow};
use super::jobs::{Shut, Unboxed};
use super::state::{At, Backup, Flowed, Purpose};
use crate::entry::{Entry, Stamp};
use crate::store::{Bad, Store};

/// A box opened, until the actor knows whether its entries fit.
#[derive(Debug)]
pub(super) struct Given {
    entries: Vec<Entry>,
    padlock: Padlock,
    mark: u64,
    len: u64,
    beside: bool,
    source: Source,
}

impl Backup {
    /// Takes an opened box of `round`: a refusal says why and the password page stays; else its entries wait for the
    /// actor.
    pub(super) fn opened(&mut self, round: u32, opened: Result<Unboxed, Shut>) -> Flowed {
        let Some(Purpose::Open(source)) = self.answered(round) else {
            return Flowed::Stay;
        };
        let read = opened
            .map_err(shut_note)
            .and_then(|u| Ok((entries_of(&u.opened)?, u)));
        let (entries, unboxed) = match read {
            Ok(read) => read,
            Err(note) => {
                self.note = Some(note);
                return Flowed::Show;
            }
        };
        self.bring.given = Some(Given {
            entries,
            padlock: unboxed.opened.padlock,
            mark: unboxed.mark,
            len: unboxed.len,
            beside: unboxed.beside,
            source,
        });
        Flowed::Brought
    }

    /// The entries a box gave, once, for the actor to bring in.
    pub(crate) fn take_brought(&mut self) -> Option<Vec<Entry>> {
        let given = self.bring.given.as_mut()?;
        Some(std::mem::take(&mut given.entries))
    }

    /// After the actor took the entries: with `added` none, as they would not fit, nothing changes and the note says
    /// so; else the note counts them, the box's padlock and place are kept, and the Backup page shows the places, with
    /// Replace's question up while a hold would otherwise overwrite another box.
    pub(crate) fn settle(&mut self, added: Option<u32>, at: &At<'_>) {
        let Some(given) = self.bring.given.take() else {
            return;
        };
        let Some(added) = added else {
            self.note = Some(FillNote::Full);
            return;
        };
        if self.kept.padlock.is_none() {
            self.kept.padlock = Some(given.padlock);
            self.begin_checks(Stamp::of(&at.wall));
        }
        let foreign = matches!(given.source.picked, Picked::Foreign(_));
        let (from, beside) = (given.source.place.path.clone(), given.beside);
        if !beside {
            self.keep_place(given.source, given.mark, given.len);
        }
        self.bring.offer = false;
        let sites = &self.kept.sites;
        let elsewhere = sites.iter().any(|s| beside || s.place.path != from);
        let asks = !foreign && self.kept.held && elsewhere;
        if foreign {
            self.due = None;
            self.save();
            self.write(at.book, false, at.now);
        } else if asks {
            self.save();
        } else {
            self.release(at.book, at.now);
        }
        let asking = asks.then_some(Asking::Replace);
        self.flow = Some(Flow::Home { at: 0, asking });
        self.counts = NoteCounts { added, skipped: 0 };
        self.note = Some(FillNote::Imported);
    }

    /// Keeps the box's place: first for Bring back, last after the question about another computer's box; a place
    /// kept already stays where it is.
    fn keep_place(&mut self, source: Source, mark: u64, len: u64) {
        let Source { place, picked, .. } = source;
        let kept = self.kept.sites.iter().any(|s| s.place.path == place.path);
        // The blob keeps a path as text, so one that is not valid Unicode brings its entries but is no place.
        if kept || place.path.expose().to_str().is_none() {
            return;
        }
        let at = match picked {
            Picked::Foreign(_) => self.kept.sites.len(),
            Picked::Found | Picked::Folder(_) => 0,
        };
        let (written, mark, waiting) = (None, Some(mark), true);
        let site = Site {
            place,
            written,
            mark,
            len,
            waiting,
        };
        self.kept.sites.insert(at, site);
    }
}

/// The entries of an opened box's notebook, read as a load reads the notebook.
fn entries_of(opened: &Opened) -> Result<Vec<Entry>, FillNote> {
    let part = opened.parts.iter().find(|p| p.name == NOTEBOOK_PART);
    let part = part.ok_or(FillNote::Damaged)?;
    Store::read_part(part.bytes.expose()).map_err(|bad| match bad {
        Bad::Newer => FillNote::Unsupported,
        Bad::Damaged => FillNote::Damaged,
    })
}

/// The note of a box that did not open; only its kind is logged.
fn shut_note(shut: Shut) -> FillNote {
    tracing::warn!(?shut, "a backup did not open");
    match shut {
        Shut::Read(PlaceError::TooLarge) | Shut::Open(BoxError::TooLarge) => FillNote::TooLarge,
        Shut::Read(_) => FillNote::Unread,
        Shut::Open(BoxError::WrongPassword) => FillNote::WrongPassword,
        Shut::Open(BoxError::Damaged) => FillNote::Damaged,
        Shut::Open(BoxError::Unsupported) => FillNote::Unsupported,
        Shut::Open(BoxError::TimedOut | BoxError::Failed) => FillNote::BoxFailed,
    }
}
