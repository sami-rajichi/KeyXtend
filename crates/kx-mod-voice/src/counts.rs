//! The numbers the store keeps, never content (design "Counts"): per day, model and device, the dictations, the time
//! heard and the time waited, for P13's dashboard; each card's check (`checks`); and Compare's results (`scores`).

mod checks;
mod scores;

pub use checks::{Checked, Checks};
pub use scores::{Board, Score, Scores};

use std::collections::BTreeMap;

use kx_module_api::ports::{CardId, Date};

use crate::flow::Count;
use crate::models::table;

/// The model a count is for: a models table row by its id, or any other file, whose name is never kept.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Model {
    /// A row of the models table.
    Row(String),
    /// A model file the table does not name.
    Other,
}

impl Model {
    /// The model `name` names: a row's id, else any other file.
    #[must_use]
    pub fn of(name: &str) -> Self {
        let row = table().ok().and_then(|t| t.get(name));
        row.map_or(Self::Other, |r| Self::Row(r.id.clone()))
    }
}

/// Where a dictation was written.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Device {
    /// The processor.
    Cpu,
    /// This graphics card, to the end.
    Card(CardId),
}

/// What a count is kept under: a local day, a model and a device.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bucket {
    /// The local day of the dictations.
    pub date: Date,
    /// The model that wrote them.
    pub model: Model,
    /// Where they were written.
    pub device: Device,
}

/// The numbers of one bucket.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Tally {
    /// How many dictations were written.
    pub dictations: u32,
    /// How long the sound heard was, in ms.
    pub sound_ms: u64,
    /// How long the texts took after Stop, in ms.
    pub wait_ms: u64,
}

/// The counts, oldest day first.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Counts(BTreeMap<Bucket, Tally>);

impl Counts {
    /// The counts of `rows`, as the store loaded them.
    #[must_use]
    pub fn new(rows: BTreeMap<Bucket, Tally>) -> Self {
        Self(rows)
    }

    /// Every bucket with its numbers, oldest day first.
    #[must_use]
    pub fn rows(&self) -> &BTreeMap<Bucket, Tally> {
        &self.0
    }

    /// Counts `count`, a dictation written on local day `date`; each number stops at its top rather than wrap.
    pub fn add(&mut self, date: Date, count: &Count) {
        let device = count.card.clone().map_or(Device::Cpu, Device::Card);
        let model = Model::of(count.model.expose());
        let tally = self.0.entry(Bucket {
            date,
            model,
            device,
        });
        let tally = tally.or_default();
        tally.dictations = tally.dictations.saturating_add(1);
        tally.sound_ms = tally.sound_ms.saturating_add(count.done.sound_ms.into());
        tally.wait_ms = tally.wait_ms.saturating_add(count.done.wait_ms.into());
    }

    /// Drops every bucket of the oldest day, so the counts fit in one blob; false when none is left.
    pub fn drop_oldest(&mut self) -> bool {
        let Some(oldest) = self.0.keys().next().map(|b| b.date) else {
            return false;
        };
        self.0.retain(|b, _| b.date != oldest);
        true
    }
}

#[cfg(test)]
mod tests;
