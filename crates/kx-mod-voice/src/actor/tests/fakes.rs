//! The fakes behind the actor's ports, drive D: holding the models folder with English's default model, and the
//! English layout and a run's end the tests tell.

use std::path::Path;
use std::sync::Arc;

use kx_module_api::ports::{Date, Drive, LayoutChanged, LayoutId, MB, SpeechDone};
use kx_module_api::{Mono, Redacted};
use kx_platform_fake::{
    FakeCalendar, FakeClipboardWriter, FakeClock, FakeCloud, FakeFieldWatch, FakeFolderReveal,
    FakeModelFiles, FakeSessionWatch, FakeShelf, FakeSpeech, FakeWebLink,
};

use crate::models::{Row, table};

/// The models folder the rig holds.
pub(super) const DIR: &str = r"D:\KeyXtend\models";
/// When the rig's clock starts.
const START: Mono = Mono::from_us(1_000);

/// The fakes behind the actor's ports, shared with it.
#[derive(Clone)]
pub(super) struct Fakes {
    pub speech: FakeSpeech,
    pub files: FakeModelFiles,
    pub clip: Arc<FakeClipboardWriter>,
    pub reveal: Arc<FakeFolderReveal>,
    pub shelf: FakeShelf,
    pub field: FakeFieldWatch,
    pub session: FakeSessionWatch,
    pub clock: FakeClock,
    pub calendar: FakeCalendar,
    pub cloud: FakeCloud,
    pub link: Arc<FakeWebLink>,
}

impl Fakes {
    /// Drive D: with room, the models folder holding English's default model, a text box in front.
    pub(super) fn new() -> Self {
        let fakes = Self {
            speech: FakeSpeech::new(),
            files: FakeModelFiles::new(),
            clip: Arc::new(FakeClipboardWriter::new()),
            reveal: Arc::new(FakeFolderReveal::new()),
            shelf: FakeShelf::new(),
            field: FakeFieldWatch::new(),
            session: FakeSessionWatch::new(),
            clock: FakeClock::new(START),
            calendar: FakeCalendar::new(Date::new(2026, 11, 15).unwrap(), 12, 0, 60),
            cloud: FakeCloud::new(),
            link: Arc::new(FakeWebLink::new()),
        };
        let drive = Drive {
            root: Redacted::new(r"D:\".into()),
            name: Redacted::new("D:".into()),
            free: 900_000 * MB,
        };
        fakes.files.set_drives(vec![drive]);
        let row = english();
        let file = Path::new(DIR).join(&row.file);
        fakes.files.add_file(&file, row.bytes, row.print);
        fakes
    }
}

/// English's default model.
pub(super) fn english() -> &'static Row {
    let table = table().unwrap();
    table.default_for("en").unwrap()
}

/// The English layout.
pub(super) fn english_layout() -> LayoutChanged {
    LayoutChanged {
        id: LayoutId::new(1),
        iso: "en".into(),
        name: "English".into(),
        rtl: false,
    }
}

/// A run's end after `pieces` pieces.
pub(super) fn done(pieces: u16) -> SpeechDone {
    SpeechDone {
        pieces,
        sound_ms: 4_000,
        wait_ms: 900,
        load_ms: 0,
        memory_mb: 800,
        card: false,
    }
}
