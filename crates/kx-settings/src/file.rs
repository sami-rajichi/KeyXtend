//! The whole settings file: read it back, repairing damage, and save it without ever losing the old copy.
//! A save never leaves the settings file missing: the good file is copied, then replaced in one step.

use crate::names::Files;
use crate::section;
use kx_module_api::{ModuleId, Notice, Settings, SettingsError, SettingsSpec, keys, parse_as};
use read::{Problem, read_table};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;
use thiserror::Error;

/// Where the loaded table came from.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The settings file was read.
    Main,
    /// The previous copy was used: the settings file was damaged or lost.
    Restored,
    /// There was no settings file and no usable previous copy, so it starts empty.
    Fresh,
    /// Nothing was usable, so it starts empty and a damaged copy was kept.
    Defaults,
    /// Another program holds the settings file, so it starts empty and must not save this run.
    Locked,
}

/// The whole-file table and what happened while reading it.
#[derive(Clone, Debug, PartialEq)]
pub struct Loaded {
    /// The file's table; empty when it starts clean.
    pub table: toml::Table,
    /// Where the table came from.
    pub outcome: Outcome,
    /// Messages for the user; empty when all is well.
    pub notices: Vec<Notice>,
}

/// Why a save failed; the old text stays in the settings file.
/// Its texts name the step only, since the kernel logs them and a path holds the account name.
#[derive(Debug, Error)]
pub enum SaveError {
    /// The data folder could not be created.
    #[error("cannot create the data folder")]
    CreateDir {
        /// The folder.
        path: PathBuf,
        /// The cause.
        #[source]
        source: io::Error,
    },
    /// The temp file could not be written and synced.
    #[error("cannot write the temp file")]
    WriteTemp {
        /// The temp file.
        path: PathBuf,
        /// The cause.
        #[source]
        source: io::Error,
    },
    /// The settings file could not be copied to the previous copy.
    #[error("cannot keep the previous copy")]
    KeepPrevious {
        /// The previous copy.
        path: PathBuf,
        /// The cause.
        #[source]
        source: io::Error,
    },
    /// The temp file could not replace the settings file, which is left as it was.
    #[error("cannot put the new settings in place")]
    Replace {
        /// The settings file.
        path: PathBuf,
        /// The cause.
        #[source]
        source: io::Error,
    },
}

/// A notice with only its key, about no module.
pub(crate) fn notice(key: &'static str) -> Notice {
    Notice {
        key,
        module: None,
        args: Vec::new(),
    }
}

/// What to use when the settings file gave nothing: the previous copy, else an empty table.
fn fallback(files: &Files, damaged: bool) -> Loaded {
    let (table, outcome, notices) = match read_table(&files.previous) {
        Ok(table) => (
            table,
            Outcome::Restored,
            vec![notice(keys::SETTINGS_RESTORED)],
        ),
        Err(_) if damaged => (
            toml::Table::new(),
            Outcome::Defaults,
            vec![notice(keys::SETTINGS_DEFAULTS)],
        ),
        Err(_) => (toml::Table::new(), Outcome::Fresh, Vec::new()),
    };
    Loaded {
        table,
        outcome,
        notices,
    }
}

/// The settings file's table as it is, for a later copy of the app that must not change it: nothing when it is
/// missing, held, too big or damaged. It never repairs, moves or writes a file.
#[must_use]
pub fn peek(files: &Files) -> Option<toml::Table> {
    read_table(&files.settings).ok()
}

/// Module `id`'s section of a peeked settings table `file`, read as the store reads it, for a later copy of the app:
/// migrated, repaired value by value, and the defaults for a newer version or any fault.
///
/// # Errors
/// When `spec`'s own defaults do not read as `T`.
pub fn peek_section<T: Settings>(
    file: Option<&toml::Table>,
    id: ModuleId,
    spec: &'static SettingsSpec,
) -> Result<T, SettingsError> {
    let defaults = spec.defaults_as::<T>()?;
    let raw = file.and_then(|f| f.get(id.as_str())).cloned();
    let read = section::guarded_read(id, spec, raw);
    let effective = read.and_then(|r| parse_as::<T>(&r.section.effective).ok());
    Ok(effective.unwrap_or(defaults))
}

/// Reads the settings file and repairs it when it is damaged.
///
/// A damaged file is moved to the broken copy, so the next save cannot make it the previous copy.
/// A locked file is left alone: nothing is restored, and the store must not save this run.
#[must_use]
pub fn load(files: &Files) -> Loaded {
    let (table, outcome, notices) = match read_table(&files.settings) {
        Ok(table) => (table, Outcome::Main, Vec::new()),
        Err(Problem::Missing) => return fallback(files, false),
        Err(Problem::Damaged) => {
            set_aside(files);
            return fallback(files, true);
        }
        Err(Problem::Locked) => (
            toml::Table::new(),
            Outcome::Locked,
            vec![notice(keys::SETTINGS_LOCKED)],
        ),
    };
    Loaded {
        table,
        outcome,
        notices,
    }
}

/// Moves a damaged settings file to the broken copy; if the move fails the file stays.
fn set_aside(files: &Files) {
    let _ = fs::rename(&files.settings, &files.broken);
}

/// Writes `text` to the temp file and syncs it to disk.
fn write_temp(files: &Files, text: &str) -> io::Result<()> {
    let mut file = File::create(&files.temp)?;
    file.write_all(text.as_bytes())?;
    file.sync_all()
}

/// Copies a settings file that reads well to the previous copy; the file itself stays where it is.
///
/// A damaged file is moved to the broken copy instead, so it never replaces a good previous copy.
/// A locked file fails the copy, so the save stops and the file stays as it was.
fn keep_previous(files: &Files) -> Result<(), SaveError> {
    match read_table(&files.settings) {
        Ok(_) | Err(Problem::Locked) => {}
        Err(Problem::Missing) => return Ok(()),
        Err(Problem::Damaged) => {
            set_aside(files);
            return Ok(());
        }
    }
    match fs::copy(&files.settings, &files.previous) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(SaveError::KeepPrevious {
            path: files.previous.clone(),
            source,
        }),
    }
}

/// The save steps, in order; the settings file is replaced only by the last one, in one step.
fn save_steps(files: &Files, text: &str) -> Result<(), SaveError> {
    fs::create_dir_all(&files.dir).map_err(|source| SaveError::CreateDir {
        path: files.dir.clone(),
        source,
    })?;
    write_temp(files, text).map_err(|source| SaveError::WriteTemp {
        path: files.temp.clone(),
        source,
    })?;
    keep_previous(files)?;
    fs::rename(&files.temp, &files.settings).map_err(|source| SaveError::Replace {
        path: files.settings.clone(),
        source,
    })
}

/// Saves the full settings text; the old file stays if any step fails.
///
/// # Errors
/// `SaveError` naming the step that failed, with the file's path in its field.
pub fn save(files: &Files, text: &str) -> Result<(), SaveError> {
    let result = save_steps(files, text);
    if result.is_err() {
        // A leftover temp file is harmless and the next save overwrites it, so a failed delete is ignored.
        let _ = fs::remove_file(&files.temp);
    }
    result
}

mod read;

#[cfg(test)]
mod tests;
