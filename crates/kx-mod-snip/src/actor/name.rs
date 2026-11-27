//! The name the Save dialog proposes (spec "The Save dialog and the folder"): the snip's local date and time, PNG for a
//! picture and MP4 for a video, in the save folder: `save_dir`, else the Snips folder.

use std::path::PathBuf;

use kx_module_api::MS_PER_S;
use kx_module_api::ports::{FileKind, FolderPath, LocalTime, S_PER_MIN, SnipFile};

use super::Actor;

/// What stands between the date and the time.
const DATE_TIME: char = ' ';
/// What stands between the hour, the minute and the second, as a file name takes no colon.
const TIME_SEP: char = '-';
/// The digits of an hour, a minute or a second.
const DIGITS: usize = 2;

/// The name of a snip taken at `at`, without its extension: `2026-11-24 14-05-33`.
pub(super) fn stem(at: &LocalTime) -> String {
    let second = at
        .unix_ms
        .div_euclid(i64::from(MS_PER_S))
        .rem_euclid(S_PER_MIN);
    let (date, hour, minute) = (at.date, at.hour, at.minute);
    format!(
        "{date}{DATE_TIME}{hour:0DIGITS$}{TIME_SEP}{minute:0DIGITS$}{TIME_SEP}{second:0DIGITS$}"
    )
}

impl Actor {
    /// The file the Save dialog proposes for a snip of `kind`, in the folder as last written, so a change on the panel
    /// during the snip counts; `None` in a hidden run, or before the snip's time.
    pub(super) fn proposed(&self, kind: FileKind) -> Option<SnipFile> {
        let folder = self.save_folder()?;
        let path = folder.expose().join(stem(self.stamp.as_ref()?));
        SnipFile::new(path.with_extension(kind.ext()), kind)
    }

    /// The folder the Save dialog opens at: `save_dir` as last written when it is a folder path, else the Snips folder;
    /// `None` in a hidden run, which dims Save.
    pub(super) fn save_folder(&self) -> Option<FolderPath> {
        if self.seen.hidden {
            return None;
        }
        let dir = &self.settings.save_dir;
        let typed = (!dir.is_empty())
            .then(|| FolderPath::new(PathBuf::from(dir)).ok())
            .flatten();
        typed.or_else(|| self.ports.folder.snips())
    }
}
