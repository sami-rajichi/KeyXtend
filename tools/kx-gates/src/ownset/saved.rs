//! The owner's two settings files kept byte for byte, and their copy in a sidecar file that outlives a hard stop.

use std::fs;
use std::io::{ErrorKind, Read};
use std::path::{Path, PathBuf};

use kx_settings::{Files, MAX_FILE_BYTES};
use serde::{Deserialize, Serialize};

/// One kept file: its path and its bytes, none when it did not exist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct Saved {
    pub path: PathBuf,
    pub bytes: Option<Vec<u8>>,
}

/// The settings file and its last good copy, as kept.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct Pair {
    pub settings: Saved,
    pub previous: Saved,
}

/// The bytes of `path`, none when it does not exist; an error past the settings store's own cap.
fn read(path: &Path) -> Result<Option<Vec<u8>>, String> {
    let fail = |e: std::io::Error| format!("{}: {e}", path.display());
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(fail(e)),
    };
    let mut bytes = Vec::new();
    let cap = MAX_FILE_BYTES.saturating_add(1);
    file.take(cap).read_to_end(&mut bytes).map_err(fail)?;
    within_cap(path, &bytes)?;
    Ok(Some(bytes))
}

/// Refuses `bytes` for `path` past the settings store's own cap.
fn within_cap(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_FILE_BYTES {
        return Err(format!(
            "{} is over the settings store's {MAX_FILE_BYTES} bytes",
            path.display()
        ));
    }
    Ok(())
}

impl Saved {
    /// The file at `path` as it is now.
    fn of(path: &Path) -> Result<Self, String> {
        let bytes = read(path)?;
        Ok(Self {
            path: path.to_path_buf(),
            bytes,
        })
    }

    /// Writes the kept bytes back, or removes the file when there was none, then checks it byte for byte.
    fn restore(&self) -> Result<(), String> {
        let fail = |e: std::io::Error| format!("{}: {e}", self.path.display());
        match &self.bytes {
            Some(bytes) => fs::write(&self.path, bytes).map_err(fail)?,
            None if self.path.exists() => fs::remove_file(&self.path).map_err(fail)?,
            None => {}
        }
        if read(&self.path)? == self.bytes {
            Ok(())
        } else {
            Err(format!(
                "{} did not come back as it was",
                self.path.display()
            ))
        }
    }
}

impl Pair {
    /// Both files in `files` as they are now.
    pub fn of(files: &Files) -> Result<Self, String> {
        Ok(Self {
            settings: Saved::of(&files.settings)?,
            previous: Saved::of(&files.previous)?,
        })
    }

    /// True when it keeps the files in `files`.
    pub fn keeps(&self, files: &Files) -> bool {
        self.settings.path == files.settings && self.previous.path == files.previous
    }

    /// Puts both files back as kept and checks them: every error, joined.
    pub fn restore(&self) -> Result<(), String> {
        let errors: Vec<String> = [&self.settings, &self.previous]
            .into_iter()
            .filter_map(|s| s.restore().err())
            .collect();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    }
}

/// The pair a stopped run left in sidecar `side`; none when there is none. A file it holds past the settings store's
/// cap is refused before anything is written.
pub(super) fn load(side: &Path) -> Result<Option<Pair>, String> {
    let fail =
        |e: &dyn std::fmt::Display| format!("{}: {e}; check it, then delete it", side.display());
    let bytes = match fs::read(side) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(fail(&e)),
    };
    let pair: Pair = serde_json::from_slice(&bytes).map_err(|e| fail(&e))?;
    for s in [&pair.settings, &pair.previous] {
        within_cap(&s.path, s.bytes.as_deref().unwrap_or_default()).map_err(|e| fail(&e))?;
    }
    Ok(Some(pair))
}

/// Writes `pair` to sidecar `side` and checks it reads back the same.
pub(super) fn save(side: &Path, pair: &Pair) -> Result<(), String> {
    let fail = |e: &dyn std::fmt::Display| format!("{}: {e}", side.display());
    let text = serde_json::to_vec(pair).map_err(|e| fail(&e))?;
    fs::write(side, &text).map_err(|e| fail(&e))?;
    if load(side)?.as_ref() == Some(pair) {
        Ok(())
    } else {
        Err(format!("{} did not read back as written", side.display()))
    }
}

/// Deletes sidecar `side`; one already gone is fine.
pub(super) fn drop_side(side: &Path) -> Result<(), String> {
    match fs::remove_file(side) {
        Err(e) if e.kind() != ErrorKind::NotFound => Err(format!("{}: {e}", side.display())),
        _ => Ok(()),
    }
}
