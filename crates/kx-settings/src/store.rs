//! The `Store`: every module's settings, read with repair, changed with checks, saved as changes only.

use crate::file::{self, Outcome, SaveError};
use crate::names::{Files, VERSION_KEY};
use crate::section::{self, Section};
use kx_module_api::{ModuleId, Notice, SettingsError, SettingsSpec, keys};
use serde::ser::{Serialize, SerializeMap, Serializer};
use std::collections::{BTreeMap, BTreeSet};
use std::panic::{self, AssertUnwindSafe};
use thiserror::Error;
use toml::{Table, Value};

/// What `Store::load` found: notices for the user and modules whose spec is broken.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LoadReport {
    /// The file's notices, then one per section problem, in registration order.
    pub notices: Vec<Notice>,
    /// Modules whose spec is broken, registered twice, or whose settings code panicked.
    /// They get no settings, and their file section stays as it was.
    pub broken: Vec<ModuleId>,
}

/// Why a change or a save was refused.
#[derive(Debug, Error)]
pub enum StoreError {
    /// No registered module with a sound spec has this id.
    #[error("module {0} has no settings in the store")]
    Unknown(ModuleId),
    /// The module's section comes from a newer version and is kept as it is.
    #[error("the settings of {0} come from a newer version and cannot change")]
    Newer(ModuleId),
    /// The module's settings code panicked; nothing changed.
    #[error("the settings code of module {0} panicked")]
    Panicked(ModuleId),
    /// The version key belongs to the store, not to the module.
    #[error("{} is the section version, not a setting", VERSION_KEY)]
    Reserved,
    /// The module's validator refused the change.
    #[error(transparent)]
    Settings(#[from] SettingsError),
    /// The settings could not be written as TOML text.
    #[error("the settings cannot be written as TOML")]
    Text(#[from] toml::ser::Error),
    /// The settings file could not be saved.
    #[error(transparent)]
    File(#[from] SaveError),
}

/// A failed save, with a notice the first time saving fails since the last success.
#[derive(Debug, Error)]
#[error("the settings were not saved")]
pub struct Unsaved {
    /// Why the save failed.
    #[source]
    pub error: StoreError,
    /// The `settings-unsaved` notice, or `None` when the user was already told.
    pub notice: Option<Notice>,
}

/// Every registered module's settings, plus the file entries no module owns.
#[derive(Debug)]
pub struct Store {
    /// Where the settings file lives.
    files: Files,
    /// The registered modules with a sound spec.
    sections: BTreeMap<ModuleId, Section>,
    /// Unknown sections, top-level keys and broken modules' sections, written back as they were.
    kept: Table,
    /// True when the file does not match the store.
    unsaved: bool,
    /// True when the user was told that saving failed, until a save succeeds.
    warned: bool,
}

/// One top-level entry of the written file.
enum Entry<'a> {
    /// A kept entry, written as it was read.
    Kept(&'a Value),
    /// A module's version and changes.
    Changes(u32, &'a Table),
}

impl Serialize for Entry<'_> {
    fn serialize<S: Serializer>(&self, out: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Kept(value) => value.serialize(out),
            Self::Changes(version, changes) => {
                let mut map = out.serialize_map(None)?;
                map.serialize_entry(VERSION_KEY, version)?;
                for (key, value) in *changes {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            }
        }
    }
}

impl Store {
    /// Reads the settings file and each registered module's section, in registration order.
    #[must_use]
    pub fn load(files: Files, specs: &[(ModuleId, &'static SettingsSpec)]) -> (Self, LoadReport) {
        let loaded = file::load(&files);
        let mut store = Self {
            files,
            sections: BTreeMap::new(),
            kept: Table::new(),
            unsaved: matches!(loaded.outcome, Outcome::Restored | Outcome::Defaults),
            warned: false,
        };
        let mut report = LoadReport {
            notices: loaded.notices,
            broken: Vec::new(),
        };
        let mut table = loaded.table;
        let mut seen = BTreeSet::new();
        for &(id, spec) in specs {
            let raw = table.get(id.as_str()).cloned();
            let read = seen
                .insert(id)
                .then(|| section::guarded_read(id, spec, raw))
                .flatten();
            let Some(read) = read else {
                report.broken.push(id);
                continue;
            };
            table.remove(id.as_str());
            store.unsaved |= read.repaired;
            report.notices.extend(read.notice);
            store.sections.insert(id, read.section);
        }
        store.kept = table;
        (store, report)
    }

    /// The module's settings without the version key, ready for `parse_as`.
    #[must_use]
    pub fn get(&self, module: ModuleId) -> Option<&Table> {
        self.sections.get(&module).map(|s| &s.effective)
    }

    /// Sets one value when the module's validator accepts the result; true when the stored value changed.
    ///
    /// # Errors
    /// `Unknown`, `Newer`, `Reserved`, `Panicked`, or the validator's `Settings` error; nothing changes then.
    pub fn set(&mut self, module: ModuleId, key: &str, value: Value) -> Result<bool, StoreError> {
        let section = self
            .sections
            .get_mut(&module)
            .ok_or(StoreError::Unknown(module))?;
        if section.newer.is_some() {
            return Err(StoreError::Newer(module));
        }
        if key == VERSION_KEY {
            return Err(StoreError::Reserved);
        }
        // Nothing changes before the validator returns, so a panic leaves the section as it was.
        let changed = panic::catch_unwind(AssertUnwindSafe(|| section.set(key, value)))
            .map_err(|_| StoreError::Panicked(module))??;
        self.unsaved |= changed;
        Ok(changed)
    }

    /// True when the file does not match the store yet, after a repair at load or a change.
    #[must_use]
    pub fn needs_save(&self) -> bool {
        self.unsaved
    }

    /// The file text: each module's changes after its version, and every kept entry as it was.
    ///
    /// # Errors
    /// `Text` when TOML cannot represent a value.
    pub fn to_text(&self) -> Result<String, StoreError> {
        let mut doc: BTreeMap<&str, Entry<'_>> = self
            .kept
            .iter()
            .map(|(key, value)| (key.as_str(), Entry::Kept(value)))
            .collect();
        for (id, section) in &self.sections {
            let entry = match &section.newer {
                Some(raw) => Entry::Kept(raw),
                None if section.changes.is_empty() => continue,
                None => Entry::Changes(section.version(), &section.changes),
            };
            doc.insert(id.as_str(), entry);
        }
        Ok(toml::to_string(&doc)?)
    }

    /// Writes `to_text` to the settings file safely.
    ///
    /// # Errors
    /// `Unsaved` with the cause, and a notice the first time since the last success.
    pub fn save(&mut self) -> Result<(), Unsaved> {
        let saved = self
            .to_text()
            .and_then(|text| Ok(file::save(&self.files, &text)?));
        match saved {
            Ok(()) => {
                self.unsaved = false;
                self.warned = false;
                Ok(())
            }
            Err(error) => {
                let notice = (!self.warned).then(|| file::notice(keys::SETTINGS_UNSAVED));
                self.warned = true;
                Err(Unsaved { error, notice })
            }
        }
    }
}
