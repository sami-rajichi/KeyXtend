//! `WinPlaces`: the places port over the table, the environment, the drives Windows lists and the disk room.

use std::cmp::Reverse;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use kx_crypto::DiskRoom;
use kx_module_api::Redacted;
use kx_module_api::ports::{Found, Place, PlaceError, PlaceKind, PlaceState, Places};

use super::quiet::Quiet;
use super::table::{Cloud, Table};
use super::{Drive, drives, file, list, paths, table};
use crate::disk::WinDiskRoom;

/// Lists the drives Windows has now.
type Lister = Box<dyn Fn() -> Vec<Drive> + Send + Sync>;

/// The places port on Windows. It has no `Debug`, as its folders name the user.
pub struct WinPlaces {
    /// The folder each environment variable the table names holds.
    env: HashMap<String, PathBuf>,
    drives: Lister,
    room: Box<dyn DiskRoom>,
    /// The data folder, which no box may go in, above or below.
    data: PathBuf,
}

impl WinPlaces {
    /// The places over this session's environment, Windows' drives and their free space; `data` is refused as one.
    #[must_use]
    pub fn new(data: PathBuf) -> Self {
        let env = table().map(env).unwrap_or_default();
        let lister = || table().map(drives::live).unwrap_or_default();
        Self::at(env, Box::new(lister), Box::new(WinDiskRoom), data)
    }

    /// The places over `env`, the drives `drives` lists, `room` and the data folder `data`, as tests build them.
    pub(super) fn at(
        env: HashMap<String, PathBuf>,
        drives: Lister,
        room: Box<dyn DiskRoom>,
        data: PathBuf,
    ) -> Self {
        Self {
            env,
            drives,
            room,
            data,
        }
    }

    /// The drives there now, in Windows' order; network drives only when `remote`, so a lost share never stalls a
    /// suggestion or a find.
    fn drives(&self, remote: bool) -> Vec<Drive> {
        let listed = (self.drives)().into_iter().filter(|d| remote || !d.remote);
        listed.filter(|d| d.root.is_dir()).collect()
    }

    /// The folder `cloud` names, when its base is known and absolute.
    fn folder(&self, cloud: &Cloud, drives: &[Drive]) -> Option<PathBuf> {
        let labelled = |label: &String| drives.iter().find(|d| d.label.eq_ignore_ascii_case(label));
        let base = match (&cloud.env, &cloud.label) {
            (Some(env), _) => self.env.get(env).cloned(),
            (None, Some(label)) => labelled(label).map(|d| d.root.clone()),
            (None, None) => None,
        };
        let base = base.filter(|b| b.is_absolute())?;
        Some(cloud.parts.iter().fold(base, |path, part| path.join(part)))
    }

    /// The cloud folders there now, in the table's order, each once.
    fn clouds(&self, table: &Table, drives: &[Drive]) -> Vec<Place> {
        let (mut seen, mut found) = (Vec::new(), Vec::new());
        for cloud in &table.cloud {
            let Some(folder) = self.folder(cloud, drives).filter(|f| f.is_dir()) else {
                continue;
            };
            let key = paths::key(&folder);
            if seen.contains(&key) {
                continue;
            }
            seen.push(key);
            let own = folder.file_name().map(|n| n.to_string_lossy().into_owned());
            let name = cloud.name.clone().or(own).unwrap_or_default();
            found.push(Place {
                kind: PlaceKind::Cloud,
                name: Redacted::new(name),
                path: Redacted::new(folder),
            });
        }
        found
    }
}

/// The folder each environment variable `table` names holds, when it holds one.
fn env(table: &Table) -> HashMap<String, PathBuf> {
    let names = table.cloud.iter().filter_map(|c| c.env.clone());
    let held = |name: String| {
        let value = std::env::var_os(&name).filter(|v| !v.is_empty());
        value.map(|v| (name, PathBuf::from(v)))
    };
    names.filter_map(held).collect()
}

/// The length of `bytes` as the port counts it.
fn len(bytes: &[u8]) -> u64 {
    u64::try_from(bytes.len()).unwrap_or(u64::MAX)
}

impl Places for WinPlaces {
    fn suggested(&self) -> Vec<Place> {
        let _quiet = Quiet::new();
        let Ok(table) = table() else {
            return Vec::new();
        };
        let drives = self.drives(false);
        let mut places = self.clouds(table, &drives);
        let removable = drives.iter().filter(|d| d.kind == PlaceKind::Removable);
        places.extend(removable.map(|d| d.place(table)));
        places
    }

    fn folders(&self, folder: Option<&Path>) -> Result<Vec<Place>, PlaceError> {
        let _quiet = Quiet::new();
        let table = table()?;
        match folder {
            None => Ok(self.drives(true).iter().map(|d| d.place(table)).collect()),
            Some(folder) => list::folders(folder, table),
        }
    }

    fn files(&self, folder: &Path, ext: &str) -> Result<Vec<Place>, PlaceError> {
        let _quiet = Quiet::new();
        list::files(folder, ext, table()?)
    }

    fn find(&self) -> Vec<Found> {
        let _quiet = Quiet::new();
        let Ok(table) = table() else {
            return Vec::new();
        };
        let places = self.suggested();
        let mut found: Vec<Found> = places.iter().flat_map(|p| list::boxes(p, table)).collect();
        found.sort_by_key(|f| Reverse(f.modified_ms));
        found
    }

    fn modified(&self, file: &Path) -> Option<i64> {
        let _quiet = Quiet::new();
        file::modified(file)
    }

    fn read(&self, file: &Path, cap: u64) -> Result<Redacted<Vec<u8>>, PlaceError> {
        let _quiet = Quiet::new();
        file::read(file, cap, table()?).map(Redacted::new)
    }

    fn write(&self, folder: &Path, bytes: &[u8]) -> Result<(), PlaceError> {
        let _quiet = Quiet::new();
        let table = table()?;
        if paths::touches(folder, &self.data) {
            return Err(PlaceError::Refused);
        }
        if !folder.is_dir() {
            return Err(PlaceError::Missing);
        }
        if self.room.free(folder).is_ok_and(|free| free < len(bytes)) {
            return Err(PlaceError::Full);
        }
        file::write(folder, bytes, table).map_err(|e| table.error(&e))
    }

    fn state(&self, folder: &Path, need: u64) -> PlaceState {
        let _quiet = Quiet::new();
        if paths::touches(folder, &self.data) {
            return PlaceState::Refused;
        }
        if !folder.is_dir() {
            return PlaceState::Missing;
        }
        match self.room.free(folder) {
            Ok(free) if free < need => PlaceState::Full,
            Ok(_) => PlaceState::Ready,
            Err(_) => PlaceState::Missing,
        }
    }
}
