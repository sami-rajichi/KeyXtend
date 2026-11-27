//! The fake dialogs, held to the port: picks answered in turn and Cancel after, an answer of another kind refused, a
//! chosen file written only through its answer, a full or read-only place refusing, and where each dialog opened with
//! which limits.

use super::*;
use crate::snip::tests::{CHOSEN, PNG, SNIPS, UNSAVED, WITHIN, dir, file, png, rig};
use kx_module_api::ports::SnipFolder;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

/// A recording's length.
const LENGTH_MS: u32 = 2_500;

/// Each pick limit at the start of its range.
const fn limits() -> PickLimits {
    PickLimits {
        job_mb: *PickLimits::JOB_MB.start(),
        dialog_min: *PickLimits::DIALOG_MIN.start(),
        idle_s: *PickLimits::IDLE_S.start(),
        restart_ms: *PickLimits::RESTART_MS.start(),
        restart_max_ms: *PickLimits::RESTART_MAX_MS.start(),
    }
}

/// The picture `PNG` to write.
fn bytes() -> Redacted<Vec<u8>> {
    Redacted::new(PNG.to_vec())
}

/// The name Save proposes for `kind`, in the Snips folder.
fn proposed(kind: FileKind) -> SnipFile {
    let name = PathBuf::from("Snip").with_extension(kind.ext());
    file(SNIPS, name.to_str().unwrap(), kind)
}

/// The file the user chose for `kind`, outside the Snips folder.
fn chosen(kind: FileKind) -> SnipFile {
    let name = PathBuf::from("mine").with_extension(kind.ext());
    file(CHOSEN, name.to_str().unwrap(), kind)
}

/// A recording of the unsaved part, held by `folder`.
fn recording(folder: &FakeSnipFolder) -> SnipFile {
    let video = file(UNSAVED, "1.mp4", FileKind::Mp4);
    let length_ms = LENGTH_MS;
    folder.put(video.clone(), Stored::Video { length_ms });
    video
}

/// The error a save answered, or `None` for an answer.
fn refusal(answer: Result<Option<Box<dyn SaveTo>>, AskError>) -> Option<AskError> {
    answer.err()
}

#[test]
fn a_chosen_file_takes_the_picture_there_and_nothing_else() {
    let (_, folder) = rig();
    let ask = FakeFileAsk::new(&folder);
    let video = recording(&folder);
    ask.queue_save(Pick::File(chosen(FileKind::Png)));
    let to = ask
        .save(&proposed(FileKind::Png), &limits())
        .unwrap()
        .unwrap();
    assert_eq!(to.file(), &chosen(FileKind::Png));
    assert_eq!(to.png(&bytes()), Ok(()));
    assert_eq!(folder.stored(&chosen(FileKind::Png)), Some(png()));
    assert_eq!(to.copy(&video), Err(SaveError::BadPath), "no video");
    assert_eq!(ask.saves(), [proposed(FileKind::Png)]);
}

#[test]
fn a_chosen_video_copies_only_a_recording_of_the_unsaved_part() {
    let (_, folder) = rig();
    let ask = FakeFileAsk::new(&folder);
    let video = recording(&folder);
    let elsewhere = file(SNIPS, "2.mp4", FileKind::Mp4);
    folder.put(elsewhere.clone(), Stored::Video { length_ms: 1 });
    let gone = file(UNSAVED, "3.mp4", FileKind::Mp4);
    ask.queue_save(Pick::File(chosen(FileKind::Mp4)));
    let to = ask
        .save(&proposed(FileKind::Mp4), &limits())
        .unwrap()
        .unwrap();
    assert_eq!(to.png(&bytes()), Err(SaveError::BadPath), "no picture");
    assert_eq!(to.copy(&elsewhere), Err(SaveError::BadPath));
    assert_eq!(to.copy(&gone), Err(SaveError::Missing));
    assert_eq!(to.copy(&video), Ok(()));
    let copied = Stored::Video {
        length_ms: LENGTH_MS,
    };
    assert_eq!(folder.stored(&chosen(FileKind::Mp4)), Some(copied));
    assert_eq!(folder.unsaved().unwrap().len(), 1, "the recording stays");
}

#[test]
fn picks_come_in_turn_then_cancel_and_another_kind_is_refused() {
    let (_, folder) = rig();
    let ask = FakeFileAsk::new(&folder);
    let asked = proposed(FileKind::Png);
    ask.queue_save(Pick::Cancel);
    ask.queue_save(Pick::Fail(AskError::Failed));
    ask.queue_save(Pick::File(chosen(FileKind::Mp4)));
    assert!(matches!(ask.save(&asked, &limits()), Ok(None)));
    assert_eq!(refusal(ask.save(&asked, &limits())), Some(AskError::Failed));
    assert_eq!(
        refusal(ask.save(&asked, &limits())),
        Some(AskError::BadPath)
    );
    assert!(
        matches!(ask.save(&asked, &limits()), Ok(None)),
        "Cancel once none is left"
    );
    assert_eq!(ask.saves(), vec![asked; 4]);
    assert!(folder.files().is_empty());
}

