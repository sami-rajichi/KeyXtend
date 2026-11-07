//! The OS-neutral rules of the hidden-run probe (ADR-0037): which package mirrors to look in, the probe's steps and its
//! clean-up, the copies list and the checked delete. An adapter gives the folders and the disk calls (`MirrorDisk`).

use std::io::{self, ErrorKind};
use std::path::{Component, Path, PathBuf};

use crate::APP_DIR_NAME;
use crate::ui::{HiddenCopy, HiddenError, Probe};

/// The disk calls behind the probe and the copies; an adapter's real disk, or a fake in tests.
pub trait MirrorDisk {
    /// The names of the folders right inside `dir`, links left out.
    ///
    /// # Errors
    /// `NotFound` when `dir` is missing, any other kind when it cannot be read.
    fn folders(&self, dir: &Path) -> io::Result<Vec<String>>;

    /// Whether `path` is a folder itself, not a link to one.
    fn is_dir(&self, path: &Path) -> bool;

    /// Whether anything is at `path`.
    fn exists(&self, path: &Path) -> bool;

    /// Whether the folder `dir` holds nothing; false when it is missing or cannot be read.
    fn is_empty(&self, dir: &Path) -> bool;

    /// Makes the folder `dir`, whose parent must exist; true when it was missing.
    ///
    /// # Errors
    /// When it is missing and cannot be made.
    fn make_dir(&self, dir: &Path) -> io::Result<bool>;

    /// Writes a new empty file at `path`.
    ///
    /// # Errors
    /// When anything is there already or the write fails.
    fn write_new(&self, path: &Path) -> io::Result<()>;

    /// Deletes the file at `path`.
    ///
    /// # Errors
    /// When it is not deleted.
    fn remove_file(&self, path: &Path) -> io::Result<()>;

    /// Deletes the folder `dir` when it is empty.
    ///
    /// # Errors
    /// When it holds anything or is not deleted.
    fn remove_empty(&self, dir: &Path) -> io::Result<()>;

    /// Deletes the folder `dir` and everything in it at once; a link inside goes as a link, never followed.
    ///
    /// # Errors
    /// When any of it is not deleted.
    fn remove_tree(&self, dir: &Path) -> io::Result<()>;
}

/// Where packages keep their private copies of the local app-data folder, from the adapter's table.
/// It has no `Debug`, as its folders name the user.
#[derive(Clone)]
pub struct Mirrors {
    /// The user's local app-data folder.
    pub local: PathBuf,
    /// The folder that holds one folder per package.
    pub root: PathBuf,
    /// The private copy of `local` inside each package folder, relative to it.
    pub mirror: PathBuf,
    /// Ends the app's name in a package folder's name, as `_` does in `Claude_pzs8sxrjxfjjc`.
    pub app_end: String,
}

impl Mirrors {
    /// `package`'s private copy of the local app-data folder.
    fn mirror_in(&self, package: &str) -> PathBuf {
        self.root.join(package).join(&self.mirror)
    }

    /// The copy of our data folder in `package`'s mirror, named for the UI.
    fn copy(&self, package: String) -> HiddenCopy {
        let path = self.mirror_in(&package).join(APP_DIR_NAME);
        let app = self.app_of(&package);
        HiddenCopy::new(path, package, app)
    }

    /// The app's short name in `package`'s name, before `app_end`; the whole name when none comes first.
    fn app_of(&self, package: &str) -> String {
        let app = package
            .split_once(self.app_end.as_str())
            .map(|(app, _)| app);
        app.filter(|a| !a.is_empty()).unwrap_or(package).to_owned()
    }
}

/// Whether `dir` lies below the local app-data folder `local`, the only place the probe ever looks at.
#[must_use]
pub fn under(dir: &Path, local: &Path) -> bool {
    place(dir, local).is_some()
}

