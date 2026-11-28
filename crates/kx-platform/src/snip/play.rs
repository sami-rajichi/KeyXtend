//! The rule `MediaOpen` plays by on every system: only an MP4 directly in the Snips folder or its unsaved part, and
//! only a plain file there, never a link or a folder.

use super::unsaved_part;
use kx_module_api::ports::{FileKind, PlayError, SnipFile, SnipFolder};
use std::fs;

/// Refuses `video` unless the player may open it.
///
/// # Errors
/// `BadPath` for a picture, a file elsewhere or any file in a hidden run, `Missing` unless a plain file is there.
pub fn playable(folder: &dyn SnipFolder, video: &SnipFile) -> Result<(), PlayError> {
    let in_snips = folder.snips().is_some_and(|s| video.lies_in(&s));
    let in_part = unsaved_part(folder).is_some_and(|u| video.lies_in(&u));
    if video.kind() != FileKind::Mp4 || !(in_snips || in_part) {
        return Err(PlayError::BadPath);
    }
    let plain = fs::symlink_metadata(video.expose()).is_ok_and(|m| m.is_file());
    plain.then_some(()).ok_or(PlayError::Missing)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SNIP_DIR, UNSAVED_DIR};
    use kx_module_api::ports::FolderPath;
    use kx_platform_fake::FakeSnipFolder;
    use kx_test_support::tempdir::TempDir;
    use std::path::Path;
    use std::sync::Arc;

    /// The fake folder naming `dir`'s Snips folder, or a hidden run's.
    fn folder(dir: Option<&Path>) -> FakeSnipFolder {
        let clock = Arc::new(crate::worker::fake::Hands::default());
        let Some(dir) = dir else {
            return FakeSnipFolder::hidden(clock);
        };
        let snips = dir.join(SNIP_DIR);
        let part = FolderPath::new(snips.join(UNSAVED_DIR)).unwrap();
        FakeSnipFolder::new(clock, FolderPath::new(snips).unwrap(), part)
    }

    /// `name` of `kind` under `dir`, written as a plain file unless `write` is false.
    fn file(dir: &Path, name: &str, kind: FileKind, write: bool) -> SnipFile {
        let path = dir.join(name);
        if write {
            fs::write(&path, b"x").unwrap();
        }
        SnipFile::new(path, kind).unwrap()
    }

    #[test]
    fn a_video_of_the_snips_folder_or_its_unsaved_part_plays_and_nothing_else() {
        let dir = TempDir::new("play").unwrap();
        let (snips, part) = (
            dir.path().join(SNIP_DIR),
            dir.path().join(SNIP_DIR).join(UNSAVED_DIR),
        );
        fs::create_dir_all(&part).unwrap();
        let held = folder(Some(dir.path()));
        let mp4 = FileKind::Mp4;
        assert_eq!(playable(&held, &file(&snips, "1.mp4", mp4, true)), Ok(()));
        assert_eq!(playable(&held, &file(&part, "2.mp4", mp4, true)), Ok(()));
        let picture = file(&snips, "3.png", FileKind::Png, true);
        assert_eq!(playable(&held, &picture), Err(PlayError::BadPath));
        let elsewhere = file(dir.path(), "4.mp4", mp4, true);
        assert_eq!(playable(&held, &elsewhere), Err(PlayError::BadPath));
        let gone = file(&snips, "5.mp4", mp4, false);
        assert_eq!(playable(&held, &gone), Err(PlayError::Missing));
        fs::create_dir(snips.join("6.mp4")).unwrap();
        let odd = file(&snips, "6.mp4", mp4, false);
        assert_eq!(playable(&held, &odd), Err(PlayError::Missing), "a folder");
        let one = file(&snips, "1.mp4", mp4, false);
        assert_eq!(playable(&folder(None), &one), Err(PlayError::BadPath));
    }
}
