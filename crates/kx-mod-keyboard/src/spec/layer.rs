//! The shortcuts layer on the side grid: the Next key's cell, the cells a page fills in reading order, and the ids of
//! the keys the layer generates.

use super::{BoardError, BoardSpec, Role, SideSpec};
use kx_module_api::ports::ShortcutId;
use serde::Deserialize;

/// What starts a shortcut key's id: `sc-`, then its shortcut, such as `sc-copy-all`.
const SHORTCUT_PREFIX: &str = "sc-";

/// The id of the Next key, which the layer shows while it has more than one page.
pub const NEXT_ID: &str = "layer-next";

/// `[layer]` in `board.toml`.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayerSpec {
    /// The Next key's side cell, column then row from 1, while the layer has more than one page.
    pub next: [u16; 2],
}

/// A cell of the side grid, 1-based.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SideCell {
    /// Column, from 1.
    pub col: u16,
    /// Row, from 1.
    pub row: u16,
}

/// Where the layer draws: every side cell but the Shortcuts key's in reading order, and the Next key's cell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayerGrid {
    /// Every side cell, row by row and left to right, less the Shortcuts key's.
    cells: Vec<SideCell>,
    /// The Next key's cell.
    next: SideCell,
}

impl LayerGrid {
    /// The cells a page fills when the layer has `pages` pages, in reading order; Next's is left out when there is
    /// more than one page.
    #[must_use]
    pub fn cells(&self, pages: usize) -> Vec<SideCell> {
        let with_next = pages > 1;
        let free = |c: &&SideCell| !(with_next && **c == self.next);
        self.cells.iter().filter(free).copied().collect()
    }

    /// How many shortcuts a page holds when the layer has `pages` pages.
    #[must_use]
    pub fn capacity(&self, pages: usize) -> usize {
        self.cells(pages).len()
    }

    /// The Next key's cell.
    #[must_use]
    pub fn next(&self) -> SideCell {
        self.next
    }
}

impl BoardSpec {
    /// Where this board's shortcuts layer draws.
    #[must_use]
    pub fn layer_grid(&self) -> LayerGrid {
        let key = self.layer_key().map(|k| SideCell {
            col: k.col,
            row: k.row,
        });
        let rows = u16::try_from(self.rows.len()).unwrap_or(u16::MAX);
        let cols = self.side_columns;
        let row = |row| (1..=cols).map(move |col| SideCell { col, row });
        let cells = (1..=rows).flat_map(row).filter(|c| Some(*c) != key);
        let [col, row] = self.layer.next;
        LayerGrid {
            cells: cells.collect(),
            next: SideCell { col, row },
        }
    }

    /// True when side key `id` hides while the layer shows: every side key but the one that shows the layer.
    #[must_use]
    pub fn layer_hides(&self, id: &str) -> bool {
        self.side
            .iter()
            .any(|s| s.id == id && s.role != Role::Layer)
    }

    /// The side key that shows the layer, if any.
    fn layer_key(&self) -> Option<&SideSpec> {
        self.side.iter().find(|s| s.role == Role::Layer)
    }

    /// Refuses a second side key that shows the layer, a Next cell off the side grid or on that key, and a board id
    /// in the form of one the layer generates.
    pub(super) fn check_layer(&self) -> Result<(), BoardError> {
        let mut keys = self.side.iter().filter(|s| s.role == Role::Layer);
        let first = keys.next();
        if let Some(second) = keys.next() {
            return Err(BoardError::Role(second.id.clone()));
        }
        let [col, row] = self.layer.next;
        let on_key = first.is_some_and(|k| (k.col, k.row) == (col, row));
        if on_key || !self.on_grid(col, row) {
            return Err(BoardError::Next);
        }
        let kept = |id: &str| id == NEXT_ID || id.starts_with(SHORTCUT_PREFIX);
        match self.roles().into_iter().find(|(id, ..)| kept(id)) {
            Some((id, ..)) => Err(BoardError::Reserved(id)),
            None => Ok(()),
        }
    }
}

/// The id of the layer key that sends shortcut `id`.
#[must_use]
pub fn shortcut_key_id(id: &ShortcutId) -> String {
    format!("{SHORTCUT_PREFIX}{}", id.as_str())
}
