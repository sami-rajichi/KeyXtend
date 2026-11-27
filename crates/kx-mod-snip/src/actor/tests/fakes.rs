//! The fakes behind the actor's ports, sharing one Snips folder and clock: a screen of 64 by 48 pixels, a picture's
//! PNG the media worker gives, and a seal both the recorder and the passes hold.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::Sender;

use kx_module_api::ports::{
    Date, FileKind, FolderPath, MicSeal, PictureError, Point, Rect, SnipFile,
};
use kx_module_api::{Bus, Mono, SettingsHandle};
use kx_platform_fake::{
    FakeCalendar, FakeClipboardWriter, FakeClock, FakeFileAsk, FakeFolderReveal, FakeMediaOpen,
    FakePictures, FakeRecorder, FakeScreenFreeze, FakeSnipFolder,
};

use super::super::{Leftovers, Ports};
use super::rig::in_place;

/// The Snips folder; forward slashes read as a drive path on every system.
pub(super) const SNIPS: &str = "C:/Users/Alex/KeyXtend/snips";
/// Its unsaved part.
pub(super) const UNSAVED: &str = "C:/Users/Alex/KeyXtend/snips/unsaved";
/// A place the user chose.
pub(super) const CHOSEN: &str = "C:/Users/Alex/Pictures";
/// The one screen.
pub(super) const SCREEN: Rect = Rect {
    x: 0,
    y: 0,
    w: 64,
    h: 48,
};
/// A region's first corner.
pub(super) const NEAR: Point = Point { x: 4, y: 6 };
/// Its opposite corner: the part is 20 by 10 pixels.
pub(super) const FAR: Point = Point { x: 23, y: 15 };
/// The PNG the media worker gives.
pub(super) const PNG: [u8; 4] = [0x89, 0x50, 0x4e, 0x47];
/// When the rig's clock starts.
const START: Mono = Mono::from_us(1_000);

/// `path` as a folder.
pub(super) fn dir(path: &str) -> FolderPath {
    FolderPath::new(PathBuf::from(path)).unwrap()
}

/// The file `name` of `kind` in the folder at `path`.
pub(super) fn file(path: &str, name: &str, kind: FileKind) -> SnipFile {
    SnipFile::new(PathBuf::from(path).join(name), kind).unwrap()
}

/// The fakes behind the actor's ports, shared with it.
#[derive(Clone)]
pub(super) struct Fakes {
    pub clock: FakeClock,
    pub calendar: FakeCalendar,
    pub freeze: FakeScreenFreeze,
    pub folder: FakeSnipFolder,
    pub seal: MicSeal,
    pub recorder: FakeRecorder,
    pub dialogs: FakeFileAsk,
    pub player: FakeMediaOpen,
    pub clip: Arc<FakeClipboardWriter>,
    pub pictures: FakePictures,
    pub reveal: Arc<FakeFolderReveal>,
    pub leftovers: Leftovers,
}

impl Fakes {
    /// One screen, the Snips folder with room, everything ready to record, and a PNG for each picture.
    pub fn new() -> Self {
        Self::making(Ok(PNG.to_vec()))
    }

    /// As `new`, the media worker answering each picture's PNG with `png`.
    pub fn making(png: Result<Vec<u8>, PictureError>) -> Self {
        let clock = FakeClock::new(START);
        let folder = FakeSnipFolder::new(Arc::new(clock.clone()), dir(SNIPS), dir(UNSAVED));
        Self::over(clock, folder, png)
    }

    /// As `new`, in a hidden run with no Snips folder (ADR-0037).
    pub fn hidden() -> Self {
        let clock = FakeClock::new(START);
        let folder = FakeSnipFolder::hidden(Arc::new(clock.clone()));
        Self::over(clock, folder, Ok(PNG.to_vec()))
    }

    /// The fakes over `folder`, read by `clock`, giving `png` for each picture.
    fn over(clock: FakeClock, folder: FakeSnipFolder, png: Result<Vec<u8>, PictureError>) -> Self {
        let seal = MicSeal::new();
        let pictures = FakePictures::new();
        pictures.queue_png(png);
        Self {
            calendar: FakeCalendar::new(Date::new(2026, 11, 24).unwrap(), 14, 5, 60),
            freeze: FakeScreenFreeze::new(vec![(SCREEN, 96)], Point { x: 10, y: 10 }),
            recorder: FakeRecorder::new(seal.clone(), &folder),
            dialogs: FakeFileAsk::new(&folder),
            player: FakeMediaOpen::new(&folder),
            clip: Arc::new(FakeClipboardWriter::new()),
            reveal: Arc::new(FakeFolderReveal::new()),
            leftovers: Leftovers::default(),
            pictures,
            seal,
            folder,
            clock,
        }
    }

    /// The actor's ports over these fakes, with jobs run in place.
    pub fn ports(
        &self,
        bus: Bus,
        handle: SettingsHandle,
        back: Sender<super::super::Msg>,
    ) -> Ports {
        Ports {
            bus,
            clock: Arc::new(self.clock.clone()),
            calendar: Arc::new(self.calendar.clone()),
            freeze: Arc::new(self.freeze.clone()),
            recorder: Arc::new(self.recorder.clone()),
            mics: Arc::new(self.seal.clone()),
            folder: Arc::new(self.folder.clone()),
            dialogs: Arc::new(self.dialogs.clone()),
            player: Arc::new(self.player.clone()),
            clip: self.clip.clone(),
            pictures: Arc::new(self.pictures.clone()),
            reveal: self.reveal.clone(),
            handle,
            back,
            spawn: in_place,
            leftovers: Arc::clone(&self.leftovers),
        }
    }
}
