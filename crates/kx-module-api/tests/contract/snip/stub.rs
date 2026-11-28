//! A platform for the snip contract that answers every port at once, as a stand-in: a desktop too big for the least
//! cap whose freeze is one pixel, a recording that tells one second and its end and hears the microphone only with
//! the app's seal, a Save dialog whose user keeps the name, a folder dialog that keeps its folder, an `unsaved` part
//! holding one day-old video, a PNG for any shot, a video never there to play, and a clipboard that takes only kept
//! copies.

use kx_module_api::Redacted;
use kx_module_api::ports::{
    AskError, BASE_DPI, BGRX_BYTES, ClipError, ClipPart, ClipWait, ClipboardWriter, DibAsk,
    EncodeAsk, FileAsk, FileKind, FolderPath, FreezeError, Frozen, FrozenScreen, Gear, MB,
    MediaOpen, MicPass, MicSeal, PickLimits, PictureAsk, PictureConfig, PictureError, PictureRun,
    Pictures, Pixels, PlayError, Point, RecordAsk, RecordDone, RecordEnd, RecordError, RecordFacts,
    RecordNote, RecordRun, RecordWaits, Recorder, Rect, SaveError, SaveTo, ScreenFreeze, Shot,
    ShotCap, SnipFile, SnipFolder, SpeedAsk, UnsavedFile,
};
use kx_test_support::paths::native_path;
use std::sync::LazyLock;
use std::time::Duration;

/// The app's one microphone seal, which the host provides under `MicPassKey` and the stub's recorder checks.
pub(super) static SEAL: LazyLock<MicSeal> = LazyLock::new(MicSeal::new);

/// How long the stub's recording runs, in ms.
pub(super) const LENGTH_MS: u32 = 1000;
/// What the stub's desktop would take frozen, in MB: more than the least cap, less than the most.
pub(super) const DESKTOP_MB: u32 = 128;
/// The stub's Snips folder, written for Windows and read through `native_path`.
const SNIPS: &str = r"C:\snips";
/// The stub's `unsaved` part.
const UNSAVED: &str = r"C:\snips\unsaved";
/// The age of the stub's one unsaved video: a day.
const AGE: Duration = Duration::from_hours(24);
/// Bytes free on the stub's drive.
const FREE: u64 = 1 << 32;
/// The code the stub refuses with, as a system that cannot freeze would.
const REFUSED: i32 = -1;
/// The bytes every PNG starts with, all the stub's encoder answers.
const PNG_HEAD: [u8; 8] = *b"\x89PNG\r\n\x1a\n";

/// The stub's `unsaved` part.
#[allow(clippy::unwrap_used, reason = "a plain absolute path")]
pub(super) fn unsaved() -> FolderPath {
    FolderPath::new(native_path(UNSAVED)).unwrap()
}

/// The video `name` of the stub's `unsaved` part; `name` must end in `.mp4`.
#[allow(clippy::unwrap_used, reason = "the tests name only videos")]
pub(super) fn video(name: &str) -> SnipFile {
    SnipFile::new(native_path(&format!(r"{UNSAVED}\{name}")), FileKind::Mp4).unwrap()
}

/// Each job wait at the start of its range.
pub(super) const fn waits() -> RecordWaits {
    RecordWaits {
        start_ms: *RecordWaits::START_MS.start(),
        tick_ms: *RecordWaits::TICK_MS.start(),
        end_ms: *RecordWaits::END_MS.start(),
        switch_ms: *RecordWaits::SWITCH_MS.start(),
        speed_ms: *RecordWaits::SPEED_MS.start(),
        speed_mb_ms: *RecordWaits::SPEED_MB_MS.start(),
    }
}

/// Each pick limit at the start of its range.
pub(super) const fn limits() -> PickLimits {
    PickLimits {
        job_mb: *PickLimits::JOB_MB.start(),
        dialog_min: *PickLimits::DIALOG_MIN.start(),
        idle_s: *PickLimits::IDLE_S.start(),
        restart_ms: *PickLimits::RESTART_MS.start(),
        restart_max_ms: *PickLimits::RESTART_MAX_MS.start(),
    }
}

/// A platform that answers every snip port at once.
pub(super) struct Stub;

/// Refuses a microphone pass the app's seal did not make.
fn sealed(pass: Option<&MicPass>) -> Result<(), RecordError> {
    let opens = pass.is_none_or(|p| SEAL.opens(p));
    opens.then_some(()).ok_or(RecordError::BadPass)
}

/// The stub's recording: it holds its file and fails only for a foreign pass.
struct Run(SnipFile);

impl RecordRun for Run {
    fn file(&self) -> &SnipFile {
        &self.0
    }
    fn mic(&mut self, on: Option<MicPass>) -> Result<(), RecordError> {
        sealed(on.as_ref())
    }
    fn sound(&mut self, _: bool) -> Result<(), RecordError> {
        Ok(())
    }
    fn stop(&mut self) -> Result<(), RecordError> {
        Ok(())
    }
    fn cancel(&mut self) {}
}

