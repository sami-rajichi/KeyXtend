//! The shortcuts layer: the pages it shows, whether it is up and on which page, and the keys it draws in which side
//! cells. It is pure, with no I/O and no clock.

use crate::spec::{LayerGrid, NEXT_ID, Role, SideCell, shortcut_key_id};
use kx_module_api::ports::ShortcutId;
use std::collections::HashSet;

/// A key the layer draws.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LayerKey {
    /// Sends a shortcut.
    Shortcut(ShortcutId),
    /// Shows the next page.
    Next,
}

impl LayerKey {
    /// The key's id: `sc-` and its shortcut, or `NEXT_ID`.
    #[must_use]
    pub fn id(&self) -> String {
        match self {
            Self::Shortcut(id) => shortcut_key_id(id),
            Self::Next => NEXT_ID.to_owned(),
        }
    }

    /// The key's role.
    #[must_use]
    pub fn role(&self) -> Role {
        match self {
            Self::Shortcut(_) => Role::Shortcut,
            Self::Next => Role::Next,
        }
    }
}

/// A key the layer draws, in its side cell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayerCell {
    /// The side cell.
    pub cell: SideCell,
    /// The key.
    pub key: LayerKey,
}

/// The shortcuts layer: shown or hidden, on one of the pages it shows; a hidden layer keeps its page.
#[derive(Clone, Debug)]
pub struct Layer {
    /// Where it draws.
    grid: LayerGrid,
    /// The pages it shows: known ids only, each once, no page empty or past the room.
    pages: Vec<Vec<ShortcutId>>,
    /// The current page, from 0.
    page: usize,
    /// It is shown.
    shown: bool,
}

impl Layer {
    /// A hidden layer on its first page, over `pages` less the ids not in `known`, as `renew` filters them.
    #[must_use]
    pub fn new(grid: LayerGrid, pages: &[Vec<ShortcutId>], known: &HashSet<ShortcutId>) -> Self {
        let pages = kept(&grid, pages, known);
        Self {
            grid,
            pages,
            page: 0,
            shown: false,
        }
    }

    /// Takes new pages or known ids, dropping unknown ids, repeats and pages left empty. It stays shown, a page past
    /// the end comes back to the last, and nothing left hides it.
    pub fn renew(&mut self, pages: &[Vec<ShortcutId>], known: &HashSet<ShortcutId>) {
        self.pages = kept(&self.grid, pages, known);
        self.page = self.page.min(self.pages.len().saturating_sub(1));
        self.shown &= self.any();
    }

    /// Shows or hides the layer; it cannot show while there is nothing to show.
    pub fn toggle(&mut self) {
        self.shown = !self.shown && self.any();
    }

    /// Turns to the next page, the first after the last; nothing happens while hidden or with one page.
    pub fn next_page(&mut self) {
        let count = self.pages.len();
        if self.shown && count > 1 {
            self.page = (self.page + 1) % count;
        }
    }

    /// True while the layer is shown.
    #[must_use]
    pub fn shown(&self) -> bool {
        self.shown
    }

    /// True when there is anything to show.
    #[must_use]
    pub fn any(&self) -> bool {
        !self.pages.is_empty()
    }

    /// How many pages it shows.
    #[must_use]
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// The current page, from 0.
    #[must_use]
    pub fn page(&self) -> usize {
        self.page
    }

    /// The keys to draw now: none while hidden, else the page's shortcuts in reading order, then Next when there is
    /// more than one page.
    #[must_use]
    pub fn cells(&self) -> Vec<LayerCell> {
        let Some(page) = self.pages.get(self.page).filter(|_| self.shown) else {
            return Vec::new();
        };
        let count = self.pages.len();
        let keys = page.iter().cloned().map(LayerKey::Shortcut);
        let pairs = self.grid.cells(count).into_iter().zip(keys);
        let mut cells: Vec<LayerCell> = pairs.map(|(cell, key)| LayerCell { cell, key }).collect();
        if count > 1 {
            let next = self.grid.next();
            cells.push(LayerCell {
                cell: next,
                key: LayerKey::Next,
            });
        }
        cells
    }
}

/// `pages` with only the ids in `known`, each the first time it comes, every page cut to the room on `grid`, and
/// the pages left empty dropped.
fn kept(
    grid: &LayerGrid,
    pages: &[Vec<ShortcutId>],
    known: &HashSet<ShortcutId>,
) -> Vec<Vec<ShortcutId>> {
    let mut seen = HashSet::new();
    let mut kept = Vec::new();
    for p in pages {
        let fresh = p
            .iter()
            .filter(|id| known.contains(*id) && seen.insert(*id));
        let page: Vec<ShortcutId> = fresh.cloned().collect();
        if !page.is_empty() {
            kept.push(page);
        }
    }
    let room = grid.capacity(kept.len());
    for p in &mut kept {
        p.truncate(room);
    }
    kept.retain(|p| !p.is_empty());
    kept
}

#[cfg(test)]
mod tests;
