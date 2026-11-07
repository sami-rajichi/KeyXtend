//! A run whose saves would be hidden saves nothing (ADR-0037): it reads nothing from the data folder, keeps its words,
//! copies and notebook in memory with Quick-fill off, logs to standard error alone and shows a banner. Only a normal
//! run asks about the hidden copies earlier runs left.

mod copies;
#[cfg(test)]
pub(crate) mod fixture;

pub use copies::{Asked, Deletes};

use crate::log::{self, Log};
use kx_crypto::{DiskRoom, KeyGuard, MemoryShelf, MemoryStore, SealedFiles, SealedShelfFiles};
use kx_kernel::{Kernel, KernelError, KernelSettings};
use kx_module_api::ports::{SealedShelf, SealedStore};
use kx_module_api::{ModuleId, Notice, NoticeArg, Redacted, args, keys};
use kx_platform::ui::{Hidden, HiddenError, Probe};
use kx_settings::{Files, peek};
use kx_ui::Hub;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// The modules a hidden run keeps off: Quick-fill, which never opens its notebook there (ADR-0037 decision 2).
pub const KEPT_OFF: &[ModuleId] = &[
    #[cfg(feature = "mod-vault")]
    kx_mod_vault::ID,
];

/// What the probe made of this run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Run {
    /// Other programs see what this run saves.
    Normal,
    /// The system would hide this run's saves in another app's private copy, so it reads and writes nothing there.
    Hidden {
        /// The app's short name, for the banner.
        app: Redacted<String>,
        /// The modules kept off for this run alone.
        off: Vec<ModuleId>,
    },
}

/// The run, and why the probe failed when it did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Probed {
    /// The run; normal when the probe failed.
    pub run: Run,
    /// The probe's error, which names no folder.
    pub error: Option<HiddenError>,
}

impl Probed {
    /// Logs once how the probe went: a failed probe, or a run that saves nothing; never a folder or an app.
    pub fn tell(&self) {
        if let Some(e) = self.error {
            tracing::warn!(error = %e, "the data folder could not be probed, so this run saves as usual");
        }
        if self.run.is_hidden() {
            tracing::warn!("this run was started from inside another app, so it saves nothing");
        }
    }
}

impl Run {
    /// The run `hidden` finds for `data_dir`; a failed probe is a normal run.
    pub fn probe(hidden: &dyn Hidden, data_dir: &Path) -> Probed {
        let (run, error) = match hidden.probe(data_dir) {
            Ok(Probe::Normal) => (Self::Normal, None),
            Ok(Probe::Hidden { app, .. }) => (Self::hidden(app), None),
            Err(e) => (Self::Normal, Some(e)),
        };
        Probed { run, error }
    }

    /// A run started from inside app `app`, keeping `KEPT_OFF` off.
    #[must_use]
    pub fn hidden(app: String) -> Self {
        Self::Hidden {
            app: Redacted::new(app),
            off: KEPT_OFF.to_vec(),
        }
    }

    /// True when this run saves nothing.
    #[must_use]
    pub fn is_hidden(&self) -> bool {
        matches!(self, Self::Hidden { .. })
    }

    /// The user's settings file in `files`, read without changing it, for the single-copy reach; none when hidden.
    #[must_use]
    pub fn peek(&self, files: &Files) -> Option<toml::Table> {
        if self.is_hidden() { None } else { peek(files) }
    }

    /// A store of its own: sealed files in `dir` under `guard`'s key, or memory alone when hidden, so the service is
    /// there even while its module is kept off.
    pub fn store(
        &self,
        dir: PathBuf,
        guard: impl FnOnce() -> Box<dyn KeyGuard>,
    ) -> Arc<dyn SealedStore> {
        if self.is_hidden() {
            Arc::new(MemoryStore::new())
        } else {
            Arc::new(SealedFiles::new(dir, guard()))
        }
    }

    /// The clipboard history's shelf: sealed day folders in `dir`, or memory alone when hidden; `disk` tells its room.
    pub fn shelf(
        &self,
        dir: PathBuf,
        guard: impl FnOnce() -> Box<dyn KeyGuard>,
        disk: Box<dyn DiskRoom>,
    ) -> Arc<dyn SealedShelf> {
        if self.is_hidden() {
            Arc::new(MemoryShelf::new(dir, disk))
        } else {
            Arc::new(SealedShelfFiles::new(dir, guard(), disk))
        }
    }

    /// Makes the folder `dir` unless hidden; the error's kind when it could not be made.
    #[must_use]
    pub fn make_dir(&self, dir: &Path) -> Option<ErrorKind> {
        if self.is_hidden() {
            return None;
        }
        std::fs::create_dir_all(dir).err().map(|e| e.kind())
    }

    /// Readies `kernel` and `hub` before the settings load: a hidden run boots fresh, keeps its modules off and shows
    /// its banner.
    ///
    /// # Errors
    /// `WrongPhase` when the kernel loaded already.
    pub fn ready(&self, kernel: &mut Kernel, hub: &Hub) -> Result<(), KernelError> {
        if let Self::Hidden { off, .. } = self {
            kernel.fresh(off)?;
        }
        if let Some(banner) = self.banner() {
            hub.notice(banner);
        }
        Ok(())
    }

    /// The banner a hidden run shows, naming the app it was started from.
    #[must_use]
    pub fn banner(&self) -> Option<Notice> {
        match self {
            Self::Normal => None,
            Self::Hidden { app, .. } => Some(about_app(keys::HIDDEN_RUN, app.expose())),
        }
    }

    /// This run's log at the level of the kernel's `loaded` settings, or its defaults': a file in `files`, or standard
    /// error alone when hidden. `None` when no level reads.
    #[must_use]
    pub fn log(&self, loaded: Option<KernelSettings>, files: &Files) -> Option<Log> {
        let settings = log::or_defaults(loaded)?;
        if self.is_hidden() {
            log::unfiled(&settings)
        } else {
            log::open(&settings, files)
        }
    }

    /// Asks through `port` about each hidden copy `kernel` does not keep, once the modules started. A hidden run asks
    /// nothing, as the list would hold the very copy that hides it.
    pub fn ask(&self, port: Arc<dyn Hidden>, kernel: &Kernel, hub: &Arc<Hub>) -> Option<Asked> {
        (!self.is_hidden()).then(|| Asked::ask(port, kernel, hub))
    }
}

/// Notice `key`, whose text names app `app`.
fn about_app(key: &'static str, app: &str) -> Notice {
    Notice {
        args: vec![(args::APP, NoticeArg::Text(app.to_owned()))],
        ..Notice::plain(key)
    }
}

#[cfg(test)]
mod tests;
