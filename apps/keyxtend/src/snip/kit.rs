//! What makes Snip's ports: the Windows adapter in the app, the fakes in its tests. `Ports::of` picks the folder for
//! the run, so the app and its tests share that choice.

use super::Guarded;
use kx_module_api::ports::{FileAsk, MediaOpen, MicSeal, Recorder, ScreenFreeze, SnipFolder};
use kx_platform_windows::WindowsPlatform;
use std::io;
use std::path::Path;
use std::sync::Arc;

/// The platform's makers of Snip's ports; each port but the freeze works over the folder it is given.
pub trait Kit {
    /// Freezes every screen.
    fn freeze(&self) -> Arc<dyn ScreenFreeze>;
    /// The Snips folder, or none when `hidden`.
    fn folder(&self, hidden: bool) -> Arc<dyn SnipFolder>;
    /// Records into `folder`'s unsaved part, opening only `seal`'s passes.
    fn recorder(&self, seal: MicSeal, folder: Arc<dyn SnipFolder>) -> Arc<dyn Recorder>;
    /// The system's Save and folder dialogs, copying videos from `folder`.
    fn dialogs(&self, folder: Arc<dyn SnipFolder>) -> Arc<dyn FileAsk>;
    /// Plays `folder`'s videos.
    fn player(&self, folder: Arc<dyn SnipFolder>) -> Arc<dyn MediaOpen>;
    /// Makes the Snips folder with rights the user and the system alone hold, as the models folder is made.
    fn guarded(&self) -> Guarded;
}

impl Kit for WindowsPlatform {
    fn freeze(&self) -> Arc<dyn ScreenFreeze> {
        self.screen_freeze()
    }

    fn folder(&self, hidden: bool) -> Arc<dyn SnipFolder> {
        self.snip_folder(hidden)
    }

    fn recorder(&self, seal: MicSeal, folder: Arc<dyn SnipFolder>) -> Arc<dyn Recorder> {
        WindowsPlatform::recorder(self, seal, folder)
    }

    fn dialogs(&self, folder: Arc<dyn SnipFolder>) -> Arc<dyn FileAsk> {
        self.file_ask(folder)
    }

    fn player(&self, folder: Arc<dyn SnipFolder>) -> Arc<dyn MediaOpen> {
        self.media_open(folder)
    }

    fn guarded(&self) -> Guarded {
        guarded
    }
}

/// Makes `folder`, whose parent exists, with the models folder's maker: protected, the user and the system alone.
#[cfg(windows)]
pub(super) fn guarded(folder: &Path) -> io::Result<()> {
    use kx_platform::models::Disk;
    kx_platform_windows::WinDisk.make_guarded(folder)
}

/// Makes `folder`, whose parent exists; other systems have no Windows rights to set.
#[cfg(not(windows))]
pub(super) fn guarded(folder: &Path) -> io::Result<()> {
    std::fs::create_dir(folder)
}