#[test]
fn a_full_or_read_only_place_refuses_every_write() {
    let (_, folder) = rig();
    let ask = FakeFileAsk::new(&folder);
    let video = recording(&folder);
    ask.queue_save(Pick::Stuck(chosen(FileKind::Png), SaveError::Full));
    ask.queue_save(Pick::Stuck(chosen(FileKind::Mp4), SaveError::ReadOnly));
    let to = ask
        .save(&proposed(FileKind::Png), &limits())
        .unwrap()
        .unwrap();
    assert_eq!(to.png(&bytes()), Err(SaveError::Full));
    let to = ask
        .save(&proposed(FileKind::Mp4), &limits())
        .unwrap()
        .unwrap();
    assert_eq!(to.copy(&video), Err(SaveError::ReadOnly));
    assert_eq!(folder.files(), [video], "nothing was written");
}

#[test]
fn the_folder_dialog_answers_in_turn_and_keeps_where_it_opened() {
    let (_, folder) = rig();
    let ask = FakeFileAsk::new(&folder);
    ask.queue_folder(Ok(Some(dir(CHOSEN))));
    ask.queue_folder(Err(AskError::Gone));
    assert_eq!(ask.folder(&dir(SNIPS), &limits()), Ok(Some(dir(CHOSEN))));
    assert_eq!(ask.folder(&dir(SNIPS), &limits()), Err(AskError::Gone));
    assert_eq!(
        ask.folder(&dir(CHOSEN), &limits()),
        Ok(None),
        "Cancel once none is left"
    );
    assert_eq!(ask.folders(), [dir(SNIPS), dir(SNIPS), dir(CHOSEN)]);
}

#[test]
fn each_dialog_keeps_its_limits_and_limits_out_of_range_show_none() {
    let (_, folder) = rig();
    let ask = FakeFileAsk::new(&folder);
    let longer = PickLimits {
        dialog_min: limits().dialog_min + 1,
        ..limits()
    };
    assert!(matches!(
        ask.save(&proposed(FileKind::Png), &limits()),
        Ok(None)
    ));
    assert_eq!(ask.folder(&dir(SNIPS), &longer), Ok(None));
    let bad = PickLimits {
        job_mb: 0,
        ..limits()
    };
    let refused = refusal(ask.save(&proposed(FileKind::Png), &bad));
    assert_eq!(refused, Some(AskError::Range("job_mb")));
    assert_eq!(
        ask.folder(&dir(SNIPS), &bad),
        Err(AskError::Range("job_mb"))
    );
    assert_eq!(ask.limits(), [limits(), longer]);
    assert_eq!((ask.saves().len(), ask.folders().len()), (1, 1));
}

#[test]
fn a_test_waits_for_dialogs_and_writes_on_another_thread() {
    let (_, folder) = rig();
    let ask = FakeFileAsk::new(&folder);
    assert!(!ask.wait_saves(1, Duration::ZERO), "none yet");
    ask.queue_save(Pick::File(chosen(FileKind::Png)));
    let port: Arc<dyn FileAsk> = Arc::new(ask.clone());
    let module = thread::spawn(move || {
        let to = port
            .save(&proposed(FileKind::Png), &limits())
            .ok()
            .flatten();
        let wrote = to.is_some_and(|to| to.png(&bytes()).is_ok());
        wrote && port.folder(&dir(SNIPS), &limits()).is_ok()
    });
    assert!(ask.wait_saves(1, WITHIN));
    assert!(ask.wait_writes(1, WITHIN));
    assert!(ask.wait_folders(1, WITHIN));
    assert!(module.join().unwrap());
    assert_eq!(folder.stored(&chosen(FileKind::Png)), Some(png()));
}

#[test]
fn clones_share_their_picks_and_debug_shows_no_path() {
    let (_, folder) = rig();
    let ask = FakeFileAsk::new(&folder);
    let port: Box<dyn FileAsk> = Box::new(ask.clone());
    ask.queue_save(Pick::File(chosen(FileKind::Png)));
    assert!(
        port.save(&proposed(FileKind::Png), &limits())
            .unwrap()
            .is_some()
    );
    let shown = format!("{ask:?}");
    assert!(shown.contains("FakeFileAsk"), "{shown}");
    assert!(!shown.contains("Alex"), "{shown}");
}
