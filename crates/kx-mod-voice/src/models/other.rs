//! The fit against this PC (plan edge case 23) and a model file no row names (edge case 37, ADR-0043 decision 7): it
//! is capped at its size plus `job_mb`, and serves the languages its own facts name.

use std::path::Path;

use kx_module_api::Redacted;
use kx_module_api::ports::{FactsAsk, JobLimits, MB, Memory, ModelFacts, ModelState, SpeechModel};

use super::{Row, Table};

/// The marks between a language and its region in a model's own code, as `fr-FR` or `pt_BR`.
const REGION_MARKS: [char; 2] = ['-', '_'];

/// A model file in the folder that no row names: Other file, not checked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Other {
    /// Its file name in the models folder.
    pub name: String,
    /// Its size in bytes.
    pub bytes: u64,
}

impl Other {
    /// The memory it counts as, its size in MB rounded up; `None` for an empty file or one past the port's range.
    #[must_use]
    pub fn need_mb(&self) -> Option<u32> {
        let mb = u32::try_from(self.bytes.div_ceil(MB)).ok()?;
        SpeechModel::CPU_MB.contains(&mb).then_some(mb)
    }

    /// Its job's memory cap: its size plus `job_mb`.
    #[must_use]
    pub fn cap_mb(&self, job_mb: u32) -> Option<u32> {
        self.need_mb()?.checked_add(job_mb)
    }

    /// The ask that reads its facts from `folder` in a job of `job`'s limits, capped at its size plus `job_mb`.
    #[must_use]
    pub fn facts(&self, folder: &Path, job: JobLimits) -> Option<FactsAsk> {
        Some(FactsAsk {
            job,
            file: Redacted::new(folder.join(&self.name)),
            size_mb: self.need_mb()?,
        })
    }

    /// The model a job loads from `folder` for `lang`, by the code its `facts` give; `None` when they name no such
    /// language.
    #[must_use]
    pub fn model(&self, folder: &Path, facts: &ModelFacts, lang: &str) -> Option<SpeechModel> {
        Some(SpeechModel {
            file: Redacted::new(folder.join(&self.name)),
            lang: code_in(facts, lang)?.to_owned(),
            runner: facts.runner.clone(),
            cpu_mb: self.need_mb()?,
        })
    }
}

/// The first of `facts`' codes whose language, before any region, is `lang` in any case.
#[must_use]
pub fn code_in<'f>(facts: &'f ModelFacts, lang: &str) -> Option<&'f str> {
    let speaks = |code: &&str| {
        (code.split(REGION_MARKS).next()).is_some_and(|l| l.eq_ignore_ascii_case(lang))
    };
    facts.langs.iter().map(String::as_str).find(speaks)
}

impl Row {
    /// Not for this PC, with the need, the total and the free memory, when the row's memory and `job_mb` pass the
    /// processor's total memory.
    #[must_use]
    pub fn not_here(&self, memory: &Memory, job_mb: u32) -> Option<ModelState> {
        let need_mb = self.cpu_mb.saturating_add(job_mb);
        let (have_mb, free_mb) = (memory.total_mb, memory.free_mb);
        let state = ModelState::NotHere {
            need_mb,
            have_mb,
            free_mb,
        };
        (need_mb > have_mb).then_some(state)
    }
}

impl Table {
    /// The rows for `lang` that fit this PC, best first; empty when none does, which the panel says.
    #[must_use]
    pub fn usable(&self, lang: &str, memory: &Memory, job_mb: u32) -> Vec<&Row> {
        let fits = |r: &&Row| r.not_here(memory, job_mb).is_none();
        self.order(lang).rows.into_iter().filter(fits).collect()
    }
}
