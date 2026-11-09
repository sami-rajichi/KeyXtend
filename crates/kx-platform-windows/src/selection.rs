//! The focused box's selection (ADR-0041): its kind from UI Automation, `field.toml` and the table in `selection.toml`,
//! whether its selection holds text, its line boxes and its caret box, with no text read but at Translate's click. The
//! kind rules, the walk up to the text, the table, the boxes and where the selection handler sits are pure; `uia`,
//! `win` and `hear` talk to UI Automation, and the run is `kx_platform::selection`'s.
#![cfg_attr(
    not(windows),
    allow(
        dead_code,
        reason = "Only the Windows watch runs this logic; tests cover it everywhere."
    )
)]

mod follow;
#[cfg(windows)]
mod hear;
mod kinds;
mod spot;
#[cfg(windows)]
mod uia;
#[cfg(windows)]
mod win;

use std::sync::LazyLock;

use serde::Deserialize;
use thiserror::Error;

use crate::MAX_DEPTH;
use crate::field::listed;
pub use spot::{join, line, middle};
#[cfg(windows)]
pub use win::WinSelectionWatch;

/// The kind table, compiled in so it is never read from disk.
const TABLE: &str = include_str!("selection.toml");

/// The shipped table, parsed once; tests prove it parses, so the closed fallback never runs.
static SHIPPED: LazyLock<Table> = LazyLock::new(|| shipped(TABLE));

/// The table in `text`; when it does not parse, a closed one, under which every box is closed.
fn shipped(text: &str) -> Table {
    Table::parse(text).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "no box shows selection chips");
        Table::closed()
    })
}

/// The shipped kind table.
pub(crate) fn table() -> &'static Table {
    &SHIPPED
}

/// Why a kind table is unusable.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub(crate) enum TableError {
    /// The table is not TOML of the expected shape, or names a key it does not know.
    #[error("the selection table does not parse: {0}")]
    Parse(String),
    /// A list or a class name is empty.
    #[error("the selection table has an empty list or class name")]
    Empty,
    /// A control type is both editable and read-only.
    #[error("the selection table lists a control type twice")]
    Twice,
    /// The walk up is 0, which turns it off, or deeper than any UI Automation walk.
    #[error("the selection table's walk_up is not 1 to {max}", max = MAX_DEPTH)]
    Depth,
}

/// The kind table: Windows' control types and window classes for telling a box's kind.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Table {
    /// UI Automation control types whose text may be edited.
    edit_controls: Vec<i32>,
    /// Control types whose text is only ever read.
    read_controls: Vec<i32>,
    /// Window classes of terminals.
    terminal_classes: Vec<String>,
    /// How many levels a look climbs for a text pattern above a focused element that has none.
    walk_up: u8,
    /// Set only on the fallback for a table that does not parse.
    #[serde(skip)]
    closed: bool,
}

impl Table {
    /// A table that lists nothing, under which every box is closed.
    pub(crate) fn closed() -> Self {
        Self {
            edit_controls: Vec::new(),
            read_controls: Vec::new(),
            terminal_classes: Vec::new(),
            walk_up: 0,
            closed: true,
        }
    }

    /// Whether this is the fallback under which every box is closed.
    pub(crate) fn is_closed(&self) -> bool {
        self.closed
    }

    /// Reads and checks the table in `text`.
    pub(crate) fn parse(text: &str) -> Result<Self, TableError> {
        let table: Self = toml::from_str(text).map_err(|e| TableError::Parse(e.to_string()))?;
        let lists = [&table.edit_controls, &table.read_controls];
        let names = &table.terminal_classes;
        let blank = names.iter().any(|n| n.trim().is_empty());
        if blank || names.is_empty() || lists.iter().any(|l| l.is_empty()) {
            return Err(TableError::Empty);
        }
        if table.edit_controls.iter().any(|c| table.reads(*c)) {
            return Err(TableError::Twice);
        }
        if !(1..=MAX_DEPTH).contains(&usize::from(table.walk_up)) {
            return Err(TableError::Depth);
        }
        Ok(table)
    }

    /// Whether control type `control` holds text that may be edited.
    pub(crate) fn edits(&self, control: i32) -> bool {
        self.edit_controls.contains(&control)
    }

    /// Whether control type `control` holds text that is only ever read.
    pub(crate) fn reads(&self, control: i32) -> bool {
        self.read_controls.contains(&control)
    }

    /// Whether windows of class `class` are terminals.
    pub(crate) fn terminal(&self, class: &str) -> bool {
        listed(&self.terminal_classes, class)
    }

    /// How many levels a look climbs for a text pattern above a focused element that has none.
    pub(crate) fn walk_up(&self) -> u8 {
        self.walk_up
    }

    /// The editable and the read-only control types.
    #[cfg(test)]
    pub(crate) fn controls(&self) -> (&[i32], &[i32]) {
        (&self.edit_controls, &self.read_controls)
    }
}

#[cfg(test)]
mod tests;
