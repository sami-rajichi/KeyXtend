//! The hidden-run probe on Windows (ADR-0037): a program started from a packaged app writes its new app-data files to
//! that package's private copy, so the probe looks for its file there. The rules are `kx_platform::mirror`'s; this
//! module gives them the table, the real disk and the folders.

mod disk;
#[cfg(test)]
mod tests;

use std::fmt::Write as _;
use std::path::{Component, Path, PathBuf};
use std::sync::LazyLock;

use kx_platform::mirror::{self, Mirrors};
use kx_platform::ui::{Hidden, HiddenCopy, HiddenError, Probe};
use serde::Deserialize;
use thiserror::Error;

use disk::WinMirrorDisk;

/// The package table, compiled in so it is never read from disk.
const TABLE: &str = include_str!("redirect.toml");

/// The shipped table, parsed and, when broken, logged once; tests prove it parses.
static SHIPPED: LazyLock<Result<Table, TableError>> = LazyLock::new(|| {
    let parsed = Table::parse(TABLE);
    if let Err(error) = &parsed {
        tracing::warn!(%error, "the package table does not parse");
    }
    parsed
});

/// The shipped table. One that does not parse makes each list, delete and probe under the app-data folder
/// `Unreadable`, so nothing is ever deleted; a probe elsewhere stays `Normal`.
fn table() -> Result<&'static Table, HiddenError> {
    SHIPPED.as_ref().map_err(|_| HiddenError::Unreadable)
}

/// Why the package table is unusable.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
enum TableError {
    /// The table is not TOML of the expected shape, or names a key it does not know.
    #[error("the package table does not parse: {0}")]
    Parse(String),
    /// A value is empty, or a folder name or the probe's start is no plain name.
    #[error("the package table has an empty value or a name that is no plain name")]
    Bad,
}

/// The package table: the packages root, the mirror in each package, the app name's end and the probe name.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Table {
    /// The folder of package folders inside the local app-data folder.
    packages: String,
    /// The private copy of the local app-data folder inside a package folder, folder by folder.
    mirror: Vec<String>,
    /// Ends the app's name in a package folder's name.
    app_end: String,
    /// The start of a probe file's name.
    probe: String,
    /// The random bytes after it, as hex digits.
    probe_bytes: usize,
}

impl Table {
    /// Reads and checks the table in `text`.
    fn parse(text: &str) -> Result<Self, TableError> {
        let t: Self = toml::from_str(text).map_err(|e| TableError::Parse(e.to_string()))?;
        let plain = t
            .mirror
            .iter()
            .chain([&t.packages, &t.probe])
            .all(|n| one_name(n));
        if !plain || t.mirror.is_empty() || t.app_end.is_empty() || t.probe_bytes == 0 {
            return Err(TableError::Bad);
        }
        Ok(t)
    }

    /// The mirrors of the local app-data folder `local`, whose packages are in `root`.
    fn mirrors(&self, local: PathBuf, root: PathBuf) -> Mirrors {
        Mirrors {
            local,
            root,
            mirror: self.mirror.iter().collect(),
            app_end: self.app_end.clone(),
        }
    }

    /// A fresh probe name: the table's start, then random hex digits.
    fn probe_name(&self) -> Result<String, HiddenError> {
        let mut bytes = vec![0; self.probe_bytes];
        getrandom::fill(&mut bytes).map_err(|_| HiddenError::Unreadable)?;
        let mut name = self.probe.clone();
        for b in bytes {
            // Writing to a `String` cannot fail.
            let _ = write!(name, "{b:02x}");
        }
        Ok(name)
    }
}

/// Whether `name` is one plain name, with no separator, drive or `..`.
pub(crate) fn one_name(name: &str) -> bool {
    let mut parts = Path::new(name).components();
    matches!((parts.next(), parts.next()), (Some(Component::Normal(n)), None) if n == name)
}

/// The hidden-run port on Windows: the probe and the copies in each package's private copy of the local app-data
/// folder. It has no `Debug`, as its folders name the user.
pub struct WinHidden {
    /// The local app-data folder; `None` when Windows did not name it.
    local: Option<PathBuf>,
    /// The packages root; `None` without the local folder or the table.
    root: Option<PathBuf>,
}

impl WinHidden {
    /// The port over the user's local app-data folder. When Windows names none, no data folder lies under it, so each
    /// probe is normal and each list or delete `Unreadable`.
    #[cfg(windows)]
    #[must_use]
    pub fn new() -> Self {
        let local = crate::dirs::local_data().ok();
        let root = local
            .as_ref()
            .zip(table().ok())
            .map(|(l, t)| l.join(&t.packages));
        Self { local, root }
    }

    /// The port over the local app-data folder `local`, whose packages are in `root`, as tests build them.
    #[must_use]
    pub fn at(local: PathBuf, root: PathBuf) -> Self {
        Self {
            local: Some(local),
            root: Some(root),
        }
    }

    /// The mirrors and the table, or `Unreadable` without either.
    fn mirrors(&self) -> Result<(Mirrors, &'static Table), HiddenError> {
        let table = table()?;
        let local = self.local.clone().ok_or(HiddenError::Unreadable)?;
        let root = self.root.clone().ok_or(HiddenError::Unreadable)?;
        Ok((table.mirrors(local, root), table))
    }
}

#[cfg(windows)]
impl Default for WinHidden {
    fn default() -> Self {
        Self::new()
    }
}

impl Hidden for WinHidden {
    fn probe(&self, data_dir: &Path) -> Result<Probe, HiddenError> {
        // Before the table, the packages root or the random name, so a folder elsewhere never fails.
        let under = self
            .local
            .as_deref()
            .is_some_and(|l| mirror::under(data_dir, l));
        if !under {
            return Ok(Probe::Normal);
        }
        let (at, table) = self.mirrors()?;
        mirror::probe(&WinMirrorDisk, &at, data_dir, &table.probe_name()?)
    }

    fn copies(&self) -> Result<Vec<HiddenCopy>, HiddenError> {
        mirror::copies(&WinMirrorDisk, &self.mirrors()?.0)
    }

    fn remove(&self, copy: &HiddenCopy) -> Result<(), HiddenError> {
        mirror::remove(&WinMirrorDisk, &self.mirrors()?.0, copy)
    }
}
