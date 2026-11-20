//! Each card's check for a model file under one driver and in one clip's language (ADR-0044, ADR-0052), with its day:
//! ids and numbers, never the clip's texts. A driver update, a new file or another language finds no result, so the
//! check runs again (edge 26); a card that fails mid-run loses that file's results alone (edge 25).

use kx_module_api::ports::{CardId, Date, Fingerprint, Verdict, fits_iso};

/// One card's check of one model file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Checked {
    /// The card.
    pub card: CardId,
    /// Its driver's version when it ran.
    pub driver: String,
    /// The model file's fingerprint, so a new file of the same name is checked again.
    pub model: Fingerprint,
    /// The ISO 639 code of the clip it ran on; `None` for a result kept before checks named it, which serves none.
    pub lang: Option<String>,
    /// How it ended.
    pub verdict: Verdict,
    /// The local day it ran.
    pub day: Date,
}

impl Checked {
    /// True for the same card and file, whatever the driver and the language.
    fn same(&self, card: &CardId, model: &Fingerprint) -> bool {
        self.card == *card && self.model == *model
    }

    /// True when `other` takes this result's place: the same card and file, in its language or where either has none.
    fn shares(&self, other: &Self) -> bool {
        let lang = match (&self.lang, &other.lang) {
            (Some(mine), Some(theirs)) => mine == theirs,
            _ => true,
        };
        lang && self.same(&other.card, &other.model)
    }
}

/// The checks kept, one per card, file and language.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Checks(Vec<Checked>);

impl Checks {
    /// The checks of `rows`, as the store loaded them; a later row for the same place wins.
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

    /// Keeps `checked` in place of the card's result for that file and language under any driver, and of an old one
    /// of no language. A language that is no ISO code is never kept, as the store reads back only those.
    pub fn put(&mut self, checked: Checked) {
        if checked.lang.as_deref().is_some_and(|l| !fits_iso(l)) {
            return;
        }
        self.0.retain(|c| !c.shares(&checked));
        self.0.push(checked);
    }

    /// The verdict of `card` for file `model` under `driver`, on the clip of `lang`, as the device rule needs it;
    /// `None` before a check ran there.
    #[must_use]
    pub fn verdict(
        &self,
        card: &CardId,
        driver: &str,
        model: &Fingerprint,
        lang: &str,
    ) -> Option<Verdict> {
        let here = |c: &&Checked| c.driver == driver && c.lang.as_deref() == Some(lang);
        let row = self.0.iter().filter(|c| c.same(card, model)).find(here);
        row.map(|c| c.verdict)
    }

    /// `card` failed mid-run on file `model` (`Out::Fell`): drops that file's results in every language, so its check
    /// runs again; false when it had none. Its other files keep theirs.
    pub fn fell(&mut self, card: &CardId, model: &Fingerprint) -> bool {
        let before = self.0.len();
        self.0.retain(|c| !c.same(card, model));
        self.0.len() < before
    }
}
