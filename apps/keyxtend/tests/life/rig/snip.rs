//! Snip's fakes for the app: one small frozen screen, and a Snips folder in memory at its place under the data folder,
//! or none, each with the recorder, the dialogs and the player over it. The app's own `Ports::of` picks one for the
//! run, one seal backs the recorders, and a fake Esc watch stands for the hand's Esc.

use keyxtend::hidden::Run;
use keyxtend::snip::{Guarded, Kit as Make, Ports};
use kx_module_api::Clock;
use kx_module_api::ports::{
    BASE_DPI, FileAsk, FolderPath, MediaOpen, MicSeal, Point, Recorder, Rect, ScreenFreeze,
    SnipFolder,
};
use kx_platform::{Platform, snip_dirs};
use kx_platform_fake::{
    FakeEscWatch, FakeFileAsk, FakeMediaOpen, FakePlatform, FakeRecorder, FakeScreenFreeze,
    FakeSnipFolder,
};
use std::sync::Arc;

/// The one screen, small so each freeze paints quickly.
pub const SCREEN: Rect = Rect {
    x: 0,
    y: 0,
    w: 320,
    h: 180,
};
/// Where the pointer is, on that screen.
const POINTER: Point = Point { x: 10, y: 10 };

/// A Snips folder and the fakes that work in it.
pub struct Kit {
    /// The folder's files, in memory.
    pub folder: FakeSnipFolder,
    /// Records into its unsaved part.
    pub recorder: FakeRecorder,
    /// The Save and folder dialogs, which ask nothing until queued.
    pub dialogs: FakeFileAsk,
    /// Plays its videos.
    pub player: FakeMediaOpen,
}

impl Kit {
    /// The fakes over `folder`, the recorder holding `seal`.
    fn new(folder: FakeSnipFolder, seal: &MicSeal) -> Self {
        let recorder = FakeRecorder::new(seal.clone(), &folder);
        let (dialogs, player) = (FakeFileAsk::new(&folder), FakeMediaOpen::new(&folder));
        Self {
            folder,
            recorder,
            dialogs,
            player,
        }
    }
}

/// Snip's fakes for either run.
pub struct Snips {
    /// The frozen desktop.
    pub freeze: FakeScreenFreeze,
    /// The seal the recorders hold, as the app's one seal.
    pub seal: MicSeal,
    /// A normal run's folder and fakes.
    pub normal: Kit,
    /// A hidden run's: no folder.
    pub hidden: Kit,
    /// Behind the hand's Esc, which a test presses.
    pub esc: Arc<FakeEscWatch>,
}

impl Snips {
    /// The fakes over `platform`'s data folder, aging files by its clock.
    pub fn new(platform: &FakePlatform) -> Self {
        let clock: Arc<dyn Clock> = Arc::new(platform.clock_handle());
        let (snips, unsaved) = snip_dirs(platform.dirs().data_dir().path());
        let (snips, unsaved) = (FolderPath::new(snips), FolderPath::new(unsaved));
        let folder = FakeSnipFolder::new(Arc::clone(&clock), snips.unwrap(), unsaved.unwrap());
        let seal = MicSeal::new();
        Self {
            freeze: FakeScreenFreeze::new(vec![(SCREEN, BASE_DPI)], POINTER),
            normal: Kit::new(folder, &seal),
            hidden: Kit::new(FakeSnipFolder::hidden(clock), &seal),
            seal,
            esc: Arc::new(FakeEscWatch::new()),
        }
    }

    /// The ports the app makes for `run` from these fakes.
    pub fn ports(&self, run: &Run) -> Ports {
        Ports::of(self, run, self.seal.clone())
    }

    /// The kit whose folder is `folder`: the hidden one has none.
    fn over(&self, folder: &Arc<dyn SnipFolder>) -> &Kit {
        if folder.snips().is_none() {
            &self.hidden
        } else {
            &self.normal
        }
    }
}

impl Make for Snips {
    fn freeze(&self) -> Arc<dyn ScreenFreeze> {
        Arc::new(self.freeze.clone())
    }

    fn folder(&self, hidden: bool) -> Arc<dyn SnipFolder> {
        let kit = if hidden { &self.hidden } else { &self.normal };
        Arc::new(kit.folder.clone())
    }

    /// The kit's recorder already holds `seal`, which `ports` passes.
    fn recorder(&self, _seal: MicSeal, folder: Arc<dyn SnipFolder>) -> Arc<dyn Recorder> {
        Arc::new(self.over(&folder).recorder.clone())
    }

    fn dialogs(&self, folder: Arc<dyn SnipFolder>) -> Arc<dyn FileAsk> {
        Arc::new(self.over(&folder).dialogs.clone())
    }

    fn player(&self, folder: Arc<dyn SnipFolder>) -> Arc<dyn MediaOpen> {
        Arc::new(self.over(&folder).player.clone())
    }

    /// A plain folder: the rights are the platform's, which the app's own tests check.
    fn guarded(&self) -> Guarded {
        |folder| std::fs::create_dir(folder)
    }
}
