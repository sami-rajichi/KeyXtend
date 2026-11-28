//! Snip's place under the data folder, the ports a kit makes for each run, and the recorder behind the app's seal: a
//! pass from another seal never reaches the platform's recorder, at the start or at a switch, while the app's own pass
//! and none go through.

use super::*;
use kx_module_api::Mono;
use kx_module_api::ports::{FolderPath, MicPass, Part, RecordAsk, RecordError, RecordWaits, Rect};
use kx_platform_fake::{FakeClock, FakeRecorder, FakeSnipFolder};
use kx_test_support::tempdir::TempDir;

mod kit;
mod make;

/// The file a test's guarded maker leaves in each folder it makes.
const MARK: &str = "guarded";

/// A guarded maker for tests: it makes `folder` and leaves `MARK` in it.
fn marked(folder: &Path) -> io::Result<()> {
    fs::create_dir(folder)?;
    fs::write(folder.join(MARK), [])
}

/// The part filmed.
const PART: Rect = Rect {
    x: 0,
    y: 0,
    w: 64,
    h: 36,
};

/// An empty Snips folder in memory at `dirs`' place.
fn folder(dirs: &AppDirs) -> FakeSnipFolder {
    let clock = Arc::new(FakeClock::new(Mono::from_us(0)));
    let (snips, unsaved) = snip_dirs(dirs.data_dir().path());
    let (snips, unsaved) = (FolderPath::new(snips), FolderPath::new(unsaved));
    FakeSnipFolder::new(clock, snips.unwrap(), unsaved.unwrap())
}

/// A program folder whose installed data folder lies in `user`.
fn dirs(user: &TempDir) -> AppDirs {
    AppDirs {
        exe_dir: user.path().join("exe"),
        user_dir: user.path().to_path_buf(),
    }
}

/// A recording of `PART` at the least values, hearing the microphone with `mic`.
fn ask(mic: Option<MicPass>) -> RecordAsk {
    RecordAsk {
        part: Part::new(PART).unwrap(),
        fps: RecordAsk::FPS[0],
        mic,
        sound: false,
        max_min: *RecordAsk::MAX_MIN.start(),
        min_free_mb: *RecordAsk::MIN_FREE_MB.start(),
        job_mb: *RecordAsk::JOB_MB.start(),
        waits: RecordWaits {
            start_ms: *RecordWaits::START_MS.start(),
            tick_ms: *RecordWaits::TICK_MS.start(),
            end_ms: *RecordWaits::END_MS.start(),
            switch_ms: *RecordWaits::SWITCH_MS.start(),
            speed_ms: *RecordWaits::SPEED_MS.start(),
            speed_mb_ms: *RecordWaits::SPEED_MB_MS.start(),
        },
    }
}

/// The app's seal over a platform recorder holding `inner`'s seal, which the test reads.
fn sealed(app: &MicSeal, inner: &MicSeal, user: &TempDir) -> (Sealed, FakeRecorder) {
    let platform = FakeRecorder::new(inner.clone(), &folder(&dirs(user)));
    let recorder = Sealed::new(app.clone(), Arc::new(platform.clone()));
    (recorder, platform)
}

#[test]
fn the_unsaved_part_lies_in_the_snips_folder_under_the_data_folder() {
    let user = TempDir::new("snip-dirs").unwrap();
    let dirs = dirs(&user);
    let data = dirs.data_dir().path().to_path_buf();
    assert_eq!(unsaved(&dirs), data.join(SNIP_DIR).join(UNSAVED_DIR));
}

#[test]
fn a_start_with_another_seals_pass_is_refused_before_the_recorder_hears_it() {
    let (app, other) = (MicSeal::new(), MicSeal::new());
    let user = TempDir::new("snip-start").unwrap();
    let (recorder, platform) = sealed(&app, &other, &user);
    let got = recorder.start(ask(Some(other.pass())), Box::new(|_| {}));
    assert_eq!(got.err(), Some(RecordError::BadPass));
    assert!(platform.starts().is_empty(), "the platform never heard it");
}

#[test]
fn a_switch_with_another_seals_pass_is_refused_before_the_run_hears_it() {
    let (app, other) = (MicSeal::new(), MicSeal::new());
    let user = TempDir::new("snip-switch").unwrap();
    let (recorder, platform) = sealed(&app, &other, &user);
    let mut run = recorder.start(ask(None), Box::new(|_| {})).unwrap();
    assert_eq!(run.mic(Some(other.pass())), Err(RecordError::BadPass));
    assert_eq!(platform.runs()[0].heard.len(), 1, "no switch reached it");
    assert_eq!(run.file(), &platform.runs()[0].file);
    run.cancel();
}

#[test]
fn the_apps_own_pass_and_none_go_through_to_the_platform() {
    let app = MicSeal::new();
    let user = TempDir::new("snip-own").unwrap();
    let (recorder, platform) = sealed(&app, &app, &user);
    assert_eq!(recorder.facts(), platform.facts());
    let mut run = recorder
        .start(ask(Some(app.pass())), Box::new(|_| {}))
        .unwrap();
    assert_eq!(run.mic(None), Ok(()));
    assert_eq!(run.mic(Some(app.pass())), Ok(()));
    let heard: Vec<bool> = platform.runs()[0].heard.iter().map(|h| h.mic).collect();
    assert_eq!(heard, [true, false, true], "the start's, then each switch");
    assert_eq!(run.stop(), Ok(()));
    assert_eq!(platform.runs()[0].stops, 1);
}
