//! The folder list and the find on the real file system: entries that show, capped, sorted, and never through a link.

use std::ffi::OsStr;
use std::fs::{self, DirEntry, File, Metadata};
use std::io::{self, Read};
use std::os::windows::fs::MetadataExt;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use kx_module_api::Redacted;
use kx_module_api::ports::{
    BOX_MARK, Found, Place, PlaceError, PlaceKind, is_box_head, is_box_name,
};

use super::quiet;
use super::table::{Table, only};

/// The errors a list returns besides `Io`.
const LIST: [PlaceError; 1] = [PlaceError::Missing];
/// The first bytes a find reads to check the mark.
const HEAD: u64 = BOX_MARK.len() as u64;

/// The folders right in `folder` that show, A to Z ignoring case.
pub(super) fn folders(folder: &Path, table: &Table) -> Result<Vec<Place>, PlaceError> {
    list(folder, table, PlaceKind::Folder, |facts, _| facts.is_dir())
}

/// The files right in `folder` whose extension is `ext` in any case, A to Z ignoring case.
pub(super) fn files(folder: &Path, ext: &str, table: &Table) -> Result<Vec<Place>, PlaceError> {
    let fits = |name: &OsStr| {
        Path::new(name)
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case(ext))
    };
    list(folder, table, PlaceKind::File, |facts, name| {
        facts.is_file() && fits(name)
    })
}

/// The boxes in `place`'s top level and in its subfolders, as many subfolders as the table allows a find.
pub(super) fn boxes(place: &Place, table: &Table) -> Vec<Found> {
    let mut found = Vec::new();
    let Ok(top) = shown(place.path.expose(), table) else {
        return found;
    };
    let mut subs = Vec::new();
    for (entry, facts) in top {
        if !facts.is_dir() {
            found.extend(boxed(place, &entry, &facts));
        } else if subs.len() < table.max_subfolders {
            subs.push(entry.path());
        }
    }
    for sub in subs {
        let Ok(inner) = shown(&sub, table) else {
            continue;
        };
        found.extend(inner.filter_map(|(entry, facts)| boxed(place, &entry, &facts)));
    }
    found
}

/// The entries right in `folder` that show, with the facts the listing itself gives: a link or junction is neither a
/// folder nor a file there, so nothing ever goes through one.
fn shown(folder: &Path, table: &Table) -> io::Result<impl Iterator<Item = (DirEntry, Metadata)>> {
    quiet::touch();
    let entries = fs::read_dir(folder)?.filter_map(Result::ok);
    let facts = entries.filter_map(|e| e.metadata().ok().map(|m| (e, m)));
    Ok(facts.filter(|(_, m)| !table.hides(m.file_attributes())))
}

/// The entries in `folder` that show and `keep` takes, as `kind`: as many as the table's limit allows, in the system's
/// order, then A to Z ignoring case.
fn list(
    folder: &Path,
    table: &Table,
    kind: PlaceKind,
    keep: impl Fn(&Metadata, &OsStr) -> bool,
) -> Result<Vec<Place>, PlaceError> {
    let entries = shown(folder, table).map_err(|e| only(table.error(&e), &LIST))?;
    let kept = entries.filter(|(entry, facts)| keep(facts, &entry.file_name()));
    let mut list: Vec<Place> = kept
        .take(table.max_listed)
        .map(|(entry, _)| item(kind, &entry))
        .collect();
    list.sort_by_cached_key(|p| p.name.expose().to_lowercase());
    Ok(list)
}

/// `entry` as a place of `kind`, named by its file name.
fn item(kind: PlaceKind, entry: &DirEntry) -> Place {
    let name = entry.file_name().to_string_lossy().into_owned();
    Place {
        kind,
        name: Redacted::new(name),
        path: Redacted::new(entry.path()),
    }
}

/// The box `entry` is, found in `place`: a file with a box's name and the mark in its first bytes.
fn boxed(place: &Place, entry: &DirEntry, facts: &Metadata) -> Option<Found> {
    let named = entry.file_name().to_str().is_some_and(is_box_name);
    if !facts.is_file() || !named || !marked(&entry.path()) {
        return None;
    }
    Some(Found {
        place: place.clone(),
        path: Redacted::new(entry.path()),
        modified_ms: ms(facts.modified().ok()?),
    })
}

/// Whether `file`'s first bytes are the box's mark.
fn marked(file: &Path) -> bool {
    let mut head = Vec::new();
    let read = File::open(file).and_then(|f| f.take(HEAD).read_to_end(&mut head));
    read.is_ok() && is_box_head(&head)
}

/// `time` in ms since the Unix epoch, negative before it.
pub(super) fn ms(time: SystemTime) -> i64 {
    match time.duration_since(UNIX_EPOCH) {
        Ok(after) => i64::try_from(after.as_millis()).unwrap_or(i64::MAX),
        Err(before) => i64::try_from(before.duration().as_millis()).map_or(i64::MIN, |b| -b),
    }
}