/// Whether what a run saves in `data_dir` is hidden: a probe file named `name` written there and looked for in each
/// package's mirror reached with no link, then deleted. A folder not `under` `local` is normal without any disk call;
/// an empty data folder the probe made or found in the hidden copy goes again, and a hidden run stays hidden whatever
/// its clean-up gives.
///
/// # Errors
/// `Unreadable` when the folders cannot be read or the probe not written or deleted.
pub fn probe<D: MirrorDisk>(
    disk: &D,
    at: &Mirrors,
    data_dir: &Path,
    name: &str,
) -> Result<Probe, HiddenError> {
    let Some(place) = place(data_dir, &at.local) else {
        return Ok(Probe::Normal);
    };
    let made = disk
        .make_dir(data_dir)
        .map_err(|_| HiddenError::Unreadable)?;
    let file = data_dir.join(name);
    let wrote = disk.write_new(&file).is_ok();
    let found = if wrote {
        scan(disk, at, &place, name)
    } else {
        Err(HiddenError::Unreadable)
    };
    // Only a probe this call wrote is deleted, never a file that already had its name.
    let deleted = !wrote || disk.remove_file(&file).is_ok();
    // Folders left empty go: ours, and the one Windows made in the hidden copy; one filled meanwhile stays.
    if made {
        let _ = disk.remove_empty(data_dir);
    }
    if let Ok(Some(package)) = &found {
        let _ = disk.remove_empty(&at.mirror_in(package).join(&place));
    }
    match found {
        Ok(Some(package)) => Ok(Probe::Hidden {
            app: at.app_of(&package),
            package,
        }),
        Ok(None) if deleted => Ok(Probe::Normal),
        _ => Err(HiddenError::Unreadable),
    }
}

/// Every package whose mirror holds our app folder with anything in it, reached with no link on the way, A to Z.
///
/// # Errors
/// `Unreadable` when the packages cannot be listed.
pub fn copies<D: MirrorDisk>(disk: &D, at: &Mirrors) -> Result<Vec<HiddenCopy>, HiddenError> {
    let steps = at.mirror.join(APP_DIR_NAME);
    let held = packages(disk, &at.root)?.into_iter().filter(|p| {
        let package = at.root.join(p);
        no_link(disk, &package, &steps) && !disk.is_empty(&package.join(&steps))
    });
    Ok(held.map(|p| at.copy(p)).collect())
}

/// Deletes `copy` once `copies` lists it again now.
///
/// # Errors
/// `Refused` when it is not listed, `Unreadable` when the list fails, `Undeleted` when the delete does.
pub fn remove<D: MirrorDisk>(disk: &D, at: &Mirrors, copy: &HiddenCopy) -> Result<(), HiddenError> {
    if !copies(disk, at)?.contains(copy) {
        return Err(HiddenError::Refused);
    }
    disk.remove_tree(copy.path())
        .map_err(|_| HiddenError::Undeleted)
}

/// `dir`'s place under `base`, compared step by step without ASCII case so another spelling still counts; `None` for
/// `base` itself, outside it, or when a step below it is no plain folder name, such as `..`.
fn place(dir: &Path, base: &Path) -> Option<PathBuf> {
    let mut parts = dir.components();
    for want in base.components() {
        let got = parts.next()?;
        if !got.as_os_str().eq_ignore_ascii_case(want.as_os_str()) {
            return None;
        }
    }
    let rest = parts.as_path();
    let plain = rest.components().all(|c| matches!(c, Component::Normal(_)));
    (plain && !rest.as_os_str().is_empty()).then(|| rest.to_path_buf())
}

/// The package folders in `root`, A to Z; none when `root` is missing.
fn packages<D: MirrorDisk>(disk: &D, root: &Path) -> Result<Vec<String>, HiddenError> {
    match disk.folders(root) {
        Ok(mut names) => {
            names.sort();
            Ok(names)
        }
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(Vec::new()),
        Err(_) => Err(HiddenError::Unreadable),
    }
}

/// The first package, A to Z, whose mirror holds the probe `name` at `place`, reached with no link on the way, so a
/// junction back to the real folder never passes for a hidden copy.
fn scan<D: MirrorDisk>(
    disk: &D,
    at: &Mirrors,
    place: &Path,
    name: &str,
) -> Result<Option<String>, HiddenError> {
    let steps = at.mirror.join(place);
    let mut all = packages(disk, &at.root)?.into_iter();
    Ok(all.find(|p| {
        let package = at.root.join(p);
        no_link(disk, &package, &steps) && disk.exists(&package.join(&steps).join(name))
    }))
}

/// Whether each step of `steps` below `base` is a folder itself, never a link.
fn no_link<D: MirrorDisk>(disk: &D, base: &Path, steps: &Path) -> bool {
    let mut path = base.to_path_buf();
    steps.components().all(|step| {
        path.push(step);
        disk.is_dir(&path)
    })
}

#[cfg(test)]
mod tests;
