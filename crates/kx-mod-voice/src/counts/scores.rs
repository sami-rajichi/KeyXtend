//! Compare's last results per language (design "Compare models"): the day, where the models ran, and each model's
//! numbers by its file's fingerprint. The recording and the texts are never kept.

use kx_module_api::ports::{CardId, Date, Fingerprint, fits_iso};

/// One model's numbers in a run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Score {
    /// The model file's fingerprint; the panel names it from the table or the folder.
    pub model: Fingerprint,
    /// Words wrong, out of `words`; `None` for a language with no set text.
    pub wrong: Option<u16>,
    /// The set text's words.
    pub words: u16,
    /// How long its text took, in ms.
    pub wait_ms: u32,
    /// The most memory its job took, in MB.
    pub mb: u32,
}

/// One language's last run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Board {
    /// The language, an ISO 639 code.
    pub lang: String,
    /// The local day it ran.
    pub day: Date,
    /// The device was Automatic.
    pub auto: bool,
    /// The card the models ran on; `None` for the processor.
    pub card: Option<CardId>,
    /// Each model's numbers, in the order they ran.
    pub scores: Vec<Score>,
}

/// The last run of each language.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Scores(Vec<Board>);

impl Scores {
    /// The runs of `boards`, as the store loaded them; a later one of the same language wins.
    #[must_use]
    pub fn new(boards: Vec<Board>) -> Self {
        let mut scores = Self::default();
        for board in boards {
            scores.put(board);
        }
        scores
    }

    /// Every language's last run.
    #[must_use]
    pub fn boards(&self) -> &[Board] {
        &self.0
    }

    /// Keeps `board` in place of its language's last run; false, keeping nothing, when its language is no ISO code
    /// (`fits_iso`), as the store reads back only those.
    pub fn put(&mut self, board: Board) -> bool {
        if !fits_iso(&board.lang) {
            return false;
        }
        self.0.retain(|b| b.lang != board.lang);
        self.0.push(board);
        true
    }

    /// The last run of `lang`, which the home list's Last run names.
    #[must_use]
    pub fn get(&self, lang: &str) -> Option<&Board> {
        self.0.iter().find(|b| b.lang == lang)
    }
}
