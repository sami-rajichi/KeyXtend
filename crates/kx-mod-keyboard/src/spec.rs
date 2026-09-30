//! The board as `board.toml` writes it: grid sizes, main rows, side keys and the D-pad, checked when read.

mod check;

use kx_module_api::ports::Scan;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The board, compiled in so it is never read from disk.
const BOARD: &str = include_str!("../board.toml");
/// What starts a character key's id, `c-<row>-<index>`.
const CHAR_PREFIX: &str = "c";

/// What a key is, which sets its colours.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyKind {
    /// A character key.
    Char,
    /// A modifier or navigation key.
    Mod,
    /// Esc or Backspace.
    Danger,
    /// Enter.
    Enter,
    /// The language key.
    Lang,
    /// The space bar.
    Space,
    /// A side-block key.
    Act,
}

/// A modifier that latches; both Shift keys share one latch.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Latch {
    /// Shift.
    Shift,
    /// Control.
    Ctrl,
    /// Left Alt.
    Alt,
    /// Right Alt, which is `AltGr` on layouts that have it.
    AltGr,
    /// The Windows key.
    Win,
}

/// What a click on a key does.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// Types its character in the layout shown; only the keys of a `chars` run have it.
    Char,
    /// Sends its scan code, with the latched modifiers.
    Send,
    /// Caps Lock: sends its scan code and lights while Caps Lock is on.
    Caps,
    /// Latches a modifier.
    Latch(Latch),
    /// Steps through or picks the layouts.
    Lang,
    /// Comes in a later version: drawn dimmed and does nothing.
    Later,
}

impl Role {
    /// True when a key with this role sends a scan code.
    #[must_use]
    pub fn sends(self) -> bool {
        matches!(self, Self::Char | Self::Send | Self::Caps | Self::Latch(_))
    }
}

/// A named key in a main row.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeySpec {
    /// Short id, such as `esc`.
    pub id: String,
    /// Width in key units.
    pub w: f64,
    /// Kind.
    pub kind: KeyKind,
    /// The scan code it sends; none when its role sends none.
    #[serde(default)]
    pub sc: Option<Scan>,
    /// What a click does.
    pub role: Role,
}

/// A run of one-unit character keys, by scan code.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharSpec {
    /// Scan codes, left to right.
    pub chars: Vec<Scan>,
}

/// One entry of a main row.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum RowItem {
    /// A named key.
    Key(KeySpec),
    /// Character keys.
    Chars(CharSpec),
}

/// A side key and its cell, 1-based.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SideSpec {
    /// Short id, such as `mic`.
    pub id: String,
    /// Column, from 1.
    pub col: u16,
    /// Row, from 1.
    pub row: u16,
    /// What a click does.
    pub role: Role,
}

/// The D-pad on the side grid.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DpadSpec {
    /// First column, first row, last column and last row, 1-based and inclusive.
    pub cell: [u16; 4],
    /// Largest width in key units.
    pub units: f64,
    /// It comes in a later version, so it is drawn dimmed.
    pub later: bool,
}

/// `board.toml`, in logical pixels at size 1.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoardSpec {
    /// One key unit wide.
    pub unit_px: f64,
    /// One row high.
    pub row_px: f64,
    /// Main block width in units.
    pub main_units: f64,
    /// Grid columns per unit.
    pub columns_per_unit: u16,
    /// Side block columns.
    pub side_columns: u16,
    /// Side column width in units.
    pub side_unit: f64,
    /// Gap between the blocks.
    pub body_gap_px: f64,
    /// Plate padding: top, right, bottom, left.
    pub plate_pad_px: [f64; 4],
    /// Top bar height.
    pub top_bar_px: f64,
    /// Gap under the top bar.
    pub top_gap_px: f64,
    /// The D-pad.
    pub dpad: DpadSpec,
    /// Main rows, top to bottom.
    pub rows: Vec<Vec<RowItem>>,
    /// Side keys.
    pub side: Vec<SideSpec>,
}

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

impl BoardSpec {
    /// The board compiled into the app, checked.
    ///
    /// # Errors
    /// A `BoardError` when the compiled-in board is broken.
    pub fn shipped() -> Result<Self, BoardError> {
        Self::read(BOARD)
    }

    /// Reads and checks a board from `text`.
    ///
    /// # Errors
    /// `Parse` when `text` is not a board, else the error from `check`.
    pub fn read(text: &str) -> Result<Self, BoardError> {
        let spec: Self = toml::from_str(text).map_err(|e| BoardError::Parse(e.to_string()))?;
        spec.check()?;
        Ok(spec)
    }

    /// Grid columns a key `w` units wide spans, when that is a whole number from 1 up.
    #[must_use]
    pub fn columns(&self, w: f64) -> Option<u16> {
        let span = w * f64::from(self.columns_per_unit);
        let whole = span.round();
        let exact = (span - whole).abs() <= span.abs() * f64::EPSILON;
        let fits = (1.0..=f64::from(u16::MAX)).contains(&whole);
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "the value is whole and inside the u16 range"
        )]
        (exact && fits).then_some(whole as u16)
    }
}

/// One key of a main row, as `rows` lists it.
#[derive(Clone, Debug, PartialEq)]
pub enum RowKey<'a> {
    /// A named key.
    Named(&'a KeySpec),
    /// A character key.
    Char {
        /// Its id, `c-<row>-<index>`.
        id: String,
        /// Its scan code.
        sc: Scan,
    },
}

impl BoardSpec {
    /// The keys of main row `row`, left to right; character keys are numbered across the whole row.
    #[must_use]
    pub fn row_keys(&self, row: usize) -> Vec<RowKey<'_>> {
        let Some(items) = self.rows.get(row) else {
            return Vec::new();
        };
        let mut keys = Vec::new();
        let mut index = 0;
        for item in items {
            match item {
                RowItem::Key(k) => keys.push(RowKey::Named(k)),
                RowItem::Chars(c) => {
                    for &sc in &c.chars {
                        keys.push(RowKey::Char {
                            id: char_id(row, index),
                            sc,
                        });
                        index += 1;
                    }
                }
            }
        }
        keys
    }

    /// The scan codes of every character key, row by row, left to right.
    #[must_use]
    pub fn char_scans(&self) -> Vec<Scan> {
        let runs = self.rows.iter().flatten().filter_map(|i| match i {
            RowItem::Chars(c) => Some(c.chars.iter().copied()),
            RowItem::Key(_) => None,
        });
        runs.flatten().collect()
    }
}

/// The id of character key `index` in main row `row`, both from 0.
#[must_use]
pub fn char_id(row: usize, index: usize) -> String {
    format!("{CHAR_PREFIX}-{row}-{index}")
}

#[cfg(test)]
mod tests;
