//! Compare models: the set text with the models it runs and where, each model's turn while they run, and the results
//! with their day and device. Only numbers are kept; a text shows only for a language with no set text.

use super::DeviceLabel;
use crate::Redacted;
use crate::ports::Date;

/// Where one model stands while Compare runs.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RunStep {
    /// It wrote the recording, as in Done · 2.1 s.
    Done {
        /// How long its text took, in ms.
        wait_ms: u32,
    },
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
    /// Its text, shown only for a language with no set text and never kept.
    pub text: Option<Redacted<String>>,
    /// It is ticked.
    pub tick: bool,
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
        /// The user reads it aloud now, so Start is gone and the bar's Stop ends it.
        recording: bool,
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
