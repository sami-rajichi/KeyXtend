//! What the actor does after a panel input, in order: reads, writes, downloads, a card's check, History's acts and
//! Compare's run. Each read comes back through the `Panel` method its line names.

use std::path::PathBuf;

use kx_module_api::ports::{CardId, CheckAsk, FactsAsk, FetchAsk, FetchButton, Fingerprint};

use crate::compare::Act;
use crate::counts::Checked;
use crate::history::Asked;

/// A card's check to run in the background, with what its verdict is kept by.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    /// The card.
    pub card: CardId,
    /// Its driver's version now.
    pub driver: String,
    /// The model file's fingerprint.
    pub print: Fingerprint,
    /// The clip's language, which scores its texts.
    pub clip: String,
    /// The ask for `Speech::check`.
    pub ask: CheckAsk,
}

/// One effect for the actor.
#[derive(Clone, Debug, PartialEq)]
pub enum Do {
    /// The panel closed: draw no panel.
    Close,
    /// Read the devices again, for their memory now.
    Devices,
    /// Read the models folder, then call `Panel::scanned`.
    Scan,
    /// Read this file's fingerprint off the module's thread, then call `Panel::printed` with the file and its size.
    Print {
        /// The file.
        file: PathBuf,
        /// Its size as scanned.
        bytes: u64,
    },
    /// Read an other file's facts, then call `Panel::facts_read` with its fingerprint.
    Facts(Fingerprint, FactsAsk),
    /// Read this folder's subfolders for the folder list, then call `Panel::browsed`.
    Browse(PathBuf),
    /// Make this models folder, then call `Panel::made`.
    Make(PathBuf),
    /// Write this `[voice]` key.
    Set(&'static str, toml::Value),
    /// Start row `row`'s download as `ask` says; its notes go to `Panel::fetch_note`.
    Fetch {
        /// The row's id.
        row: String,
        /// The ask.
        ask: FetchAsk,
    },
    /// Pause, resume or cancel row `row`'s download.
    FetchCall {
        /// The row's id.
        row: String,
        /// The button.
        button: FetchButton,
    },
    /// Delete this model file.
    Remove(PathBuf),
    /// Show this folder in the file manager.
    Reveal(PathBuf),
    /// Run this card's check; its warm-up goes to `Panel::warming` and its end to `Panel::checked`.
    Check(Box<Check>),
    /// Keep this check with `Checks::put`, then save the checks.
    Checked(Checked),
    /// Type these texts, in this order.
    Insert(Vec<u64>),
    /// Copy these texts, in this order.
    Copy(Vec<u64>),
    /// Pin these texts.
    Pin(Vec<u64>),
    /// Unpin these texts, as if dictated now.
    Unpin(Vec<u64>),
    /// Delete these texts.
    Delete(Vec<u64>),
    /// Yes to Delete all.
    DeleteAll(Asked),
    /// An effect on Compare's run.
    Compare(Act),
}
