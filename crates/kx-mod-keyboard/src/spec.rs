//! The board as `board.toml` writes it: grid sizes, main rows, side keys, the D-pad and the shortcuts layer,
//! checked when read.

mod check;
mod error;
mod keys;
mod layer;

pub use error::BoardError;
pub use keys::{RowKey, char_id};
pub use layer::{LayerGrid, LayerSpec, NEXT_ID, SideCell, shortcut_key_id};

use kx_module_api::ports::{Scan, Toggle};
use serde::{Deserialize, Serialize};

/// The board, compiled in so it is never read from disk.
const BOARD: &str = include_str!("../board.toml");

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

/// A latch key's role: the modifier it latches and, when `lone`, the lone press of a second click.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LatchRole {
    /// The modifier.
    pub latch: Latch,
    /// A second click with nothing used since the first lets go and sends the key alone, as Win opens Start.
    #[serde(default)]
    pub lone: bool,
}

impl LatchRole {
    /// A role that latches `latch`, with no lone press.
    #[must_use]
    pub const fn of(latch: Latch) -> Self {
        Self { latch, lone: false }
    }
}

/// A mouse-assist mode a side key flips; the mouse module runs it (spec §5.1).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Assist {
    /// Right-click by holding still.
    RClick,
    /// Pick up at the next press, drop at the next click.
    Grab,
}

impl From<Assist> for Toggle {
    fn from(a: Assist) -> Self {
        match a {
            Assist::RClick => Self::RightClick,
            Assist::Grab => Self::Grab,
        }
    }
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
    /// Steps through or picks the layouts.
    Lang,
    /// Asks the mouse module to flip a mode; lit while it is on, dimmed while that module is not running.
    Assist(Assist),
    /// Comes in a later version: drawn dimmed and does nothing.
    Later,
    /// The Shortcuts key: a click shows or hides the shortcuts layer.
    Layer,
    /// The Power key: a click asks for the system's power dialog.
    Power,
    /// A shortcut on the layer; only the layer's generated cells have it.
    Shortcut,
    /// The layer's Next page key; only the layer's generated cell has it.
    Next,
    /// Latches a modifier; `board.toml` writes it as a table, `{ latch = "win", lone = true }`.
    #[serde(untagged)]
    Latch(LatchRole),
}

impl Role {
    /// True when a key with this role sends a scan code.
    #[must_use]
    pub fn sends(self) -> bool {
        matches!(self, Self::Char | Self::Send | Self::Caps | Self::Latch(_))
    }
}

/// What a board key does in an open panel, as `board.toml` names it with `nav`; other keys type as usual.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Nav {
    /// Up one row.
    Up,
    /// Down one row.
    Down,
    /// Back one page.
    Left,
    /// On one page.
    Right,
    /// Picks the highlighted row.
    Enter,
    /// Closes the panel.
    Esc,
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
    /// What it does in an open panel, if anything.
    #[serde(default)]
    pub nav: Option<Nav>,
    /// It repeats while held; only a `send` key may.
    #[serde(default)]
    pub repeat: bool,
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
    /// The shortcuts layer.
    pub layer: LayerSpec,
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

#[cfg(test)]
mod tests;
