//! The whole settings file: read it back, repairing damage, and save it without ever losing the old copy.
//! A save never leaves the settings file missing: the good file is copied, then replaced in one step.

use crate::names::{Files, MAX_FILE_BYTES};
use kx_module_api::{Notice, keys};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
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

/// Why a file gave no table.
enum Problem {
    /// The file does not exist.
    Missing,
    /// The file is too big, unreadable, not UTF-8 or not TOML.
    Damaged,
}

/// Reads one file as a table, never reading more than `MAX_FILE_BYTES` and one byte.
fn read_table(path: &Path) -> Result<toml::Table, Problem> {
    let file = File::open(path).map_err(|e| match e.kind() {
        io::ErrorKind::NotFound => Problem::Missing,
        _ => Problem::Damaged,
    })?;
    let mut bytes = Vec::new();
    file.take(MAX_FILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| Problem::Damaged)?;
    if u64::try_from(bytes.len()).map_or(true, |len| len > MAX_FILE_BYTES) {
        return Err(Problem::Damaged);
    }
    let text = String::from_utf8(bytes).map_err(|_| Problem::Damaged)?;
    text.parse().map_err(|_| Problem::Damaged)
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

/// Reads the settings file and repairs it when it is damaged.
///
/// A damaged file is moved to the broken copy, so the next save cannot make it the previous copy.
#[must_use]
pub fn load(files: &Files) -> Loaded {
    match read_table(&files.settings) {
        Ok(table) => Loaded {
            table,
            outcome: Outcome::Main,
            notices: Vec::new(),
        },
        Err(Problem::Missing) => fallback(files, false),
        Err(Problem::Damaged) => {
            set_aside(files);
            fallback(files, true)
        }
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
fn keep_previous(files: &Files) -> Result<(), SaveError> {
    match read_table(&files.settings) {
        Ok(_) => {}
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

#[cfg(test)]
mod tests;
