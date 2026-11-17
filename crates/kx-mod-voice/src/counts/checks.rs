//! Each card's check for a model file under one driver (ADR-0044), with its day: ids and numbers, never the clip's
//! texts. A driver update or a new file finds no result, so the check runs again (edge 26); a card that fails mid-run
//! loses its result for that file alone (edge 25).

use kx_module_api::ports::{CardId, Date, Fingerprint, Verdict};

/// One card's check of one model file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Checked {
    /// The card.
    pub card: CardId,
    /// Its driver's version when it ran.
    pub driver: String,
    /// The model file's fingerprint, so a new file of the same name is checked again.
    pub model: Fingerprint,
    /// How it ended.
    pub verdict: Verdict,
    /// The local day it ran.
    pub day: Date,
}

impl Checked {
    /// True for the same card and file, whatever the driver.
    fn same(&self, card: &CardId, model: &Fingerprint) -> bool {
        self.card == *card && self.model == *model
    }
}

/// The checks kept, one per card and file.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Checks(Vec<Checked>);

impl Checks {
    /// The checks of `rows`, as the store loaded them; a later row for the same card and file wins.
    #[must_use]
    pub fn new(rows: Vec<Checked>) -> Self {
        let mut checks = Self::default();
        for row in rows {
            checks.put(row);
        }
        checks
    }

    /// Every check kept.
    #[must_use]
    pub fn rows(&self) -> &[Checked] {
        &self.0
    }

    /// Keeps `checked` in place of the card's result for that file under any driver.
    pub fn put(&mut self, checked: Checked) {
        self.0.retain(|c| !c.same(&checked.card, &checked.model));
        self.0.push(checked);
    }

    /// The verdict of `card` for file `model` under `driver`, as the device rule needs it; `None` before a check ran
    /// under this driver.
    #[must_use]
    pub fn verdict(&self, card: &CardId, driver: &str, model: &Fingerprint) -> Option<Verdict> {
        let row = self
            .0
            .iter()
            .find(|c| c.same(card, model) && c.driver == driver);
        row.map(|c| c.verdict)
    }

    /// `card` failed mid-run on file `model` (`Out::Fell`): drops that result, so its check runs again; false when it
    /// had none. Its other files keep theirs, as each result is per card, driver and model.
    pub fn fell(&mut self, card: &CardId, model: &Fingerprint) -> bool {
        let before = self.0.len();
        self.0.retain(|c| !c.same(card, model));
        self.0.len() < before
    }
}
