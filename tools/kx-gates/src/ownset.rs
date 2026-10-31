//! The owner's settings file around a gate that writes its own: the file and its last good copy kept byte for byte,
//! first in a sidecar file that outlives a hard stop, the gate's predict edits, and both files put back after.
#![cfg(windows)]

mod saved;

use std::fs;
use std::path::{Path, PathBuf};

use kx_kernel::{KERNEL, KernelSettings, SPEC};
use kx_mod_predict::{LEARN_KEY, ON_KEY};
use kx_module_api::ModuleId;
use kx_settings::{Files, VERSION_KEY, peek_section};
use toml::{Table, Value};

use saved::Pair;

use crate::product::NAME;

/// The owner's settings file and its last good copy, kept until `put_back`, or until dropped (a drop guard).
#[derive(Debug)]
pub struct Kept {
    pair: Pair,
    side: PathBuf,
    done: bool,
    /// How the sidecar comes back when it stays.
    later: String,
}

/// How the owner's sidecar comes back after a stop: `close`, which reads the same out folder.
fn by_close() -> String {
    format!(
        "once no copy runs, the same kx-gates' `close {NAME}` restores from it, as it reads its own out folder"
    )
}

impl Kept {
    /// Keeps the settings file and its last good copy in `files`, writing their bytes to the sidecar `side` before
    /// any gate write. A sidecar a stopped run left is restored first and kept as it is: whether one was.
    pub fn take(files: &Files, side: &Path) -> Result<(Self, bool), String> {
        Self::take_as(files, side, &by_close())
    }

    /// As `take`, for a sidecar that `later` says how to restore when it stays.
    pub fn take_as(files: &Files, side: &Path, later: &str) -> Result<(Self, bool), String> {
        let left = saved::load(side)?;
        let recovered = left.is_some();
        let pair = match left {
            Some(pair) if !pair.keeps(files) => return Err(another(side)),
            Some(pair) => put(&pair, side, later).map(|()| pair)?,
            None => {
                let pair = Pair::of(files)?;
                saved::save(side, &pair)?;
                pair
            }
        };
        let (side, later, done) = (side.to_path_buf(), later.to_string(), false);
        Ok((
            Self {
                pair,
                side,
                done,
                later,
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

    /// Puts both files back as they were, now, and deletes the sidecar once both read back as kept. It stays when
    /// `copy_runs`, as that copy may write over them when it ends.
    pub fn put_back(mut self, copy_runs: bool) -> Result<(), String> {
        self.done = true;
        put(&self.pair, &self.side, &self.later)?;
        if copy_runs {
            return Err(stays(&self.side, "a copy may still run", &self.later));
        }
        saved::drop_side(&self.side)
    }
}

impl Drop for Kept {
    /// Puts both files back but keeps the sidecar, as a drop cannot know that every copy ended.
    fn drop(&mut self) {
        if !self.done {
            let _ = self.pair.restore();
        }
    }
}

/// Why sidecar `side` stays: `why`, and `later`, how to restore from it.
fn stays(side: &Path, why: &str, later: &str) -> String {
    format!("{} stays, as {why}; {later}", side.display())
}

/// Refuses sidecar `side`, which keeps another data folder's settings.
fn another(side: &Path) -> String {
    format!(
        "{} keeps another data folder's settings; check it, then delete it",
        side.display()
    )
}

/// Restores the owner's `files` from sidecar `side` once no copy runs: the copy `close` asked to quit `ended`, and
/// none is `running`. It then deletes the sidecar once both read back as kept: whether there was one.
pub fn recover(
    files: &Files,
    side: &Path,
    ended: bool,
    running: impl FnOnce() -> bool,
) -> Result<bool, String> {
    let Some(pair) = saved::load(side)? else {
        return Ok(false);
    };
    if !ended {
        return Err(stays(
            side,
            &format!("{NAME} did not end after Quit"),
            &by_close(),
        ));
    }
    if running() {
        return Err(stays(side, &format!("{NAME} runs"), &by_close()));
    }
    if !pair.keeps(files) {
        return Err(another(side));
    }
    put(&pair, side, &by_close())?;
    saved::drop_side(side)?;
    Ok(true)
}

/// Puts `pair` back; a failure names sidecar `side`, which stays, and the way back.
fn put(pair: &Pair, side: &Path, later: &str) -> Result<(), String> {
    pair.restore()
        .map_err(|e| stays(side, &format!("the put-back failed: {e}"), later))
}

/// `owner`'s `[kernel]` section read as the store reads it, repaired value by value, with predict out of its
/// switched-off list when `on` and in it otherwise, and each of `off` in it once; the owner's version stays.
fn kernel(owner: &Table, on: bool, off: &[ModuleId]) -> Result<Table, String> {
    let fail = |e: &dyn std::fmt::Display| format!("the kernel's settings: {e}");
    let mut k: KernelSettings = peek_section(Some(owner), KERNEL, &SPEC).map_err(|e| fail(&e))?;
    let section = owner.get(KERNEL.as_str()).and_then(Value::as_table);
    let version = section.and_then(|s| s.get(VERSION_KEY)).cloned();
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
