//! The owner's settings file around a gate that writes its own: the file and its last good copy kept byte for byte,
//! first in a sidecar file that outlives a hard stop, the gate's predict edits, and both files put back after.
#![cfg(windows)]

mod saved;

use std::fs;
use std::path::{Path, PathBuf};

use kx_kernel::{KERNEL, KernelSettings};
use kx_mod_predict::{LEARN_KEY, ON_KEY};
use kx_module_api::ModuleId;
use kx_settings::{Files, VERSION_KEY};
use toml::{Table, Value};

use saved::Pair;

/// The owner's settings file and its last good copy, kept until `put_back`, or until dropped (a drop guard).
#[derive(Debug)]
pub struct Kept {
    pair: Pair,
    side: PathBuf,
    done: bool,
}

impl Kept {
    /// Keeps the settings file and its last good copy in `files`, writing their bytes to the sidecar `side` before
    /// any gate write. A sidecar a stopped run left is restored first and kept as it is: whether one was.
    pub fn take(files: &Files, side: &Path) -> Result<(Self, bool), String> {
        let left = saved::load(side)?;
        let recovered = left.is_some();
        let pair = match left {
            Some(pair) if !pair.keeps(files) => {
                return Err(format!(
                    "{} keeps another data folder's settings; check it, then delete it",
                    side.display()
                ));
            }
            Some(pair) => pair.restore().map(|()| pair)?,
            None => {
                let pair = Pair::of(files)?;
                saved::save(side, &pair)?;
                pair
            }
        };
        let side = side.to_path_buf();
        Ok((
            Self {
                pair,
                side,
                done: false,
            },
            recovered,
        ))
    }

    /// The owner's settings as a table; empty when there was no file or it does not read.
    pub fn table(&self) -> Table {
        let text = self
            .pair
            .settings
            .bytes
            .as_deref()
            .map(String::from_utf8_lossy);
        text.and_then(|t| t.parse().ok()).unwrap_or_default()
    }

    /// Writes `table` as the settings file, for the gate's run.
    pub fn write(&self, table: &Table) -> Result<(), String> {
        let text = toml::to_string(table).map_err(|e| format!("the gate's settings: {e}"))?;
        let path = &self.pair.settings.path;
        fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// Puts both files back and deletes the sidecar, only once both read back as kept.
    fn finish(&self) -> Result<(), String> {
        self.pair.restore()?;
        saved::drop_side(&self.side)
    }

    /// Puts both files back as they were, now.
    pub fn put_back(mut self) -> Result<(), String> {
        self.done = true;
        self.finish()
    }
}

impl Drop for Kept {
    fn drop(&mut self) {
        if !self.done {
            let _ = self.finish();
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