impl ScreenFreeze for Stub {
    fn freeze(&self, cap: ShotCap) -> Result<Frozen, FreezeError> {
        if cap.bytes() < u64::from(DESKTOP_MB) * MB {
            return Err(FreezeError::TooBig);
        }
        let rect = Rect {
            x: 0,
            y: 0,
            w: 1,
            h: 1,
        };
        let refused = FreezeError::Refused(REFUSED);
        let shot = Shot::new(1, 1, vec![0; BGRX_BYTES]).ok_or(refused)?;
        let screen = FrozenScreen::new(rect, BASE_DPI, shot).ok_or(refused)?;
        Ok(Frozen {
            screens: vec![screen],
            windows: Vec::new(),
            pointer: Point::default(),
        })
    }
}

impl Recorder for Stub {
    fn facts(&self) -> RecordFacts {
        RecordFacts {
            video: true,
            mic: Gear::Missing,
            sound: Gear::Ready,
        }
    }

    fn start(
        &self,
        ask: RecordAsk,
        tell: Box<dyn Fn(RecordNote) + Send>,
    ) -> Result<Box<dyn RecordRun>, RecordError> {
        ask.check()?;
        sealed(ask.mic.as_ref())?;
        tell(RecordNote::Time { ms: 0 });
        let end = RecordEnd::Stopped;
        tell(RecordNote::Done(RecordDone {
            length_ms: LENGTH_MS,
            end,
        }));
        Ok(Box::new(Run(video("a.mp4"))))
    }

    fn speed(&self, ask: &SpeedAsk) -> Result<SnipFile, RecordError> {
        ask.check()?;
        Ok(video("b.mp4"))
    }
}

impl SnipFolder for Stub {
    fn snips(&self) -> Option<FolderPath> {
        FolderPath::new(native_path(SNIPS)).ok()
    }

    fn unsaved(&self) -> Result<Vec<UnsavedFile>, SaveError> {
        let file = video("a.mp4");
        Ok(vec![UnsavedFile {
            file,
            age: AGE,
            copied: Some(AGE),
        }])
    }

    fn mark_copied(&self, video: &SnipFile) -> Result<(), SaveError> {
        mine(video)
    }

    fn prune_marks(&self) -> Result<(), SaveError> {
        Ok(())
    }

    fn remove(&self, file: &SnipFile) -> Result<(), SaveError> {
        mine(file)
    }

    fn free(&self) -> Result<u64, SaveError> {
        Ok(FREE)
    }
}

/// Refuses a file outside the stub's `unsaved` part.
fn mine(file: &SnipFile) -> Result<(), SaveError> {
    let inside = file.kind() == FileKind::Mp4 && file.lies_in(&unsaved());
    inside.then_some(()).ok_or(SaveError::BadPath)
}

/// The stub's answer: the file its user kept, written nowhere.
struct Answer(SnipFile);

impl SaveTo for Answer {
    fn file(&self) -> &SnipFile {
        &self.0
    }

    fn png(&self, _: &Redacted<Vec<u8>>) -> Result<(), SaveError> {
        let picture = self.0.kind() == FileKind::Png;
        picture.then_some(()).ok_or(SaveError::BadPath)
    }

    fn copy(&self, video: &SnipFile) -> Result<(), SaveError> {
        let film = self.0.kind() == FileKind::Mp4;
        film.then_some(()).ok_or(SaveError::BadPath)?;
        mine(video)
    }
}

impl FileAsk for Stub {
    fn save(
        &self,
        proposed: &SnipFile,
        limits: &PickLimits,
    ) -> Result<Option<Box<dyn SaveTo>>, AskError> {
        limits.check()?;
        Ok(Some(Box::new(Answer(proposed.clone()))))
    }

    fn folder(&self, at: &FolderPath, limits: &PickLimits) -> Result<Option<FolderPath>, AskError> {
        limits.check()?;
        Ok(Some(at.clone()))
    }
}

impl Pictures for Stub {
    fn open(&self, _: PictureConfig) -> Box<dyn PictureRun> {
        Box::new(Self)
    }
}

/// The stub's media worker reads no image and makes no DIB, but encodes any shot within its bound.
impl PictureRun for Stub {
    fn draw(&mut self, _: PictureAsk) -> Result<Pixels, PictureError> {
        Err(PictureError::Refused)
    }

    fn dib(&mut self, _: DibAsk) -> Result<Redacted<Vec<u8>>, PictureError> {
        Err(PictureError::Refused)
    }

    fn encode(&mut self, ask: EncodeAsk) -> Result<Redacted<Vec<u8>>, PictureError> {
        ask.check()?;
        Ok(Redacted::new(PNG_HEAD.to_vec()))
    }
}

impl MediaOpen for Stub {
    fn play(&self, _: &SnipFile) -> Result<(), PlayError> {
        Err(PlayError::Missing)
    }
}

impl ClipboardWriter for Stub {
    fn write(&self, _: &[ClipPart], _: ClipWait) -> Result<(), ClipError> {
        Err(ClipError::Refused)
    }

    fn write_lent(&self, _: &[ClipPart], _: ClipWait) -> Result<(), ClipError> {
        Err(ClipError::Refused)
    }

    fn write_kept(&self, _: &[ClipPart], _: ClipWait) -> Result<(), ClipError> {
        Ok(())
    }
}
