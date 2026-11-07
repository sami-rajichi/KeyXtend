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
