//! Snip's place and ports (ADR-0056): the Snips folder under the data folder, which the boot makes guarded unless the
//! run is hidden (ADR-0037), and the six ports the platform hands, the recorder behind the app's one seal.

mod kit;
mod sealed;

use crate::hidden::Run;
use kx_kernel::{Kernel, KernelError};
use kx_module_api::ports::{
    FileAsk, FileAskKey, MediaOpen, MediaOpenKey, MicPassKey, MicPasses, MicSeal, Recorder,
    RecorderKey, ScreenFreeze, ScreenFreezeKey, SnipFolder, SnipFolderKey,
};
use kx_platform::{AppDirs, snip_dirs};
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub use kit::Kit;
pub use kx_platform::{SNIP_DIR, UNSAVED_DIR};
pub use sealed::Sealed;

/// Makes a folder, whose parent exists, with rights the user and the system alone hold; `AlreadyExists` when it is
/// there.
pub type Guarded = fn(&Path) -> io::Result<()>;

/// The Snips folder's unsaved part for the app's `dirs`.
#[must_use]
pub fn unsaved(dirs: &AppDirs) -> PathBuf {
    snip_dirs(dirs.data_dir().path()).1
}

/// Makes the Snips folder under `dirs`' data folder by `guarded`, and its unsaved part inside, which takes its rights;
/// a hidden run makes none (ADR-0037), and a folder already there keeps its rights. The error's kind when one failed.
#[must_use]
pub fn make(run: &Run, dirs: &AppDirs, guarded: Guarded) -> Option<ErrorKind> {
    if run.is_hidden() {
        return None;
    }
    let (snips, unsaved) = snip_dirs(dirs.data_dir().path());
    let there = |dir: &Path, make: Guarded| {
        if dir.is_dir() {
            return Ok(());
        }
        match make(dir) {
            Err(e) if e.kind() == ErrorKind::AlreadyExists && dir.is_dir() => Ok(()),
            made => made,
        }
    };
    let parent = snips.parent().map_or(Ok(()), fs::create_dir_all);
    let made = parent
        .and_then(|()| there(&snips, guarded))
        .and_then(|()| there(&unsaved, |dir| fs::create_dir(dir)));
    made.err().map(|e| e.kind())
}

/// The Snip ports the platform hands the app; `recorder` holds a clone of `seal`, the app's only microphone seal.
pub struct Ports {
    /// Freezes every screen.
    pub freeze: Arc<dyn ScreenFreeze>,
    /// The Snips folder, none in a hidden run.
    pub folder: Arc<dyn SnipFolder>,
    /// Records into the folder's unsaved part.
    pub recorder: Arc<dyn Recorder>,
    /// The app's microphone seal, which alone makes passes the recorder opens.
    pub seal: MicSeal,
    /// The system's Save and folder dialogs.
    pub dialogs: Arc<dyn FileAsk>,
    /// Plays a video of the folder.
    pub player: Arc<dyn MediaOpen>,
    /// Makes the Snips folder at boot, guarded as the models folder is.
    pub guarded: Guarded,
}

impl Ports {
    /// The ports `kit` makes for `run`: no Snips folder when hidden, so Snip offers no Video and no Save (ADR-0037),
    /// and each other port over the one folder, the recorder holding a clone of `seal`, the app's one seal.
    pub fn of(kit: &dyn Kit, run: &Run, seal: MicSeal) -> Self {
        let folder = kit.folder(run.is_hidden());
        Self {
            freeze: kit.freeze(),
            recorder: kit.recorder(seal.clone(), Arc::clone(&folder)),
            dialogs: kit.dialogs(Arc::clone(&folder)),
            player: kit.player(Arc::clone(&folder)),
            guarded: kit.guarded(),
            folder,
            seal,
        }
    }

    /// Registers each port with `kernel`: the recorder behind one check of `seal`, and `seal` as the passes.
    ///
    /// # Errors
    /// When the kernel refuses a registration.
    pub fn provide(self, kernel: &mut Kernel) -> Result<(), KernelError> {
        let recorder = Sealed::new(self.seal.clone(), self.recorder);
        let passes: Arc<dyn MicPasses> = Arc::new(self.seal);
        kernel.provide_platform::<ScreenFreezeKey>(self.freeze)?;
        kernel.provide_platform::<SnipFolderKey>(self.folder)?;
        kernel.provide_platform::<RecorderKey>(Arc::new(recorder))?;
        kernel.provide_platform::<MicPassKey>(passes)?;
        kernel.provide_platform::<FileAskKey>(self.dialogs)?;
        kernel.provide_platform::<MediaOpenKey>(self.player)
    }
}

#[cfg(test)]
mod tests;
