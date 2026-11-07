//! The hidden-run port (ADR-0037): whether this run's saves would land in another app's private copy of the app-data
//! folder, the copies of our data folder already there, and deleting one. The OS-neutral rules live in `crate::mirror`.

use kx_module_api::Redacted;
use std::fmt;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// What the probe found about the data folder; its debug output hides the package and app, which tell the user's apps.
#[derive(Clone, PartialEq, Eq)]
pub enum Probe {
    /// Other programs see what this run saves.
    Normal,
    /// The system hides this run's saves in the private copy of the package folder `package`.
    Hidden {
        /// The package folder's name, such as `Claude_pzs8sxrjxfjjc`.
        package: String,
        /// The app's short name, such as `Claude`, for the UI.
        app: String,
    },
}

impl fmt::Debug for Probe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Normal => f.write_str("Normal"),
            Self::Hidden { package, app } => f
                .debug_struct("Hidden")
                .field("package", &Redacted::new(package))
                .field("app", &Redacted::new(app))
                .finish(),
        }
    }
}

/// A copy of our data folder inside another package's private copy of the app-data folder; its debug output hides
/// every part, as they name the user and their apps.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HiddenCopy {
    /// The copy's folder, which names the user.
    path: Redacted<PathBuf>,
    /// The package folder's name.
    package: Redacted<String>,
    /// The app's short name, for the UI.
    app: Redacted<String>,
}

impl HiddenCopy {
    /// The copy at `path` in package folder `package`, whose app is called `app`.
    #[must_use]
    pub fn new(path: PathBuf, package: String, app: String) -> Self {
        Self {
            path: Redacted::new(path),
            package: Redacted::new(package),
            app: Redacted::new(app),
        }
    }

    /// The copy's folder.
    #[must_use]
    pub fn path(&self) -> &Path {
        self.path.expose()
    }

    /// The package folder's name, such as `Claude_pzs8sxrjxfjjc`.
    #[must_use]
    pub fn package(&self) -> &str {
        self.package.expose()
    }

    /// The app's short name, such as `Claude`.
    #[must_use]
    pub fn app(&self) -> &str {
        self.app.expose()
    }
}

/// Why the probe, the list or a delete failed; no variant names a folder, so no user name reaches a log.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum HiddenError {
    /// The folders could not be read, or the probe file not written or deleted.
    #[error("the app-data folders could not be read or written")]
    Unreadable,
    /// The folder is no hidden copy that `Hidden::copies` lists now.
    #[error("the folder is no hidden copy")]
    Refused,
    /// The system would not delete the whole copy.
    #[error("the hidden copy could not be deleted")]
    Undeleted,
}

/// Finds a run whose saves would be hidden, and the hidden copies other runs left.
pub trait Hidden: Send + Sync {
    /// Whether what this run saves in `data_dir` would be hidden; it leaves nothing behind. A hidden run is told even
    /// when its probe file stays, as that file sits in the hidden copy.
    ///
    /// # Errors
    /// `Unreadable` when the folders cannot be read or the probe file not written or deleted.
    fn probe(&self, data_dir: &Path) -> Result<Probe, HiddenError>;

    /// Every copy of our data folder in another package's private copy of the app-data folder.
    ///
    /// # Errors
    /// `Unreadable` when the packages cannot be listed.
    fn copies(&self) -> Result<Vec<HiddenCopy>, HiddenError>;

    /// Deletes `copy` and everything in it at once, with no recycle bin; a link inside goes as a link, never followed.
    ///
    /// # Errors
    /// `Refused` when `copies` does not list it now, `Unreadable` when the list fails, `Undeleted` when the delete does.
    fn remove(&self, copy: &HiddenCopy) -> Result<(), HiddenError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_never_name_a_folder() {
        let all = [
            HiddenError::Unreadable,
            HiddenError::Refused,
            HiddenError::Undeleted,
        ];
        let texts = all.map(|e| e.to_string());
        assert!(texts.iter().all(|t| !t.contains('\\') && !t.contains('/')));
    }

    #[test]
    fn a_copy_keeps_its_parts_and_never_prints_them() {
        let path = PathBuf::from("secret-user-folder");
        let copy = HiddenCopy::new(path.clone(), "Pkg_x".into(), "AppName".into());
        assert_eq!(
            (copy.path(), copy.package(), copy.app()),
            (path.as_path(), "Pkg_x", "AppName")
        );
        let shown = format!("{copy:?}");
        let parts = ["secret-user-folder", "Pkg_x", "AppName"];
        assert!(parts.iter().all(|p| !shown.contains(p)), "{shown}");
    }

    #[test]
    fn a_hidden_probe_never_prints_its_package_or_app() {
        let hidden = Probe::Hidden {
            package: "Pkg_x".into(),
            app: "AppName".into(),
        };
        let shown = format!("{hidden:?}");
        let parts = ["Pkg_x", "AppName"];
        assert!(
            shown.starts_with("Hidden") && parts.iter().all(|p| !shown.contains(p)),
            "{shown}"
        );
        assert_eq!(format!("{:?}", Probe::Normal), "Normal");
    }
}
