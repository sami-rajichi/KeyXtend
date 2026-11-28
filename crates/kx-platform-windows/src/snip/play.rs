//! Plays a video of the Snips folder in the user's own player: the running Explorer opens it itself, as the reveals
//! open a folder, so the player never starts as our child or with our uiAccess (ADR-0056 decision 10).

use std::path::Path;
use std::sync::Arc;

use kx_module_api::ports::{MediaOpen, PlayError, SnipFile, SnipFolder};
use kx_platform::snip::playable;

use crate::explorer::{Explorer, bstr};

/// What opens a file in the user's own program: the running Explorer here, a script in tests.
pub(crate) trait Shell {
    /// Has the running file manager open `file` with its default verb, or gives the code that refused.
    fn open(&self, file: &Path) -> Result<(), i32>;
}

/// Plays the Snips folder's videos through the running Explorer; nothing is ever run inside our process.
pub struct WinMediaOpen {
    folder: Arc<dyn SnipFolder>,
}

impl WinMediaOpen {
    /// Plays the videos of `folder` and its unsaved part.
    #[must_use]
    pub fn new(folder: Arc<dyn SnipFolder>) -> Self {
        Self { folder }
    }
}

impl MediaOpen for WinMediaOpen {
    fn play(&self, video: &SnipFile) -> Result<(), PlayError> {
        play(&RunningExplorer, &*self.folder, video)
    }
}

/// Hands `video` to `shell` once `playable` takes it; the shell's refusal keeps its code.
pub(crate) fn play(
    shell: &dyn Shell,
    folder: &dyn SnipFolder,
    video: &SnipFile,
) -> Result<(), PlayError> {
    playable(folder, video)?;
    shell.open(video.expose()).map_err(PlayError::Refused)
}

/// The running Explorer's desktop shell object, found for each play on the caller's thread.
struct RunningExplorer;

impl Shell for RunningExplorer {
    fn open(&self, file: &Path) -> Result<(), i32> {
        Explorer::find()?.open(&bstr(file))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kx_module_api::ports::{FileKind, FolderPath};
    use kx_platform::{SNIP_DIR, UNSAVED_DIR};
    use kx_platform_fake::FakeSnipFolder;
    use kx_test_support::tempdir::TempDir;
    use std::cell::RefCell;
    use std::fs;
    use std::path::PathBuf;

    /// A clock that never moves; the fake folder ages nothing here.
    struct Still;

    impl kx_module_api::Clock for Still {
        fn now(&self) -> kx_module_api::Mono {
            kx_module_api::Mono::default()
        }
    }

    /// A code the scripted shell refuses with.
    const CODE: i32 = -7;

    /// A shell that keeps what it was asked to open, and refuses when told.
    #[derive(Default)]
    struct Script {
        refuses: bool,
        opened: RefCell<Vec<PathBuf>>,
    }

    impl Shell for Script {
        fn open(&self, file: &Path) -> Result<(), i32> {
            self.opened.borrow_mut().push(file.to_path_buf());
            if self.refuses { Err(CODE) } else { Ok(()) }
        }
    }

    /// A temp Snips folder with its unsaved part, and the fake folder naming it.
    fn bed() -> (TempDir, FakeSnipFolder) {
        let dir = TempDir::new("play").unwrap();
        let snips = dir.path().join(SNIP_DIR);
        let part = snips.join(UNSAVED_DIR);
        fs::create_dir_all(&part).unwrap();
        let (snips, part) = (
            FolderPath::new(snips).unwrap(),
            FolderPath::new(part).unwrap(),
        );
        (dir, FakeSnipFolder::new(Arc::new(Still), snips, part))
    }

    /// A video `name` written in the Snips folder of `dir`.
    fn video(dir: &TempDir, name: &str) -> SnipFile {
        let path = dir.path().join(SNIP_DIR).join(name);
        fs::write(&path, b"x").unwrap();
        SnipFile::new(path, FileKind::Mp4).unwrap()
    }

    #[test]
    fn a_video_of_the_snips_folder_goes_to_the_shell_once() {
        let (dir, folder) = bed();
        let (shell, kept) = (Script::default(), video(&dir, "1.mp4"));
        assert_eq!(play(&shell, &folder, &kept), Ok(()));
        assert_eq!(*shell.opened.borrow(), [kept.expose().to_path_buf()]);
    }

    #[test]
    fn a_file_the_rule_refuses_never_reaches_the_shell() {
        let (dir, folder) = bed();
        let shell = Script::default();
        let picture =
            SnipFile::new(dir.path().join(SNIP_DIR).join("1.png"), FileKind::Png).unwrap();
        assert_eq!(play(&shell, &folder, &picture), Err(PlayError::BadPath));
        let gone = SnipFile::new(dir.path().join(SNIP_DIR).join("2.mp4"), FileKind::Mp4).unwrap();
        assert_eq!(play(&shell, &folder, &gone), Err(PlayError::Missing));
        let hidden = FakeSnipFolder::hidden(Arc::new(Still));
        assert_eq!(
            play(&shell, &hidden, &video(&dir, "3.mp4")),
            Err(PlayError::BadPath)
        );
        assert!(shell.opened.borrow().is_empty());
    }

    #[test]
    fn a_refusing_shell_keeps_its_code() {
        let (dir, folder) = bed();
        let shell = Script {
            refuses: true,
            ..Script::default()
        };
        assert_eq!(
            play(&shell, &folder, &video(&dir, "1.mp4")),
            Err(PlayError::Refused(CODE))
        );
    }
}
