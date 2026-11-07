//! The backup box (design "The backup box", ADR-0038 decisions 5 to 7): what the backup keeps in its own sealed blob
//! beside the notebook (the places, the padlock, each place's last write, the marks of the boxes this computer wrote,
//! the first offer, the hold and the memory checks), the box jobs on their own thread, setup, the writes, the Backup
//! page, the memory checks, Bring back, a box from another computer and Import's pages. Paths stay redacted and are
//! never logged.

mod back;
mod bring;
mod browse;
mod checks;
mod codec;
mod flow;
mod foreign;
mod given;
mod jobs;
mod keepass;
mod mark;
mod page;
mod setup;
mod state;
mod view;
mod write;

#[cfg(test)]
pub(crate) use jobs::{Answer, Ask, Job, Look, Post, Shut};
pub(crate) use jobs::{Done, Jobs, Runner};
pub(crate) use state::{At, Backup, Config, Flowed, Links};

use std::sync::Arc;

use kx_module_api::ports::{Padlock, Place, SealedStore};

use crate::entry::Stamp;
use crate::store::{Slot, StoreError};

/// The backup's blob.
const BLOB: &str = "backup";
/// Where a backup blob that opens but does not read is kept.
const ASIDE: &str = "backup-aside";

/// A place the box lives in, and its box's last write by this computer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Site {
    /// Where it is, its kind and its short name.
    pub place: Place,
    /// When this computer last wrote its box.
    pub written: Option<Stamp>,
    /// The mark of the box this computer last wrote there, so a pick of it later knows the box as its own.
    pub mark: Option<u64>,
    /// That box's length, the room the next write asks for there, even after a start; 0 until written.
    pub len: u64,
    /// Its box is behind: the place was gone or full, a write failed, or a new padlock waits for it.
    pub waiting: bool,
}

/// The memory checks' state: the next check, the step in `check_days`, and the wrong answers.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Checks {
    /// When the next check asks.
    pub due: Option<Stamp>,
    /// How many right answers in a row stepped through `check_days`.
    pub step: u8,
    /// Wrong answers since the last right one.
    pub missed: u8,
}

/// What the backup keeps in its blob.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Kept {
    /// The places, in the order added.
    pub sites: Vec<Site>,
    /// The open padlock and its locked identity; `None` until setup makes one.
    pub padlock: Option<Padlock>,
    /// The marks of the boxes left in places since removed, so adding one again knows its box.
    pub left: Vec<u64>,
    /// The first offer was made, so it never asks again.
    pub offered: bool,
    /// A write was due when the module last stopped, so the next start writes.
    pub behind: bool,
    /// The notebook did not read, so no box is written until Back up now's Yes or Bring back, even after a start.
    pub held: bool,
    /// The memory checks.
    pub checks: Checks,
}

impl Kept {
    /// True when `mark` names a box this computer wrote, in a place it keeps or one since removed.
    #[must_use]
    pub fn wrote(&self, mark: u64) -> bool {
        let kept = self.sites.iter().any(|s| s.mark == Some(mark));
        kept || self.left.contains(&mark)
    }
}

/// What a load found.
#[derive(Debug, PartialEq, Eq)]
pub struct KeptLoad {
    /// What the blob kept, clean when nothing was saved or what was saved was damaged.
    pub kept: Kept,
    /// What was saved was damaged, so the module shows `keys::VAULT_BACKUP_DAMAGED`.
    pub damaged: bool,
}

/// The backup's blob in the notebook's sealed store.
pub struct BackupStore {
    slot: Slot,
}

impl BackupStore {
    /// A store over `sealed`.
    #[must_use]
    pub fn new(sealed: Arc<dyn SealedStore>) -> Self {
        Self {
            slot: Slot::new(sealed, BLOB, ASIDE),
        }
    }

    /// Reads the blob.
    ///
    /// # Errors
    /// `Newer` for a newer version's blob, or the sealed store's error but `Damaged`, which starts clean.
    pub fn load(&mut self) -> Result<KeptLoad, StoreError> {
        let read = self.slot.load(codec::decode)?;
        Ok(KeptLoad {
            kept: read.value.unwrap_or_default(),
            damaged: read.damaged,
        })
    }

    /// Saves `kept` in place of what was saved.
    ///
    /// # Errors
    /// `Held` while an unread blob could not be set aside, or the sealed store's error.
    pub fn save(&self, kept: &Kept) -> Result<(), StoreError> {
        self.slot.save(&codec::encode(kept))
    }
}

#[cfg(test)]
mod props;
#[cfg(test)]
mod tests;
