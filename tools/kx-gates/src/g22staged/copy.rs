//! The edges' copy of the staged worker: every file of the stage's app folder and its voice folder, the parts each edge
//! deletes from it, and edge 39's models folder with its planted part. Each folder goes at the end, on a failure or a
//! panic too; the stage itself is only read, and errors name a folder by its plain name alone.
#![cfg(windows)]

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::g22cfg::{G22Staged, is_part};

/// One edge, in the order the edges run: each deletes more of the copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Edge {
    /// Edge 39: the whole stage, a part planted beside the model.
    Plant,
    /// Edge 29: only the plainest processor part.
    Floor,
    /// Edge 28: no card part.
    NoCard,
}

impl Edge {
    /// The edges in the order they run.
    pub(super) const ORDER: [Self; 3] = [Self::Plant, Self::Floor, Self::NoCard];

    /// Its number in the plan's edge cases.
    pub(super) const fn number(self) -> u8 {
        match self {
            Self::Plant => 39,
            Self::Floor => 29,
            Self::NoCard => 28,
        }
    }
}

/// The stage's entries the copy takes, `(name, is a folder)`: every file, and of the folders only the voice files'.
pub(super) fn copied(entries: &[(String, bool)], voice: &str) -> Vec<(String, bool)> {
    let take = |(name, dir): &&(String, bool)| !dir || name == voice;
    entries.iter().filter(take).cloned().collect()
}

/// The copy's files `edge` deletes first: none for 39, every processor part but the floor for 29, the card part for 28.
pub(super) fn dropped(names: &[String], edge: Edge, g: &G22Staged, prefix: &str) -> Vec<String> {
    let pick = |n: &&String| match edge {
        Edge::Plant => false,
        Edge::Floor => is_part(n, prefix) && !n.eq_ignore_ascii_case(&g.floor_part),
        Edge::NoCard => n.eq_ignore_ascii_case(&g.card_part),
    };
    names.iter().filter(pick).cloned().collect()
}

/// Refuses a stage without the worker, the card part or the floor part, naming the first one missing.
pub(super) fn need(names: &[String], g: &G22Staged, worker: &str) -> Result<(), String> {
    let held = |want: &&str| names.iter().any(|n| n.eq_ignore_ascii_case(want));
    match [worker, g.card_part.as_str(), g.floor_part.as_str()]
        .iter()
        .find(|w| !held(w))
    {
        Some(gone) => Err(format!(
            "the stage holds no {gone}: run cargo xtask stage first"
        )),
        None => Ok(()),
    }
}

/// The entries of folder `dir`, `(name, is a folder)`.
pub(super) fn list(dir: &Path) -> Result<Vec<(String, bool)>, String> {
    let read = fs::read_dir(dir).map_err(|e| format!("the stage: {}", e.kind()))?;
    let entry = |e: io::Result<fs::DirEntry>| {
        let e = e.ok()?;
        Some((
            e.file_name().into_string().ok()?,
            e.file_type().ok()?.is_dir(),
        ))
    };
    Ok(read.filter_map(entry).collect())
}

/// The names of the files in the stage's `entries`.
pub(super) fn files(entries: &[(String, bool)]) -> Vec<String> {
    entries
        .iter()
        .filter(|(_, dir)| !dir)
        .map(|(n, _)| n.clone())
        .collect()
}

/// Copies from `stage` into `to` what `copied` picks of `entries`: each file, and the voice folder's files.
pub(super) fn fill(
    stage: &Path,
    to: &Path,
    entries: &[(String, bool)],
    voice: &str,
) -> Result<(), String> {
    let fault = |e: io::Error| format!("the copy: {}", e.kind());
    for (name, dir) in copied(entries, voice) {
        let (from, into) = (stage.join(&name), to.join(&name));
        if !dir {
            fs::copy(&from, &into).map_err(fault)?;
            continue;
        }
        fs::create_dir(&into).map_err(fault)?;
        for (file, _) in list(&from)?.into_iter().filter(|(_, d)| !d) {
            fs::copy(from.join(&file), into.join(&file)).map_err(fault)?;
        }
    }
    Ok(())
}

/// Deletes each of `names` from the copy `dir`; one already gone is fine.
pub(super) fn delete(dir: &Path, names: &[String]) -> Result<(), String> {
    for name in names {
        match fs::remove_file(dir.join(name)) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => {
                return Err(format!("{name} could not be deleted: {}", e.kind()));
            }
            _ => {}
        }
    }
    Ok(())
}

/// Edge 39's models folder `made`: the model at `model` hard-linked in, and `part` planted beside it as `plant_as`.
pub(super) fn plant(made: &Made, model: &Path, part: &Path, plant_as: &str) -> Result<(), String> {
    let name = model.file_name().ok_or("the model has no file name")?;
    let fault = |what: &str, e: io::Error| format!("{what}: {}", e.kind());
    fs::hard_link(model, made.path.join(name)).map_err(|e| fault("the model's link", e))?;
    fs::copy(part, made.path.join(plant_as)).map_err(|e| fault("the planted part", e))?;
    Ok(())
}

/// A folder the edges made; it goes at the end, on a failure or a panic too.
pub(super) struct Made {
    /// The folder.
    pub(super) path: PathBuf,
    /// Its plain name, which errors name.
    name: String,
}

impl Made {
    /// Makes `path`; one already there from an earlier run is refused, as the edges delete it.
    pub(super) fn new(path: PathBuf) -> Result<Self, String> {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if path.exists() {
            return Err(format!(
                "{name} is there from an earlier run: delete it first"
            ));
        }
        fs::create_dir(&path).map_err(|e| format!("{name}: {}", e.kind()))?;
        Ok(Self { path, name })
    }

    /// Removes the folder and all in it; one already gone is fine.
    pub(super) fn remove(&self) -> Result<(), String> {
        match fs::remove_dir_all(&self.path) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => {
                Err(format!("{} stays: {}", self.name, e.kind()))
            }
            _ => Ok(()),
        }
    }
}

impl Drop for Made {
    fn drop(&mut self) {
        let _ = self.remove();
    }
}
