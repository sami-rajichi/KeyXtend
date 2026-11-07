//! `[fill]` in `board.toml`: the key a Quick-fill fill's Tab part sends, named once by its id.

use super::{BoardError, BoardSpec, KeySpec, Role};
use kx_module_api::ports::Scan;
use serde::Deserialize;

/// `[fill]` in `board.toml`.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FillSpec {
    /// The id of the main-row `send` key a Tab part sends, alone.
    pub tab: String,
}

impl BoardSpec {
    /// The scan code a fill's Tab part sends; `None` only on a board `check` refuses.
    #[must_use]
    pub fn fill_tab(&self) -> Option<Scan> {
        let sends = |k: &&KeySpec| k.id == self.fill.tab && k.role == Role::Send;
        self.named().find(sends).and_then(|k| k.sc)
    }

    /// Refuses a Tab key that is not a main-row `send` key with a scan code.
    pub(super) fn check_fill(&self) -> Result<(), BoardError> {
        match self.fill_tab() {
            Some(_) => Ok(()),
            None => Err(BoardError::Fill(self.fill.tab.clone())),
        }
    }
}
