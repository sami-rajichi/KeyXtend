//! Compare models: the set text with the models it runs and where, each model's turn while they run, and the results
//! with their day and device. Only numbers are kept; a text shows only for a language with no set text, wiped on drop.

use super::DeviceLabel;
use super::cloud::ServiceName;
use crate::ports::{CloudWhy, Date};
use crate::{Redacted, Secret};

/// A cloud model Compare may send the recording to (ADR-0047); only a ticked one's service gets it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompareCloud {
    /// The model's shown name, or the id the user typed.
    pub name: Redacted<String>,
    /// Its service.
    pub service: ServiceName,
    /// It is ticked.
    pub tick: bool,
}

/// Where one model stands while Compare runs.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RunStep {
    /// It wrote the recording, as in Done · 2.1 s.
    Done {
        /// How long its text took, in ms.
        wait_ms: u32,
    },
    /// It failed, or Compare skipped it after its fault, so it has no numbers.
    Failed,
    /// It writes the recording now.
    Writing,
    /// Its turn has not come.
    Waiting,
}

/// One model's row while Compare runs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompareRun {
    /// The model's shown name, or the user's own file name.
    pub name: Redacted<String>,
    /// Where it stands.
    pub step: RunStep,
    /// Why a cloud model failed, the reason the bar would give; `None` for a local model or one that did not fail.
    pub why: Option<CloudWhy>,
}

/// One model's numbers in Compare's table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompareRow {
    /// The model's shown name, or the user's own file name.
    pub name: Redacted<String>,
    /// Words wrong, out of `words`; `None` for a language with no set text.
    pub wrong: Option<u16>,
    /// The set text's words.
    pub words: u16,
    /// How long its text took, in ms.
    pub wait_ms: u32,
    /// The most memory its job took, in MB.
    pub mb: u32,
    /// Its text, shown only for a language with no set text, never kept, and wiped when the last view holding it drops.
    pub text: Option<Secret>,
    /// It is ticked.
    pub tick: bool,
    /// The service that wrote it, for a cloud model; `None` for one on this PC.
    pub cloud: Option<ServiceName>,
}

/// Where Compare's microphone stands before its models write.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CompareMic {
    /// Closed, so Start shows.
    Closed,
    /// It opens: Start is gone and Cancel ends it, but nothing asks to read yet, as no bar shows.
    Opening,
    /// It is open and the user reads aloud now, so Start is gone and the bar's Stop ends it.
    Reading,
}

/// Where Compare stands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompareSheet {
    /// The set text, as in 6 ready models · Automatic (processor), with Start until the user reads it.
    Ready {
        /// The ready models it runs.
        models: u16,
        /// Where they run.
        device: DeviceLabel,
        /// Where its microphone stands, which says whether Start shows.
        mic: CompareMic,
        /// The shown page of the cloud models of the services ready, ticked or not.
        cloud: Vec<CompareCloud>,
        /// The services the recording goes to after it is read, which the send warning names before Start; empty for
        /// none, so no warning shows.
        send: Vec<ServiceName>,
    },
    /// Model `at` of `of` writes the recording, as in Model 3 of 6 · its name, with Cancel.
    Running {
        /// The model writing, from 1.
        at: u16,
        /// The models to run.
        of: u16,
        /// The name of the model writing.
        name: Redacted<String>,
        /// The shown page of the models' rows.
        rows: Vec<CompareRun>,
    },
    /// The shown page of the results, as in 12 Nov · Automatic (processor).
    Results {
        /// The day it ran.
        day: Date,
        /// Where the models ran.
        device: DeviceLabel,
        /// The shown page's rows.
        rows: Vec<CompareRow>,
    },
}
