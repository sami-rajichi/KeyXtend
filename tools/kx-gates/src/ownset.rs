//! The owner's settings file around a gate that writes its own: the file and its last good copy kept byte for byte,
//! the gate's predict edits, and both files put back after, also on a panic (a drop guard).
#![cfg(windows)]

use std::fs;
use std::io::{ErrorKind, Read};
use std::path::{Path, PathBuf};

use kx_kernel::{KERNEL, KernelSettings};
use kx_mod_predict::{LEARN_KEY, ON_KEY};
use kx_module_api::ModuleId;
use kx_settings::{Files, MAX_FILE_BYTES, VERSION_KEY};
use toml::{Table, Value};

/// One kept file: its path and its bytes, none when it did not exist.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Saved {
    path: PathBuf,
    bytes: Option<Vec<u8>>,
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
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_FILE_BYTES {
        return Err(format!(
            "{} is over the settings store's {MAX_FILE_BYTES} bytes",
            path.display()
        ));
    }
    Ok(Some(bytes))
}

impl Saved {
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

/// The owner's settings file and its last good copy, kept until `put_back`, or until dropped.
#[derive(Debug)]
pub struct Kept {
    settings: Saved,
    previous: Saved,
    done: bool,
}

impl Kept {
    /// Keeps the settings file and its last good copy in `files`.
    pub fn take(files: &Files) -> Result<Self, String> {
        let save = |path: &PathBuf| {
            read(path).map(|bytes| Saved {
                path: path.clone(),
                bytes,
            })
        };
        Ok(Self {
            settings: save(&files.settings)?,
            previous: save(&files.previous)?,
            done: false,
        })
    }

    /// The owner's settings as a table; empty when there was no file or it does not read.
    pub fn table(&self) -> Table {
        let text = self.settings.bytes.as_deref().map(String::from_utf8_lossy);
        text.and_then(|t| t.parse().ok()).unwrap_or_default()
    }

    /// Writes `table` as the settings file, for the gate's run.
    pub fn write(&self, table: &Table) -> Result<(), String> {
        let text = toml::to_string(table).map_err(|e| format!("the gate's settings: {e}"))?;
        let path = &self.settings.path;
        fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// Puts both files back as they were: every error, joined.
    fn restore(&self) -> Result<(), String> {
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

    /// Puts both files back as they were, now.
    pub fn put_back(mut self) -> Result<(), String> {
        self.done = true;
        self.restore()
    }
}

impl Drop for Kept {
    fn drop(&mut self) {
        if !self.done {
            let _ = self.restore();
        }
    }
}

/// `owner`'s `[kernel]` section, typed over the kernel's defaults, with predict out of its switched-off list when `on`
/// and in it otherwise, and each of `off` in it once; the owner's other choices and version stay.
fn kernel(owner: &Table, on: bool, off: &[ModuleId]) -> Result<Table, String> {
    let fail = |e: &dyn std::fmt::Display| format!("the kernel's settings: {e}");
    let defaults = KernelSettings::from_defaults().map_err(|e| fail(&e))?;
    let mut values = owner
        .get(KERNEL.as_str())
        .and_then(Value::as_table)
        .cloned()
        .unwrap_or_default();
    let version = values.remove(VERSION_KEY);
    let mut merged = Table::try_from(&defaults).map_err(|e| fail(&e))?;
    merged.extend(values);
    let mut k: KernelSettings = Value::Table(merged).try_into().unwrap_or(defaults);
    let id = kx_mod_predict::ID.as_str();
    k.disabled.retain(|d| d != id);
    let predict = (!on).then_some(kx_mod_predict::ID);
    for m in off.iter().copied().chain(predict) {
        if !k.disabled.iter().any(|d| d == m.as_str()) {
            k.disabled.push(m.as_str().to_string());
        }
    }
    let mut out = Table::try_from(&k).map_err(|e| fail(&e))?;
    out.extend(version.map(|v| (VERSION_KEY.to_string(), v)));
    Ok(out)
}

/// `owner`'s settings for a run with predict `on` or off and the modules `off` switched off. An on run also shows chips
/// and pauses learning, so the gate's words never teach the owner's learner.
pub fn predict(owner: &Table, on: bool, off: &[ModuleId]) -> Result<Table, String> {
    let mut table = owner.clone();
    table.insert(
        KERNEL.as_str().to_string(),
        Value::Table(kernel(owner, on, off)?),
    );
    if on {
        let id = kx_mod_predict::ID.as_str();
        let mut section = owner
            .get(id)
            .and_then(Value::as_table)
            .cloned()
            .unwrap_or_default();
        section.insert(ON_KEY.to_string(), Value::Boolean(true));
        section.insert(LEARN_KEY.to_string(), Value::Boolean(false));
        table.insert(id.to_string(), Value::Table(section));
    }
    Ok(table)
}

#[cfg(test)]
mod tests;
