//! An in-memory disk for the hidden-run rules: folders, files and links, a redirect that acts as a package's, failing
//! calls, and a count of every call.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::io::{Error, ErrorKind};

use super::*;

/// What sits at a fake path.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    Dir,
    File,
    Link,
}

/// A disk call the fake can fail.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum Op {
    List,
    Write,
    RemoveFile,
    RemoveTree,
}

/// An in-memory disk. A redirect sends writes under its first folder to its second, as a package does, while reads
/// see both; every call is counted.
#[derive(Default)]
pub(super) struct Disk {
    items: RefCell<BTreeMap<PathBuf, Kind>>,
    redirect: Option<(PathBuf, PathBuf)>,
    fails: Vec<Op>,
    calls: Cell<usize>,
}

impl Disk {
    /// A disk with folders at `dirs`, and every package's mirror.
    pub(super) fn with(dirs: &[PathBuf]) -> Self {
        let disk = Self::default();
        for pkg in [PKG, OTHER] {
            disk.mkdirs(&mirror_of(pkg));
        }
        for dir in dirs {
            disk.mkdirs(dir);
        }
        disk
    }

    /// `self`, with writes under the local folder landing in `pkg`'s mirror.
    pub(super) fn hiding_in(mut self, pkg: &str) -> Self {
        self.redirect = Some((at().local, mirror_of(pkg)));
        self
    }

    /// `self`, failing each call of `op`.
    pub(super) fn failing(mut self, op: Op) -> Self {
        self.fails.push(op);
        self
    }

    /// Makes `dir` and every folder above it that is missing; a link on the way stays a link.
    fn mkdirs(&self, dir: &Path) {
        for d in dir.ancestors().filter(|d| !d.as_os_str().is_empty()) {
            let mut items = self.items.borrow_mut();
            items.entry(d.to_path_buf()).or_insert(Kind::Dir);
        }
    }

    /// Whether anything sits below `dir`, as the disk really holds it.
    fn holds_anything(&self, dir: &Path) -> bool {
        let items = self.items.borrow();
        items.keys().any(|p| p != dir && p.starts_with(dir))
    }

    /// Puts `kind` at `path`.
    pub(super) fn put(&self, path: &Path, kind: Kind) {
        self.items.borrow_mut().insert(path.to_path_buf(), kind);
    }

    /// Whether anything is really at `path`.
    pub(super) fn has(&self, path: &Path) -> bool {
        self.items.borrow().contains_key(path)
    }

    /// Every file on the disk.
    pub(super) fn files(&self) -> Vec<PathBuf> {
        let items = self.items.borrow();
        let files = items.iter().filter(|(_, k)| **k == Kind::File);
        files.map(|(p, _)| p.clone()).collect()
    }

    /// Forgets everything at and under `path`.
    pub(super) fn forget(&self, path: &Path) {
        self.items.borrow_mut().retain(|p, _| !p.starts_with(path));
    }

    /// How many disk calls were made.
    pub(super) fn calls(&self) -> usize {
        self.calls.get()
    }

    /// Counts a call, and fails it when told.
    fn call(&self, op: Option<Op>) -> io::Result<()> {
        self.calls.set(self.calls.get() + 1);
        let failed = op.is_some_and(|op| self.fails.contains(&op));
        if failed {
            Err(ErrorKind::Other.into())
        } else {
            Ok(())
        }
    }

    /// Where a write to `path` lands.
    fn real(&self, path: &Path) -> PathBuf {
        match &self.redirect {
            Some((from, to)) if path.starts_with(from) => to.join(path.strip_prefix(from).unwrap()),
            _ => path.to_path_buf(),
        }
    }

    /// What is at `path`, its own or where a write lands, and which of the two holds it.
    fn seen(&self, path: &Path) -> Option<(PathBuf, Kind)> {
        let items = self.items.borrow();
        [path.to_path_buf(), self.real(path)]
            .into_iter()
            .find_map(|p| items.get(&p).map(|k| (p, *k)))
    }
}

impl MirrorDisk for Disk {
    fn folders(&self, dir: &Path) -> io::Result<Vec<String>> {
        self.call(Some(Op::List))?;
        if self.seen(dir).map(|(_, k)| k) != Some(Kind::Dir) {
            return Err(ErrorKind::NotFound.into());
        }
        let items = self.items.borrow();
        let inside = items
            .iter()
            .filter(|(p, k)| p.parent() == Some(dir) && **k == Kind::Dir);
        let names = inside.map(|(p, _)| p.file_name().unwrap().to_string_lossy().into_owned());
        Ok(names.collect())
    }

    fn is_dir(&self, path: &Path) -> bool {
        let _ = self.call(None);
        self.seen(path).is_some_and(|(_, k)| k == Kind::Dir)
    }

    fn exists(&self, path: &Path) -> bool {
        let _ = self.call(None);
        self.seen(path).is_some()
    }

    fn is_empty(&self, dir: &Path) -> bool {
        let _ = self.call(None);
        let seen = self.seen(dir).filter(|(_, k)| *k == Kind::Dir);
        seen.is_some_and(|(at, _)| !self.holds_anything(&at))
    }

    fn make_dir(&self, dir: &Path) -> io::Result<bool> {
        self.call(None)?;
        if self.seen(dir).is_some() {
            return Ok(false);
        }
        self.put(&self.real(dir), Kind::Dir);
        Ok(true)
    }

    fn write_new(&self, path: &Path) -> io::Result<()> {
        self.call(Some(Op::Write))?;
        if self.seen(path).is_some() {
            return Err(ErrorKind::AlreadyExists.into());
        }
        // A redirected write makes its folder in the mirror, as Windows does.
        let real = self.real(path);
        if let Some(parent) = real.parent() {
            self.mkdirs(parent);
        }
        self.put(&real, Kind::File);
        Ok(())
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        self.call(Some(Op::RemoveFile))?;
        let (at, _) = self
            .seen(path)
            .filter(|(_, k)| *k == Kind::File)
            .ok_or(ErrorKind::NotFound)?;
        self.items.borrow_mut().remove(&at);
        Ok(())
    }

    fn remove_empty(&self, dir: &Path) -> io::Result<()> {
        self.call(None)?;
        let (at, _) = self.seen(dir).ok_or(ErrorKind::NotFound)?;
        if self.holds_anything(&at) {
            return Err(Error::other("not empty"));
        }
        self.items.borrow_mut().remove(&at);
        Ok(())
    }

    fn remove_tree(&self, dir: &Path) -> io::Result<()> {
        self.call(Some(Op::RemoveTree))?;
        self.items.borrow_mut().retain(|p, _| !p.starts_with(dir));
        Ok(())
    }
}
