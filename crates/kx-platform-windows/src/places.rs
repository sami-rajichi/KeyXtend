//! The backup box's places on Windows (ADR-0038 decision 7): the cloud folders `places.toml` finds, the removable
//! drives Windows lists, the folder list, the boxes found in the places, and writes in one step. The table, the
//! drive names and the path rules are pure, `win` serves the port, and no path is ever logged or printed.
#![cfg_attr(
    not(windows),
    allow(
        dead_code,
        reason = "Only the Windows places run these rules; tests cover them everywhere."
    )
)]

#[cfg(windows)]
mod drives;
#[cfg(windows)]
mod file;
#[cfg(windows)]
mod list;
mod paths;
#[cfg(windows)]
mod quiet;
mod table;
#[cfg(test)]
#[cfg(windows)]
mod tests;
#[cfg(windows)]
mod win;

use std::path::PathBuf;
use std::sync::LazyLock;

use kx_module_api::Redacted;
use kx_module_api::ports::{Place, PlaceError, PlaceKind};

use table::{Table, TableError};
#[cfg(windows)]
pub use win::WinPlaces;

/// The places table, compiled in so it is never read from disk.
const TABLE: &str = include_str!("places.toml");

/// The shipped table, parsed and, when broken, logged once; tests prove it parses.
static SHIPPED: LazyLock<Result<Table, TableError>> = LazyLock::new(|| {
    let parsed = Table::parse(TABLE);
    if let Err(error) = &parsed {
        tracing::warn!(%error, "the places table does not parse");
    }
    parsed
});

/// The shipped table. One that does not parse suggests and finds nothing, and every other call is `Io`.
fn table() -> Result<&'static Table, PlaceError> {
    SHIPPED.as_ref().map_err(|_| PlaceError::Io)
}

/// A drive Windows lists, of a type the table keeps. It has no `Debug`, as a label may name the user.
#[derive(Clone)]
struct Drive {
    /// `Removable` or `Drive`, from its type.
    kind: PlaceKind,
    /// Whether it is a network drive, which only the folder list touches.
    remote: bool,
    /// Its root, such as `E:\`.
    root: PathBuf,
    /// Its volume label; empty when it has none.
    label: String,
}

impl Drive {
    /// The drive as a place, named by its letter and label.
    fn place(&self, table: &Table) -> Place {
        Place {
            kind: self.kind,
            name: Redacted::new(table.drive_name(&self.root, &self.label)),
            path: Redacted::new(self.root.clone()),
        }
    }
}

/// The fixed drives there now, A to Z, each with its root and its name as the folder list shows it; none when the
/// table does not parse.
#[cfg(windows)]
pub(crate) fn fixed_drives() -> Vec<(PathBuf, String)> {
    let Ok(table) = table() else {
        return Vec::new();
    };
    let fixed = drives::live(table)
        .into_iter()
        .filter(|d| d.kind == PlaceKind::Drive && !d.remote);
    let named = |d: Drive| {
        let name = table.drive_name(&d.root, &d.label);
        (d.root, name)
    };
    fixed.map(named).collect()
}

/// True for file attributes the table hides, such as hidden or system; an unreadable table hides everything.
pub(crate) fn hides(attributes: u32) -> bool {
    match table() {
        Ok(t) => t.hides(attributes),
        Err(_) => true,
    }
}
