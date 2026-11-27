//! The Snip ports `WindowsPlatform` hands the app to register (ADR-0056): the freeze, the Snips folder, and the
//! recorder, the dialogs and the player over that folder.

use std::sync::Arc;

use kx_module_api::ports::{
    FileAsk, FolderPath, MediaOpen, MicSeal, Recorder, ScreenFreeze, SnipFolder,
};
use kx_platform::snip::{DiskSnips, WorkerFileAsk, WorkerRecorder};
use kx_platform::snip_dirs;

use super::WindowsPlatform;
use crate::{WinDisk, WinKit, WinMediaOpen, WinScreenFreeze, WinSpawner};

impl WindowsPlatform {
    /// The screen freeze, for the app to register under `ScreenFreezeKey`.
    #[must_use]
    pub fn screen_freeze(&self) -> Arc<dyn ScreenFreeze> {
        Arc::new(WinScreenFreeze::new())
    }

    /// The Snips folder in the data folder, none in a `hidden` run, for the app to register under `SnipFolderKey`; the
    /// app makes it at boot.
    #[must_use]
    pub fn snip_folder(&self, hidden: bool) -> Arc<dyn SnipFolder> {
        let (snips, _) = snip_dirs(self.dirs.data_dir().path());
        let snips = (!hidden).then(|| FolderPath::new(snips).ok()).flatten();
        if !hidden && snips.is_none() {
            tracing::warn!("the Snips folder's path was refused, so no snip is kept");
        }
        Arc::new(DiskSnips::new(snips, Arc::new(WinDisk)))
    }

    /// The recorder in the record job of the worker beside the program, into `folder`'s unsaved part and opening only
    /// `seal`'s passes, for the app to register under `RecorderKey`.
    #[must_use]
    pub fn recorder(&self, seal: MicSeal, folder: Arc<dyn SnipFolder>) -> Arc<dyn Recorder> {
        let worker = WinSpawner::beside(&self.dirs.exe_dir);
        let (clock, kit) = (Arc::new(self.clock), Arc::new(WinKit));
        Arc::new(WorkerRecorder::new(worker, clock, seal, folder, kit))
    }

    /// The system's dialogs in the pick job of the worker beside the program, copying videos from `folder`'s unsaved
    /// part, for the app to register under `FileAskKey`.
    #[must_use]
    pub fn file_ask(&self, folder: Arc<dyn SnipFolder>) -> Arc<dyn FileAsk> {
        let worker = WinSpawner::beside(&self.dirs.exe_dir);
        Arc::new(WorkerFileAsk::new(worker, Arc::new(self.clock), folder))
    }

    /// The player of `folder`'s videos through the running Explorer, for the app to register under `MediaOpenKey`.
    #[must_use]
    pub fn media_open(&self, folder: Arc<dyn SnipFolder>) -> Arc<dyn MediaOpen> {
        Arc::new(WinMediaOpen::new(folder))
    }
}
