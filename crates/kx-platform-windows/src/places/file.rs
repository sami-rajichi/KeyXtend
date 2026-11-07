//! A box's bytes on the real file system: read within a cap, and written through a temp file of its own that replaces
//! the box in one step, sweeping the old temp files of another process's crash.

use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use kx_module_api::ports::{PlaceError, box_file};

use super::table::{Table, only};
use super::{list, quiet};

/// The errors a read returns besides `TooLarge` and `Io`.
const READ: [PlaceError; 1] = [PlaceError::Missing];
/// Counts this process's writes, so each fills a temp file of its own.
static WRITES: AtomicU64 = AtomicU64::new(0);

/// The bytes of `file`, refused over `cap` before it is opened.
pub(super) fn read(file: &Path, cap: u64, table: &Table) -> Result<Vec<u8>, PlaceError> {
    quiet::touch();
    let fail = |e: io::Error| only(table.error(&e), &READ);
    // The size is asked without read access, so a file over the cap is never opened for reading.
    let size = fs::metadata(file).map_err(fail)?.len();
    if size > cap {
        return Err(PlaceError::TooLarge);
    }
    let mut bytes = Vec::with_capacity(usize::try_from(size).unwrap_or(0));
    let opened = File::open(file).map_err(fail)?;
    // A file that grew since is read one byte past the cap at most, and refused.
    let read = opened.take(cap.saturating_add(1)).read_to_end(&mut bytes);
    read.map_err(fail)?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > cap {
        return Err(PlaceError::TooLarge);
    }
    Ok(bytes)
}

/// When `file` was last written, in ms since the Unix epoch; `None` when it is gone or Windows does not tell.
pub(super) fn modified(file: &Path) -> Option<i64> {
    quiet::touch();
    Some(list::ms(fs::metadata(file).ok()?.modified().ok()?))
}

/// Writes `bytes` as the box in `folder`: to this write's own temp file, flushed, then renamed over the box in one
/// step, so a writer only ever puts its own whole file in place.
pub(super) fn write(folder: &Path, bytes: &[u8], table: &Table) -> io::Result<()> {
    quiet::touch();
    sweep(folder, table);
    let n = WRITES.fetch_add(1, Ordering::Relaxed);
    let temp = folder.join(table.temp_name(process::id(), n));
    // No live process has this name, so a file under it is a crashed run's; it goes first, as a link when it is one.
    let _ = fs::remove_file(&temp);
    let wrote = fill(&temp, bytes).and_then(|()| fs::rename(&temp, folder.join(box_file())));
    if wrote.is_err() {
        // The old box was never touched; only the temp file goes.
        let _ = fs::remove_file(&temp);
    }
    wrote
}

/// Writes `bytes` to the new file `temp` and flushes them to the disk.
fn fill(temp: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(temp)?;
    file.write_all(bytes)?;
    file.sync_all()
}

/// Deletes the temp files a crash left in `folder`: another process's, older than the table allows, never this one's.
fn sweep(folder: &Path, table: &Table) {
    let Ok(entries) = fs::read_dir(folder) else {
        return;
    };
    let (me, limit) = (process::id(), Duration::from_secs(table.temp_stale_s));
    for entry in entries.filter_map(Result::ok) {
        let owner = entry.file_name().to_str().and_then(|n| table.temp_owner(n));
        if owner.is_none_or(|pid| pid == me) {
            continue;
        }
        let facts = entry.metadata().ok().filter(Metadata::is_file);
        let age = facts.and_then(|m| m.modified().ok()?.elapsed().ok());
        if age.is_some_and(|age| age > limit) {
            let _ = fs::remove_file(entry.path());
        }
    }
}
