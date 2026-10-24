//! The focused box (ADR-0033): its kind on each focus change, from UI Automation and the table in `field.toml`, and at
//! most a few words before its caret on ask, never in a password or unknown box. The rules, the table and the run are
//! pure; `uia` and `win` talk to UI Automation.
#![cfg_attr(
    not(windows),
    allow(
        dead_code,
        reason = "Only the Windows watch runs this logic; tests cover it everywhere."
    )
)]

#[cfg(windows)]
mod focus;
mod kinds;
mod run;
#[cfg(test)]
#[cfg(windows)]
mod testedit;
#[cfg(windows)]
mod uia;
#[cfg(windows)]
mod win;

use std::sync::LazyLock;

use serde::Deserialize;
use thiserror::Error;

pub(crate) use kinds::{Facts, kind};
#[cfg(windows)]
pub use win::WinFieldWatch;

/// The kind table, compiled in so it is never read from disk.
const TABLE: &str = include_str!("field.toml");

/// The longest text one read takes before the caret, in UTF-16 units; a longer one is cut to whole words.
pub const READ_CAP: usize = 1024;

/// The shipped table, parsed once; tests prove it parses, so the empty fallback never runs.
static SHIPPED: LazyLock<Table> = LazyLock::new(|| Table::parse(TABLE).unwrap_or_default());

/// The shipped kind table.
pub(crate) fn table() -> &'static Table {
    &SHIPPED
}

/// Why a kind table is unusable.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub(crate) enum TableError {
    /// The table is not TOML of the expected shape, or names a key it does not know.
    #[error("the field table does not parse: {0}")]
    Parse(String),
    /// A list, a class name or the style bit is empty.
    #[error("the field table has an empty list, class name or style bit")]
    Empty,
}

/// The kind table: Windows' numbers and class names for telling a box's kind.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Table {
    /// UI Automation control types that hold editable text.
    text_controls: Vec<i32>,
    /// The edit style bit that hides what is typed.
    password_style: u32,
    /// Base window classes whose password style bit means a password box.
    edit_classes: Vec<String>,
    /// Window classes that hide their boxes from UI Automation.
    opaque_classes: Vec<String>,
}

impl Table {
    /// Reads and checks the table in `text`.
    pub(crate) fn parse(text: &str) -> Result<Self, TableError> {
        let table: Self = toml::from_str(text).map_err(|e| TableError::Parse(e.to_string()))?;
        let lists = [&table.edit_classes, &table.opaque_classes];
        let blank = lists
            .iter()
            .any(|l| l.is_empty() || l.iter().any(|n| n.trim().is_empty()));
        if blank || table.text_controls.is_empty() || table.password_style == 0 {
            return Err(TableError::Empty);
        }
        Ok(table)
    }

    /// Whether a window of base class `class` with `style` is an edit box that hides what is typed.
    pub(crate) fn hides(&self, class: &str, style: u32) -> bool {
        style & self.password_style != 0 && listed(&self.edit_classes, class)
    }

    /// Whether windows of class `class` hide their boxes from UI Automation.
    pub(crate) fn opaque(&self, class: &str) -> bool {
        listed(&self.opaque_classes, class)
    }

    /// Whether control type `control` holds editable text.
    pub(crate) fn holds_text(&self, control: i32) -> bool {
        self.text_controls.contains(&control)
    }

    /// The edit style bit that hides what is typed.
    #[cfg(test)]
    pub(crate) fn password_style(&self) -> u32 {
        self.password_style
    }

    /// The control types that hold editable text.
    #[cfg(test)]
    pub(crate) fn text_controls(&self) -> &[i32] {
        &self.text_controls
    }
}

/// Whether `class` is in `names`, ignoring case as Windows does; an empty class never is.
fn listed(names: &[String], class: &str) -> bool {
    !class.is_empty() && names.iter().any(|n| n.eq_ignore_ascii_case(class))
}

/// `text` after its first word, which a cut at the read cap may have split; empty when it holds one word.
pub(crate) fn after_first(text: &str) -> &str {
    text.find(char::is_whitespace).map_or("", |at| &text[at..])
}

#[cfg(test)]
mod tests;
