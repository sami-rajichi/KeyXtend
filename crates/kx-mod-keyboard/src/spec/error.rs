//! Why a board was refused, and the fixed text a refused start keeps.

use kx_module_api::ModuleError;
use thiserror::Error;

/// Why a board was refused; the texts name only board ids, never user data.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum BoardError {
    /// The text is not a board.
    #[error("the board does not parse: {0}")]
    Parse(String),
    /// A size is not a number above 0, or the main block is not whole columns.
    #[error("the board's sizes must be numbers above 0, and main_units whole columns")]
    Size,
    /// A named key is not a whole number of columns.
    #[error("key {0} is not a whole number of columns")]
    Width(String),
    /// A main row does not span the main block.
    #[error("row {row} spans {spans} of {want} columns")]
    Row {
        /// The row, from 1.
        row: usize,
        /// Columns it spans.
        spans: u32,
        /// Columns the main block has.
        want: u32,
    },
    /// The D-pad is off the side grid, or its last cell comes first.
    #[error("the D-pad must sit on the side grid, its first cell before its last")]
    Dpad,
    /// A side key is off the grid, on the D-pad or in a taken cell.
    #[error("side key {0} is off the grid, on the D-pad or in a taken cell")]
    Side(String),
    /// Two keys share an id.
    #[error("key id {0} is used twice")]
    Twice(String),
    /// A key's role does not fit where it is or its scan code.
    #[error("key {0} has a role that does not fit it")]
    Role(String),
}

impl BoardError {
    /// The broken rule as a fixed text, without the key or row it names.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Parse(_) => "the board does not parse",
            Self::Size => "a board size is not a number above 0",
            Self::Width(_) => "a board key is not whole columns",
            Self::Row { .. } => "a board row does not span the grid",
            Self::Dpad => "the D-pad is off the side grid",
            Self::Side(_) => "a side key is off the grid or in a taken cell",
            Self::Twice(_) => "a board key id is used twice",
            Self::Role(_) => "a board key's role does not fit it",
        }
    }
}

/// A broken board stops the module's start with the broken rule's fixed text.
impl From<BoardError> for ModuleError {
    fn from(e: BoardError) -> Self {
        ModuleError::Start(e.kind())
    }
}
